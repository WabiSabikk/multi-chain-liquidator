// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "../interfaces/IERC20.sol";
import {IPool} from "../interfaces/IPool.sol";
import {ISwapRouter} from "../interfaces/ISwapRouter.sol";
import {IComet} from "../interfaces/IComet.sol";

/// @title CompoundV3Liquidator
/// @notice Executes Compound V3 (Comet) liquidations funded by Aave V3 flash loans
/// @dev Compound V3 liquidation is a two-step process:
///      1. absorb() — protocol seizes underwater positions (no capital needed)
///      2. buyCollateral() — purchase absorbed collateral at 2.5-5% discount (needs base token)
///
///      Since Compound V3 has no native flash loans, we use Aave V3 flash loans to fund
///      the buyCollateral step, then swap the received collateral back to base token via
///      an approved DEX router (1inch, Uniswap V3, etc.) to repay the flash loan and capture profit.
contract CompoundV3Liquidator {
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
    event CompoundLiquidation(
        address indexed comet,
        address indexed collateralAsset,
        uint256 baseSpent,
        uint256 collateralReceived
    );
    event AbsorptionExecuted(address indexed comet, uint256 accountCount);
    event FeeTierUpdated(address tokenA, address tokenB, uint24 fee);
    event TokensRescued(address token, uint256 amount);
    event ApprovedRouterUpdated(address router, bool approved);

    // ═══════════════════════════════════════════════════════════════
    //                          STATE
    // ═══════════════════════════════════════════════════════════════
    address public immutable OWNER;
    address public immutable AAVE_POOL;
    address public immutable UNISWAP_V3_ROUTER;

    /// @notice Uniswap V3 fee tier for a given token pair (order-independent)
    mapping(address => mapping(address => uint24)) public feeTiers;

    /// @notice Whitelisted DEX routers for arbitrary calldata swaps
    mapping(address => bool) public approvedRouters;

    /// @notice Authorized callers (multi-wallet support)
    mapping(address => bool) public authorizedCallers;

    /// @notice Default fee tier when no explicit mapping exists
    uint24 private constant DEFAULT_FEE = 3_000; // 0.3%

    // ═══════════════════════════════════════════════════════════════
    //                        MODIFIERS
    // ═══════════════════════════════════════════════════════════════
    modifier onlyOwner() {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
        _;
    }

    // ═══════════════════════════════════════════════════════════════
    //                       CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════

    /// @notice Deploy the Compound V3 liquidator
    /// @param aavePool The Aave V3 Pool address (for flash loans)
    /// @param uniswapRouter The Uniswap V3 SwapRouter address
    constructor(address aavePool, address uniswapRouter) {
        OWNER = msg.sender;
        AAVE_POOL = aavePool;
        UNISWAP_V3_ROUTER = uniswapRouter;

        // Pre-approve Uniswap V3 router
        approvedRouters[uniswapRouter] = true;
    }

    // ═══════════════════════════════════════════════════════════════
    //                    EXTERNAL — OWNER
    // ═══════════════════════════════════════════════════════════════

    /// @notice Execute a full Compound V3 liquidation with multi-DEX routing
    /// @param comet The Comet market address (e.g., cUSDCv3)
    /// @param accounts Underwater accounts to absorb
    /// @param assets Collateral assets to buy after absorption
    /// @param flashLoanAmount Amount of base token to flash loan for buyCollateral
    /// @param minProfit Minimum profit in base token units; reverts if not met
    /// @param swapData Encoded calldata for DEX router swap (empty = use Uniswap V3 default)
    /// @param swapRouter Address of the DEX router to use (ignored if swapData is empty)
    function liquidateCompound(
        address comet,
        address[] calldata accounts,
        address[] calldata assets,
        uint256 flashLoanAmount,
        uint256 minProfit,
        bytes calldata swapData,
        address swapRouter
    ) external onlyOwner {
        // Step 1: Absorb underwater positions (no capital required)
        IComet(comet).absorb(address(this), accounts);
        emit AbsorptionExecuted(comet, accounts.length);

        // Step 2: Flash loan base token to fund collateral purchases
        address baseToken = IComet(comet).baseToken();
        bytes memory params = abi.encode(comet, assets, minProfit, swapData, swapRouter);
        IPool(AAVE_POOL).flashLoanSimple(address(this), baseToken, flashLoanAmount, params, 0);
    }

    /// @notice Backward-compatible overload without swapData (uses Uniswap V3 default)
    function liquidateCompound(
        address comet,
        address[] calldata accounts,
        address[] calldata assets,
        uint256 flashLoanAmount,
        uint256 minProfit
    ) external onlyOwner {
        IComet(comet).absorb(address(this), accounts);
        emit AbsorptionExecuted(comet, accounts.length);

        address baseToken = IComet(comet).baseToken();
        bytes memory params = abi.encode(comet, assets, minProfit, bytes(""), address(0));
        IPool(AAVE_POOL).flashLoanSimple(address(this), baseToken, flashLoanAmount, params, 0);
    }

    /// @notice Set or update the Uniswap V3 fee tier for a token pair
    function setFeeTier(address tokenA, address tokenB, uint24 fee) external onlyOwner {
        _setFeeTier(tokenA, tokenB, fee);
        emit FeeTierUpdated(tokenA, tokenB, fee);
    }

    /// @notice Add or remove an approved DEX router
    function setApprovedRouter(address router, bool approved) external onlyOwner {
        approvedRouters[router] = approved;
        emit ApprovedRouterUpdated(router, approved);
    }

    /// @notice Authorize or revoke additional caller wallets
    function setAuthorizedCaller(address caller, bool authorized) external {
        if (msg.sender != OWNER) revert NotOwner();
        authorizedCallers[caller] = authorized;
    }

    /// @notice Emergency rescue any ERC-20 tokens stuck in the contract
    function rescueTokens(address token) external onlyOwner {
        uint256 balance = IERC20(token).balanceOf(address(this));
        if (balance > 0) {
            bool success = IERC20(token).transfer(OWNER, balance);
            if (!success) revert TransferFailed();
            emit TokensRescued(token, balance);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //                 FLASH LOAN CALLBACK
    // ═══════════════════════════════════════════════════════════════

    /// @notice Called by Aave V3 Pool after receiving the flash-loaned base token
    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        if (msg.sender != AAVE_POOL) revert NotPool();
        if (initiator != address(this)) revert NotSelf();

        _executeCompoundCallback(asset, amount, premium, params);
        return true;
    }

    /// @dev Internal implementation — separated to avoid stack-too-deep
    function _executeCompoundCallback(
        address asset,
        uint256 amount,
        uint256 premium,
        bytes calldata params
    ) internal {
        (
            address comet,
            address[] memory assets,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(params, (address, address[], uint256, bytes, address));

        uint256 totalOwed = amount + premium;

        // Buy collateral from each absorbed asset and swap back to base token
        _buyAllCollateral(comet, asset, assets, amount, swapData, swapRouter);

        // Verify profit after all swaps
        uint256 currentBalance = IERC20(asset).balanceOf(address(this));
        if (currentBalance < totalOwed) {
            revert InsufficientProfit(0, minProfit);
        }

        uint256 profit = currentBalance - totalOwed;
        if (profit < minProfit) {
            revert InsufficientProfit(profit, minProfit);
        }

        // Approve repayment to Aave Pool
        IERC20(asset).approve(AAVE_POOL, totalOwed);

        // Send profit to owner
        if (profit > 0) {
            bool success = IERC20(asset).transfer(OWNER, profit);
            if (!success) revert TransferFailed();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //                       INTERNAL
    // ═══════════════════════════════════════════════════════════════

    function _buyAllCollateral(
        address comet,
        address baseToken,
        address[] memory assets,
        uint256 totalBase,
        bytes memory swapData,
        address swapRouter
    ) internal {
        uint256 remainingBase = totalBase;

        for (uint256 i = 0; i < assets.length; i++) {
            if (remainingBase == 0) break;

            uint256 spent = _buyAndSwapAsset(
                comet,
                baseToken,
                assets[i],
                remainingBase,
                i == assets.length - 1,
                swapData,
                swapRouter
            );
            remainingBase -= spent;
        }
    }

    function _buyAndSwapAsset(
        address comet,
        address baseToken,
        address collateralAsset,
        uint256 availableBase,
        bool isLast,
        bytes memory swapData,
        address swapRouter
    ) internal returns (uint256 baseSpent) {
        // Check how much collateral is available for purchase
        uint256 reserves = IComet(comet).getCollateralReserves(collateralAsset);
        if (reserves == 0) return 0;

        // Calculate how much base token to spend
        baseSpent = _calcBaseAmount(comet, collateralAsset, availableBase, reserves, isLast);
        if (baseSpent == 0) return 0;

        // Approve base token to Comet
        IERC20(baseToken).approve(comet, baseSpent);

        // Buy collateral at discount (minAmount = 0, we check total profit at the end)
        uint256 balBefore = IERC20(collateralAsset).balanceOf(address(this));
        IComet(comet).buyCollateral(collateralAsset, 0, baseSpent, address(this));
        uint256 received = IERC20(collateralAsset).balanceOf(address(this)) - balBefore;

        // Swap received collateral back to base token (skip if collateral IS base token)
        if (received > 0 && collateralAsset != baseToken) {
            _swapCollateral(collateralAsset, baseToken, received, 0, swapData, swapRouter);
        }

        emit CompoundLiquidation(comet, collateralAsset, baseSpent, received);
    }

    function _calcBaseAmount(
        address comet,
        address collateralAsset,
        uint256 availableBase,
        uint256 reserves,
        bool isLast
    ) internal view returns (uint256) {
        if (isLast) return availableBase;

        uint256 quotedCollateral = IComet(comet).quoteCollateral(collateralAsset, availableBase);

        if (quotedCollateral > reserves) {
            return (availableBase * reserves) / quotedCollateral;
        }

        return availableBase;
    }

    /// @notice Swap collateral token to base token via approved DEX router or Uniswap V3 fallback
    function _swapCollateral(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 amountOutMinimum,
        bytes memory swapData,
        address swapRouter
    ) internal returns (uint256) {
        // If swapData is provided, use the specified router with arbitrary calldata
        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);

            uint256 balanceBefore = IERC20(tokenOut).balanceOf(address(this));

            // Safe approve: reset to 0 first (required for USDT-style tokens)
            IERC20(tokenIn).approve(swapRouter, 0);
            IERC20(tokenIn).approve(swapRouter, amountIn);

            (bool success,) = swapRouter.call(swapData);
            if (!success) revert SwapFailed();

            // Reset residual approval after swap
            IERC20(tokenIn).approve(swapRouter, 0);

            // Verify output meets minimum
            uint256 received = IERC20(tokenOut).balanceOf(address(this)) - balanceBefore;
            if (received < amountOutMinimum) revert InsufficientProfit(received, amountOutMinimum);
            return received;
        }

        // Default: Uniswap V3 exactInputSingle
        uint24 fee = _getFeeTier(tokenIn, tokenOut);

        IERC20(tokenIn).approve(UNISWAP_V3_ROUTER, amountIn);

        ISwapRouter.ExactInputSingleParams memory swapParams = ISwapRouter.ExactInputSingleParams({
            tokenIn: tokenIn,
            tokenOut: tokenOut,
            fee: fee,
            recipient: address(this),
            deadline: block.timestamp,
            amountIn: amountIn,
            amountOutMinimum: amountOutMinimum,
            sqrtPriceLimitX96: 0
        });

        return ISwapRouter(UNISWAP_V3_ROUTER).exactInputSingle(swapParams);
    }

    function _setFeeTier(address tokenA, address tokenB, uint24 fee) internal {
        feeTiers[tokenA][tokenB] = fee;
        feeTiers[tokenB][tokenA] = fee;
    }

    function _getFeeTier(address tokenA, address tokenB) internal view returns (uint24) {
        uint24 fee = feeTiers[tokenA][tokenB];
        return fee == 0 ? DEFAULT_FEE : fee;
    }

    // ═══════════════════════════════════════════════════════════════
    //                       RECEIVE ETH
    // ═══════════════════════════════════════════════════════════════

    receive() external payable {}
}
