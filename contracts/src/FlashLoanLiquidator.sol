// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";
import {Constants} from "./libraries/Constants.sol";

/// @title FlashLoanLiquidator
/// @notice Executes Aave V3 liquidations funded by flash loans, swapping collateral via Uniswap V3
///         or any approved DEX router (1inch, etc.) using off-chain routing calldata
/// @dev Designed for multi-chain deployment. Does NOT inherit from any Aave interface — just implements
///      the `executeOperation` callback signature expected by the Pool.
contract FlashLoanLiquidator {
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
    event FeeTierUpdated(address tokenA, address tokenB, uint24 fee);
    event TokensRescued(address token, uint256 amount);
    event ApprovedRouterUpdated(address router, bool approved);

    // ═══════════════════════════════════════════════════════════════
    //                          STATE
    // ═══════════════════════════════════════════════════════════════
    address public immutable OWNER;

    /// @notice Uniswap V3 fee tier for a given token pair (order-independent via _pairKey)
    mapping(address => mapping(address => uint24)) public feeTiers;

    /// @notice Whitelisted DEX routers for arbitrary calldata swaps
    mapping(address => bool) public approvedRouters;

    /// @notice Authorized callers (multi-wallet support for parallel nonces)
    mapping(address => bool) public authorizedCallers;

    // ═══════════════════════════════════════════════════════════════
    //                        MODIFIERS
    // ═══════════════════════════════════════════════════════════════
    modifier onlyOwner() {
        _checkOwner();
        _;
    }

    function _checkOwner() internal view {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
    }

    // ═══════════════════════════════════════════════════════════════
    //                       CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════
    constructor() {
        OWNER = msg.sender;

        // Initialize common fee tiers on Arbitrum
        _setFeeTier(Constants.WETH, Constants.USDC, Constants.FEE_500);
        _setFeeTier(Constants.WETH, Constants.USDC_E, Constants.FEE_500);
        _setFeeTier(Constants.WETH, Constants.WBTC, Constants.FEE_500);
        _setFeeTier(Constants.WETH, Constants.USDT, Constants.FEE_500);
        _setFeeTier(Constants.WETH, Constants.DAI, Constants.FEE_3000);
        _setFeeTier(Constants.WETH, Constants.ARB, Constants.FEE_500);
        _setFeeTier(Constants.USDC, Constants.USDT, Constants.FEE_100);
        _setFeeTier(Constants.USDC, Constants.USDC_E, Constants.FEE_100);
        _setFeeTier(Constants.USDC, Constants.DAI, Constants.FEE_100);
        _setFeeTier(Constants.WBTC, Constants.USDC, Constants.FEE_500);
        _setFeeTier(Constants.WBTC, Constants.USDT, Constants.FEE_500);
        _setFeeTier(Constants.ARB, Constants.USDC, Constants.FEE_500);

        // Pre-approve Uniswap V3 router
        approvedRouters[Constants.UNISWAP_V3_ROUTER] = true;
    }

    // ═══════════════════════════════════════════════════════════════
    //                    EXTERNAL — OWNER
    // ═══════════════════════════════════════════════════════════════

    /// @notice Trigger a flash-loan-funded liquidation
    /// @param collateralAsset The collateral asset to seize
    /// @param debtAsset The debt asset to repay (also the flash-loaned asset)
    /// @param user The borrower whose position is unhealthy
    /// @param debtToCover Amount of debt to repay (use type(uint256).max for maximum)
    /// @param minProfit Minimum profit in debtAsset units; reverts if not met
    /// @param swapData Encoded calldata for DEX router swap (empty = use Uniswap V3 default)
    /// @param swapRouter Address of the DEX router to use (ignored if swapData is empty)
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
        IPool(Constants.AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    /// @notice Backward-compatible overload without swapData (uses Uniswap V3 default)
    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, user, minProfit, bytes(""), address(0));
        IPool(Constants.AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    /// @notice Set or update the Uniswap V3 fee tier for a token pair
    function setFeeTier(address tokenA, address tokenB, uint24 fee) external onlyOwner {
        _setFeeTier(tokenA, tokenB, fee);
        emit FeeTierUpdated(tokenA, tokenB, fee);
    }

    /// @notice Add or remove an approved DEX router
    /// @param router The router address to approve/revoke
    /// @param approved True to approve, false to revoke
    function setApprovedRouter(address router, bool approved) external onlyOwner {
        approvedRouters[router] = approved;
        emit ApprovedRouterUpdated(router, approved);
    }

    /// @notice Authorize or revoke additional caller wallets (multi-wallet nonce pool)
    /// @dev Only the original OWNER can manage authorized callers
    /// @param caller The wallet address to authorize/revoke
    /// @param authorized True to authorize, false to revoke
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

    /// @notice Called by Aave Pool after receiving the flash-loaned amount
    /// @dev Must repay amount + premium to the Pool by the end of this call
    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        if (msg.sender != Constants.AAVE_POOL) revert NotPool();
        if (initiator != address(this)) revert NotSelf();

        _executeLiquidationCallback(asset, amount, premium, params);
        return true;
    }

    /// @dev Internal implementation of the flash loan callback — separated to avoid stack-too-deep
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

        // 1. Approve debt asset to Pool for liquidation pull (amount) + flash loan repayment (totalOwed)
        IERC20(asset).approve(Constants.AAVE_POOL, amount + totalOwed);

        // 2. Liquidate — receive collateral (underlying, not aToken)
        IPool(Constants.AAVE_POOL).liquidationCall(collateralAsset, asset, user, amount, false);

        // 3. If collateral != debt asset, swap collateral back to debt asset
        if (collateralAsset != asset) {
            uint256 collateralBalance = IERC20(collateralAsset).balanceOf(address(this));
            _swapCollateral(collateralAsset, asset, collateralBalance, totalOwed, swapData, swapRouter);
        }

        // 4. Calculate and verify profit
        _verifyAndDistributeProfit(asset, totalOwed, minProfit, collateralAsset, user, amount);
    }

    /// @dev Verify profit meets minimum and distribute to owner
    function _verifyAndDistributeProfit(
        address asset,
        uint256 totalOwed,
        uint256 minProfit,
        address collateralAsset,
        address user,
        uint256 debtCovered
    ) internal {
        uint256 currentBalance = IERC20(asset).balanceOf(address(this));
        if (currentBalance < totalOwed) {
            revert InsufficientProfit(0, minProfit);
        }
        uint256 profit = currentBalance - totalOwed;

        if (profit < minProfit) {
            revert InsufficientProfit(profit, minProfit);
        }

        if (profit > 0) {
            bool success = IERC20(asset).transfer(OWNER, profit);
            if (!success) revert TransferFailed();
        }

        emit LiquidationExecuted(collateralAsset, asset, user, debtCovered, profit);
    }

    // ═══════════════════════════════════════════════════════════════
    //                       INTERNAL
    // ═══════════════════════════════════════════════════════════════

    /// @notice Swap collateral token to debt token via approved DEX router or Uniswap V3 fallback
    /// @param tokenIn Collateral token to sell
    /// @param tokenOut Debt token to receive
    /// @param amountIn Amount of collateral to swap
    /// @param amountOutMinimum Minimum output amount
    /// @param swapData Encoded calldata for DEX router (empty = Uniswap V3 default)
    /// @param swapRouter DEX router address (ignored if swapData is empty)
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

        IERC20(tokenIn).approve(Constants.UNISWAP_V3_ROUTER, amountIn);

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

        return ISwapRouter(Constants.UNISWAP_V3_ROUTER).exactInputSingle(swapParams);
    }

    /// @notice Store fee tier symmetrically (A->B and B->A)
    function _setFeeTier(address tokenA, address tokenB, uint24 fee) internal {
        feeTiers[tokenA][tokenB] = fee;
        feeTiers[tokenB][tokenA] = fee;
    }

    /// @notice Get fee tier for a pair, defaulting to 3000 (0.3%)
    function _getFeeTier(address tokenA, address tokenB) internal view returns (uint24) {
        uint24 fee = feeTiers[tokenA][tokenB];
        return fee == 0 ? Constants.FEE_3000 : fee;
    }

    // ═══════════════════════════════════════════════════════════════
    //                       RECEIVE ETH
    // ═══════════════════════════════════════════════════════════════

    /// @notice Accept ETH transfers (e.g. from WETH unwrap)
    receive() external payable {}
}
