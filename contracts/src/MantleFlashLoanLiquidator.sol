// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

/// @title MantleFlashLoanLiquidator
/// @notice Aave V3 liquidations on Mantle chain via flash loans + FusionX V3 swaps
/// @dev Standalone contract — does NOT share Constants with Arbitrum version
contract MantleFlashLoanLiquidator {
    // ═══════════════════════════════════════════════════════════════
    //                     MANTLE ADDRESSES
    // ═══════════════════════════════════════════════════════════════
    address internal constant AAVE_POOL = 0x458F293454fE0d67EC0655f3672301301DD51422;
    address internal constant FUSIONX_V3_ROUTER = 0x5989FB161568b9F133eDf5Cf6787f5597762797F;

    // Tokens
    address internal constant WETH  = 0xdEAddEaDdeadDEadDEADDEAddEADDEAddead1111;
    address internal constant WMNT  = 0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8;
    address internal constant USDC  = 0x09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9;
    address internal constant USDT0 = 0x779Ded0c9e1022225f8E0630b35a9b54bE713736;
    address internal constant USDe  = 0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34;
    address internal constant sUSDe = 0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2;
    address internal constant FBTC  = 0xC96dE26018A54D51c097160568752c4E3BD6C364;
    address internal constant wrsETH = 0x93e855643e940D025bE2e529272e4Dbd15a2Cf74;
    address internal constant GHO   = 0xfc421aD3C883Bf9E7C4f42dE845C4e4405799e73;

    // Fee tiers (same as Uniswap V3)
    uint24 internal constant FEE_100  = 100;
    uint24 internal constant FEE_500  = 500;
    uint24 internal constant FEE_3000 = 3_000;
    uint24 internal constant FEE_10000 = 10_000;

    // ═══════════════════════════════════════════════════════════════
    //                          ERRORS
    // ═══════════════════════════════════════════════════════════════
    error NotOwner();
    error NotPool();
    error NotSelf();
    error InsufficientProfit(uint256 actual, uint256 required);
    error TransferFailed();
    error RouterNotApproved(address router);
    error SwapFailed();

    // ═══════════════════════════════════════════════════════════════
    //                          EVENTS
    // ═══════════════════════════════════════════════════════════════
    event LiquidationExecuted(
        address indexed collateralAsset,
        address indexed debtAsset,
        address indexed user,
        uint256 debtCovered,
        uint256 profit
    );

    // ═══════════════════════════════════════════════════════════════
    //                          STATE
    // ═══════════════════════════════════════════════════════════════
    address public immutable OWNER;
    mapping(address => mapping(address => uint24)) public feeTiers;
    mapping(address => bool) public approvedRouters;
    mapping(address => bool) public authorizedCallers;

    modifier onlyOwner() {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
        _;
    }

    // ═══════════════════════════════════════════════════════════════
    //                       CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════
    constructor() {
        OWNER = msg.sender;

        // FusionX V3 fee tiers for Mantle tokens
        _setFeeTier(WETH, USDC, FEE_500);
        _setFeeTier(WETH, USDT0, FEE_500);
        _setFeeTier(WETH, WMNT, FEE_3000);
        _setFeeTier(WETH, USDe, FEE_3000);
        _setFeeTier(WETH, wrsETH, FEE_500);
        _setFeeTier(WMNT, USDC, FEE_3000);
        _setFeeTier(WMNT, USDT0, FEE_3000);
        _setFeeTier(USDC, USDT0, FEE_100);
        _setFeeTier(USDC, USDe, FEE_500);
        _setFeeTier(USDT0, USDe, FEE_500);
        _setFeeTier(sUSDe, USDT0, FEE_500);
        _setFeeTier(sUSDe, USDC, FEE_500);

        approvedRouters[FUSIONX_V3_ROUTER] = true;
    }

    // ═══════════════════════════════════════════════════════════════
    //                    LIQUIDATION ENTRY
    // ═══════════════════════════════════════════════════════════════

    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit,
        bytes calldata swapData,
        address swapRouter
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, user, minProfit, swapData, swapRouter);
        IPool(AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, user, minProfit, bytes(""), address(0));
        IPool(AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
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
        uint256 balance = IERC20(token).balanceOf(address(this));
        if (balance > 0) {
            bool success = IERC20(token).transfer(OWNER, balance);
            if (!success) revert TransferFailed();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //                 FLASH LOAN CALLBACK
    // ═══════════════════════════════════════════════════════════════

    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        if (msg.sender != AAVE_POOL) revert NotPool();
        if (initiator != address(this)) revert NotSelf();
        _executeLiquidationCallback(asset, amount, premium, params);
        return true;
    }

    function _executeLiquidationCallback(
        address asset,
        uint256 amount,
        uint256 premium,
        bytes calldata params
    ) internal {
        (
            address collateralAsset,
            address user,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(params, (address, address, uint256, bytes, address));

        uint256 totalOwed = amount + premium;

        // 1. Approve debt to Pool for liquidation pull (amount) + flash loan repayment (totalOwed)
        IERC20(asset).approve(AAVE_POOL, amount + totalOwed);

        // 2. Liquidate
        IPool(AAVE_POOL).liquidationCall(collateralAsset, asset, user, amount, false);

        // 3. Swap collateral → debt if needed
        if (collateralAsset != asset) {
            _swapCollateral(collateralAsset, asset, totalOwed, swapData, swapRouter);
        }

        // 4. Verify profit and distribute
        _verifyAndDistributeProfit(asset, totalOwed, minProfit, collateralAsset, user, amount);
    }

    function _swapCollateral(
        address collateralAsset,
        address debtAsset,
        uint256 minAmountOut,
        bytes memory swapData,
        address swapRouter
    ) internal {
        uint256 collateralBalance = IERC20(collateralAsset).balanceOf(address(this));

        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);
            IERC20(collateralAsset).approve(swapRouter, 0);
            IERC20(collateralAsset).approve(swapRouter, collateralBalance);
            (bool success,) = swapRouter.call(swapData);
            if (!success) revert SwapFailed();
            IERC20(collateralAsset).approve(swapRouter, 0);
        } else {
            uint24 fee = _getFeeTier(collateralAsset, debtAsset);
            IERC20(collateralAsset).approve(FUSIONX_V3_ROUTER, collateralBalance);

            ISwapRouter.ExactInputSingleParams memory swapParams = ISwapRouter.ExactInputSingleParams({
                tokenIn: collateralAsset,
                tokenOut: debtAsset,
                fee: fee,
                recipient: address(this),
                deadline: block.timestamp,
                amountIn: collateralBalance,
                amountOutMinimum: minAmountOut,
                sqrtPriceLimitX96: 0
            });

            ISwapRouter(FUSIONX_V3_ROUTER).exactInputSingle(swapParams);
        }
    }

    function _verifyAndDistributeProfit(
        address asset,
        uint256 totalOwed,
        uint256 minProfit,
        address collateralAsset,
        address user,
        uint256 debtCovered
    ) internal {
        uint256 currentBalance = IERC20(asset).balanceOf(address(this));
        if (currentBalance < totalOwed) revert InsufficientProfit(0, minProfit);
        uint256 profit = currentBalance - totalOwed;
        if (profit < minProfit) revert InsufficientProfit(profit, minProfit);

        if (profit > 0) {
            bool success = IERC20(asset).transfer(OWNER, profit);
            if (!success) revert TransferFailed();
        }

        emit LiquidationExecuted(collateralAsset, asset, user, debtCovered, profit);
    }

    // ═══════════════════════════════════════════════════════════════
    //                       INTERNAL
    // ═══════════════════════════════════════════════════════════════

    function _setFeeTier(address tokenA, address tokenB, uint24 fee) internal {
        feeTiers[tokenA][tokenB] = fee;
        feeTiers[tokenB][tokenA] = fee;
    }

    function _getFeeTier(address tokenA, address tokenB) internal view returns (uint24) {
        uint24 fee = feeTiers[tokenA][tokenB];
        return fee == 0 ? FEE_3000 : fee;
    }

    receive() external payable {}
}
