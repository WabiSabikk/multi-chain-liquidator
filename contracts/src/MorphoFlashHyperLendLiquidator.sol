// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

interface IMorphoFlash {
    function flashLoan(address token, uint256 assets, bytes calldata data) external;
}

/// @title MorphoFlashHyperLendLiquidator
/// @notice Uses Morpho Blue flash loans (0 bps fee!) to liquidate HyperLend positions
/// @dev Flow: morpho.flashLoan(debtToken, amount) → callback →
///   liquidationCall → swap collateral → repay Morpho (0 premium) → profit
contract MorphoFlashHyperLendLiquidator {
    address internal constant HYPERLEND_POOL = 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b;
    address internal constant MORPHO = 0x68e37dE8d93d3496ae143F2E900490f6280C57cD;
    address internal constant HYPERSWAP_ROUTER = 0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D;

    uint24 internal constant FEE_100  = 100;
    uint24 internal constant FEE_500  = 500;
    uint24 internal constant FEE_3000 = 3_000;

    error NotOwner();
    error NotMorpho();
    error InsufficientProfit(uint256 actual, uint256 required);
    error TransferFailed();
    error RouterNotApproved(address router);
    error SwapFailed();

    event LiquidationExecuted(
        address indexed collateralAsset,
        address indexed debtAsset,
        address indexed user,
        uint256 debtCovered,
        uint256 profit
    );

    address public immutable OWNER;
    mapping(address => mapping(address => uint24)) public feeTiers;
    mapping(address => bool) public approvedRouters;
    mapping(address => bool) public authorizedCallers;

    modifier onlyOwner() {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
        _;
    }

    constructor() {
        OWNER = msg.sender;
        approvedRouters[HYPERSWAP_ROUTER] = true;

        address WHYPE = 0x5555555555555555555555555555555555555555;
        address kHYPE = 0xfD739d4e423301CE9385c1fb8850539D657C296D;
        address wstHYPE = 0x94e8396e0869c9F2200760aF0621aFd240E1CF38;
        address beHYPE = 0xd8FC8F0b03eBA61F64D08B0bef69d80916E5DdA9;
        address USDC  = 0xb88339CB7199b77E23DB6E890353E22632Ba630f;
        address USDT0 = 0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb;
        address USDe  = 0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34;
        address UETH  = 0xBe6727B535545C67d5cAa73dEa54865B92CF7907;
        address UBTC  = 0x9FDBdA0A5e284c32744D2f17Ee5c74B284993463;

        _setFeeTier(kHYPE, WHYPE, FEE_100);
        _setFeeTier(wstHYPE, WHYPE, FEE_100);
        _setFeeTier(beHYPE, WHYPE, FEE_100);
        _setFeeTier(WHYPE, USDC, FEE_500);
        _setFeeTier(WHYPE, USDT0, FEE_500);
        _setFeeTier(WHYPE, USDe, FEE_3000);
        _setFeeTier(WHYPE, UETH, FEE_3000);
        _setFeeTier(WHYPE, UBTC, FEE_3000);
        _setFeeTier(USDC, USDT0, FEE_100);
        _setFeeTier(USDC, USDe, FEE_100);
        _setFeeTier(USDe, USDT0, FEE_100);
    }

    /// @notice Execute HyperLend liquidation with custom swap routing
    function executeLiquidation(
        address collateralAsset, address debtAsset, address user,
        uint256 debtToCover, uint256 minProfit, bytes calldata swapData, address swapRouter
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, debtAsset, user, minProfit, swapData, swapRouter);
        IMorphoFlash(MORPHO).flashLoan(debtAsset, debtToCover, params);
    }

    /// @notice Execute HyperLend liquidation with default HyperSwap routing
    function executeLiquidation(
        address collateralAsset, address debtAsset, address user,
        uint256 debtToCover, uint256 minProfit
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, debtAsset, user, minProfit, bytes(""), address(0));
        IMorphoFlash(MORPHO).flashLoan(debtAsset, debtToCover, params);
    }

    /// @notice Morpho flash loan callback — 0 fee, just repay principal
    function onMorphoFlashLoan(uint256 assets, bytes calldata data) external {
        if (msg.sender != MORPHO) revert NotMorpho();

        (
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(data, (address, address, address, uint256, bytes, address));

        // 1. Approve HyperLend pool to pull debt tokens for liquidation
        IERC20(debtAsset).approve(HYPERLEND_POOL, assets);

        // 2. Execute HyperLend liquidation — seizes collateral to this contract
        IPool(HYPERLEND_POOL).liquidationCall(collateralAsset, debtAsset, user, assets, false);

        // 3. Swap seized collateral back to debt token
        if (collateralAsset != debtAsset) {
            uint256 collateralBalance = IERC20(collateralAsset).balanceOf(address(this));
            _swapCollateral(collateralAsset, debtAsset, collateralBalance, assets, swapData, swapRouter);
        }

        // 4. Verify profit
        uint256 debtBalance = IERC20(debtAsset).balanceOf(address(this));
        uint256 profit = debtBalance > assets ? debtBalance - assets : 0;
        if (profit < minProfit) revert InsufficientProfit(profit, minProfit);

        // 5. Approve Morpho to pull back principal (0 premium!)
        IERC20(debtAsset).approve(MORPHO, assets);

        // 6. Send profit to owner
        if (profit > 0) {
            if (!IERC20(debtAsset).transfer(OWNER, profit)) revert TransferFailed();
        }

        emit LiquidationExecuted(collateralAsset, debtAsset, user, assets, profit);
    }

    function _swapCollateral(
        address tokenIn, address tokenOut, uint256 amountIn,
        uint256 amountOutMinimum, bytes memory swapData, address swapRouter
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
                tokenIn: tokenIn, tokenOut: tokenOut, fee: fee,
                recipient: address(this), deadline: block.timestamp,
                amountIn: amountIn, amountOutMinimum: amountOutMinimum, sqrtPriceLimitX96: 0
            }));
        }
    }

    /// @notice Batch liquidate multiple positions in a single TX (crash mode).
    /// Uses Morpho flash loan for total debt, then liquidates each target sequentially.
    function batchLiquidate(
        address[] calldata collateralAssets,
        address[] calldata debtAssets,
        address[] calldata users,
        uint256[] calldata debtAmounts,
        bytes[] calldata swapDatas,
        address[] calldata swapRouters
    ) external onlyOwner {
        require(collateralAssets.length == users.length, "length mismatch");
        require(debtAssets.length == users.length, "length mismatch");
        require(debtAmounts.length == users.length, "length mismatch");

        // Execute each liquidation independently (each with its own Morpho flash loan)
        for (uint256 i = 0; i < users.length; i++) {
            bytes memory params = abi.encode(
                collateralAssets[i], debtAssets[i], users[i], uint256(0),
                swapDatas.length > i ? swapDatas[i] : bytes(""),
                swapRouters.length > i ? swapRouters[i] : address(0)
            );
            try IMorphoFlash(MORPHO).flashLoan(debtAssets[i], debtAmounts[i], params) {
                // Success — profit already sent to OWNER in callback
            } catch {
                // Skip failed liquidation, continue with next
            }
        }
    }

    function setFeeTier(address a, address b, uint24 fee) external onlyOwner { _setFeeTier(a, b, fee); }
    function setApprovedRouter(address router, bool approved) external onlyOwner { approvedRouters[router] = approved; }
    function setAuthorizedCaller(address caller, bool authorized) external { if (msg.sender != OWNER) revert NotOwner(); authorizedCallers[caller] = authorized; }
    function rescueTokens(address token) external onlyOwner { uint256 b = IERC20(token).balanceOf(address(this)); if (b > 0) { if (!IERC20(token).transfer(OWNER, b)) revert TransferFailed(); } }

    function _setFeeTier(address a, address b, uint24 fee) internal { feeTiers[a][b] = fee; feeTiers[b][a] = fee; }
    function _getFeeTier(address a, address b) internal view returns (uint24) { uint24 f = feeTiers[a][b]; return f == 0 ? FEE_3000 : f; }

    receive() external payable {}
}
