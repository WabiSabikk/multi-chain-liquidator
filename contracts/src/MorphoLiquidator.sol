// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

interface IMorpho {
    struct MarketParams {
        address loanToken;
        address collateralToken;
        address oracle;
        address irm;
        uint256 lltv;
    }

    function liquidate(
        MarketParams calldata marketParams,
        address borrower,
        uint256 seizedAssets,
        uint256 repaidShares,
        bytes calldata data
    ) external returns (uint256 assetsSeized, uint256 assetsRepaid);
}

/// @title MorphoLiquidator
/// @notice Callback-based liquidator for Morpho Blue on HyperEVM
/// @dev No flash loan needed — Morpho transfers collateral before callback,
///      then pulls loan tokens after callback returns.
contract MorphoLiquidator {
    error NotOwner();
    error NotMorpho();
    error InsufficientProfit(uint256 actual, uint256 required);
    error RouterNotApproved(address router);
    error SwapFailed();
    error TransferFailed();

    event LiquidationExecuted(
        bytes32 indexed marketId,
        address indexed borrower,
        address collateralToken,
        address loanToken,
        uint256 seizedCollateral,
        uint256 repaidLoan,
        uint256 profit
    );

    address public immutable OWNER;
    address public immutable MORPHO;

    // HyperSwap V3 Router on HyperEVM
    address internal constant HYPERSWAP_ROUTER = 0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D;

    uint24 internal constant FEE_100  = 100;
    uint24 internal constant FEE_500  = 500;
    uint24 internal constant FEE_3000 = 3_000;

    mapping(address => mapping(address => uint24)) public feeTiers;
    mapping(address => bool) public approvedRouters;
    mapping(address => bool) public authorizedCallers;

    modifier onlyOwner() {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
        _;
    }

    constructor(address _morpho) {
        OWNER = msg.sender;
        MORPHO = _morpho;
        approvedRouters[HYPERSWAP_ROUTER] = true;

        // Default fee tiers for HyperEVM tokens
        address WHYPE = 0x5555555555555555555555555555555555555555;
        address USDC  = 0xb88339CB7199b77E23DB6E890353E22632Ba630f;
        address USDT0 = 0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb;
        address USDe  = 0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34;
        address sUSDe = 0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2;

        _setFeeTier(WHYPE, USDC, FEE_3000);
        _setFeeTier(WHYPE, USDT0, FEE_3000);
        _setFeeTier(WHYPE, USDe, FEE_3000);
        _setFeeTier(USDC, USDT0, FEE_100);
        _setFeeTier(USDC, USDe, FEE_500);
        _setFeeTier(sUSDe, USDC, FEE_500);
        _setFeeTier(sUSDe, USDT0, FEE_500);
    }

    /// @notice Execute a Morpho Blue liquidation
    /// @param loanToken The loan (debt) token of the market
    /// @param collateralToken The collateral token of the market
    /// @param oracle The oracle address for the market
    /// @param irm The interest rate model address
    /// @param lltv The liquidation LTV (in WAD)
    /// @param borrower The borrower to liquidate
    /// @param seizedAssets Amount of collateral to seize
    /// @param minProfit Minimum profit in loan token units
    /// @param swapData Optional calldata for custom DEX routing (empty = HyperSwap V3 default)
    /// @param swapRouter Optional router address (ignored if swapData is empty)
    function liquidate(
        address loanToken,
        address collateralToken,
        address oracle,
        address irm,
        uint256 lltv,
        address borrower,
        uint256 seizedAssets,
        uint256 minProfit,
        bytes calldata swapData,
        address swapRouter
    ) external onlyOwner {
        IMorpho.MarketParams memory params = IMorpho.MarketParams({
            loanToken: loanToken,
            collateralToken: collateralToken,
            oracle: oracle,
            irm: irm,
            lltv: lltv
        });

        bytes memory callbackData = abi.encode(
            loanToken, collateralToken, minProfit, swapData, swapRouter
        );

        IMorpho(MORPHO).liquidate(params, borrower, seizedAssets, 0, callbackData);

        // After liquidation: send profit to owner
        uint256 loanBalance = IERC20(loanToken).balanceOf(address(this));
        if (loanBalance > 0) {
            if (!IERC20(loanToken).transfer(OWNER, loanBalance)) revert TransferFailed();
        }

        // Send any remaining collateral too
        uint256 colBalance = IERC20(collateralToken).balanceOf(address(this));
        if (colBalance > 0) {
            if (!IERC20(collateralToken).transfer(OWNER, colBalance)) revert TransferFailed();
        }
    }

    /// @notice Morpho callback — called after collateral is transferred to us
    /// @dev Must approve Morpho to pull `repaidAssets` of loan token
    function onMorphoLiquidate(uint256 repaidAssets, bytes calldata data) external {
        if (msg.sender != MORPHO) revert NotMorpho();

        (
            address loanToken,
            address collateralToken,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(data, (address, address, uint256, bytes, address));

        // Swap collateral -> loan token
        uint256 collateralBalance = IERC20(collateralToken).balanceOf(address(this));

        if (collateralToken != loanToken && collateralBalance > 0) {
            _swapCollateral(collateralToken, loanToken, collateralBalance, repaidAssets, swapData, swapRouter);
        }

        // Check we have enough to repay
        uint256 loanBalance = IERC20(loanToken).balanceOf(address(this));
        uint256 profit = 0;
        if (loanBalance > repaidAssets) {
            profit = loanBalance - repaidAssets;
        }
        if (profit < minProfit) revert InsufficientProfit(profit, minProfit);

        // Approve Morpho to pull repaid amount
        IERC20(loanToken).approve(MORPHO, repaidAssets);
    }

    function _swapCollateral(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 amountOutMinimum,
        bytes memory swapData,
        address swapRouter
    ) internal {
        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);
            IERC20(tokenIn).approve(swapRouter, 0);
            IERC20(tokenIn).approve(swapRouter, amountIn);
            (bool ok,) = swapRouter.call(swapData);
            if (!ok) revert SwapFailed();
            IERC20(tokenIn).approve(swapRouter, 0);
        } else {
            uint24 fee = _getFeeTier(tokenIn, tokenOut);
            IERC20(tokenIn).approve(HYPERSWAP_ROUTER, amountIn);
            ISwapRouter(HYPERSWAP_ROUTER).exactInputSingle(ISwapRouter.ExactInputSingleParams({
                tokenIn: tokenIn,
                tokenOut: tokenOut,
                fee: fee,
                recipient: address(this),
                deadline: block.timestamp,
                amountIn: amountIn,
                amountOutMinimum: amountOutMinimum,
                sqrtPriceLimitX96: 0
            }));
        }
    }

    function setFeeTier(address tokenA, address tokenB, uint24 fee) external onlyOwner {
        _setFeeTier(tokenA, tokenB, fee);
    }

    function setApprovedRouter(address router, bool approved) external onlyOwner {
        approvedRouters[router] = approved;
    }

    function setAuthorizedCaller(address caller, bool authorized) external {
        if (msg.sender != OWNER) revert NotOwner();
        authorizedCallers[caller] = authorized;
    }

    function rescueTokens(address token) external onlyOwner {
        uint256 b = IERC20(token).balanceOf(address(this));
        if (b > 0) {
            if (!IERC20(token).transfer(OWNER, b)) revert TransferFailed();
        }
    }

    function _setFeeTier(address a, address b, uint24 fee) internal {
        feeTiers[a][b] = fee;
        feeTiers[b][a] = fee;
    }

    function _getFeeTier(address a, address b) internal view returns (uint24) {
        uint24 f = feeTiers[a][b];
        return f == 0 ? FEE_3000 : f;
    }

    receive() external payable {}
}
