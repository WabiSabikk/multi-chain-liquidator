// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";
import {IBalancerVault, IFlashLoanRecipient} from "./interfaces/IBalancerVault.sol";
import {Constants} from "./libraries/Constants.sol";

/// @title BalancerFlashLoanLiquidator
/// @notice Executes Aave V3 liquidations funded by Balancer flash loans (0% fee).
///         Same liquidation + swap logic as FlashLoanLiquidator, but uses Balancer Vault
///         instead of Aave flashLoanSimple() to save the 0.05% flash loan premium.
contract BalancerFlashLoanLiquidator is IFlashLoanRecipient {
    error NotOwner();
    error NotVault();
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
    event ApprovedRouterUpdated(address router, bool approved);

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

        // Initialize common fee tiers
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

        approvedRouters[Constants.UNISWAP_V3_ROUTER] = true;
    }

    // ═══════════════════════════════════════════════════════════════
    //                    EXTERNAL — OWNER
    // ═══════════════════════════════════════════════════════════════

    /// @notice Trigger a Balancer-flash-loan-funded liquidation (0% fee)
    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit,
        bytes calldata swapData,
        address swapRouter
    ) external onlyOwner {
        IERC20[] memory tokens = new IERC20[](1);
        tokens[0] = IERC20(debtAsset);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = debtToCover;

        bytes memory userData = abi.encode(collateralAsset, user, minProfit, swapData, swapRouter);

        IBalancerVault(Constants.BALANCER_VAULT).flashLoan(
            IFlashLoanRecipient(address(this)),
            tokens,
            amounts,
            userData
        );
    }

    /// @notice Backward-compatible overload without swapData
    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit
    ) external onlyOwner {
        IERC20[] memory tokens = new IERC20[](1);
        tokens[0] = IERC20(debtAsset);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = debtToCover;

        bytes memory userData = abi.encode(collateralAsset, user, minProfit, bytes(""), address(0));

        IBalancerVault(Constants.BALANCER_VAULT).flashLoan(
            IFlashLoanRecipient(address(this)),
            tokens,
            amounts,
            userData
        );
    }

    function setFeeTier(address tokenA, address tokenB, uint24 fee) external onlyOwner {
        _setFeeTier(tokenA, tokenB, fee);
    }

    function setApprovedRouter(address router, bool approved) external onlyOwner {
        approvedRouters[router] = approved;
        emit ApprovedRouterUpdated(router, approved);
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
    //                 BALANCER FLASH LOAN CALLBACK
    // ═══════════════════════════════════════════════════════════════

    /// @notice Called by Balancer Vault after transferring flash-loaned tokens
    /// @dev Must repay amounts[i] + feeAmounts[i] for each token by end of call.
    ///      Balancer fee is 0% — feeAmounts will be [0].
    function receiveFlashLoan(
        IERC20[] memory tokens,
        uint256[] memory amounts,
        uint256[] memory feeAmounts,
        bytes memory userData
    ) external override {
        if (msg.sender != Constants.BALANCER_VAULT) revert NotVault();

        address debtAsset = address(tokens[0]);
        uint256 amount = amounts[0];
        uint256 fee = feeAmounts[0]; // Should be 0 for Balancer

        (
            address collateralAsset,
            address user,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(userData, (address, address, uint256, bytes, address));

        uint256 totalOwed = amount + fee;

        // 1. Approve debt asset to Aave Pool for liquidation
        IERC20(debtAsset).approve(Constants.AAVE_POOL, amount);

        // 2. Liquidate on Aave — receive collateral
        IPool(Constants.AAVE_POOL).liquidationCall(collateralAsset, debtAsset, user, amount, false);

        // 3. Swap collateral back to debt asset if needed
        if (collateralAsset != debtAsset) {
            uint256 collateralBalance = IERC20(collateralAsset).balanceOf(address(this));
            _swapCollateral(collateralAsset, debtAsset, collateralBalance, totalOwed, swapData, swapRouter);
        }

        // 4. Verify profit
        uint256 currentBalance = IERC20(debtAsset).balanceOf(address(this));
        if (currentBalance < totalOwed) {
            revert InsufficientProfit(0, minProfit);
        }
        uint256 profit = currentBalance - totalOwed;
        if (profit < minProfit) {
            revert InsufficientProfit(profit, minProfit);
        }

        // 5. Repay Balancer: transfer totalOwed back to Vault
        bool repaySuccess = IERC20(debtAsset).transfer(Constants.BALANCER_VAULT, totalOwed);
        if (!repaySuccess) revert TransferFailed();

        // 6. Send profit to owner
        if (profit > 0) {
            bool profitSuccess = IERC20(debtAsset).transfer(OWNER, profit);
            if (!profitSuccess) revert TransferFailed();
        }

        emit LiquidationExecuted(collateralAsset, debtAsset, user, amount, profit);
    }

    // ═══════════════════════════════════════════════════════════════
    //                       INTERNAL
    // ═══════════════════════════════════════════════════════════════

    function _swapCollateral(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 amountOutMinimum,
        bytes memory swapData,
        address swapRouter
    ) internal returns (uint256) {
        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);

            uint256 balanceBefore = IERC20(tokenOut).balanceOf(address(this));

            IERC20(tokenIn).approve(swapRouter, 0);
            IERC20(tokenIn).approve(swapRouter, amountIn);

            (bool success,) = swapRouter.call(swapData);
            if (!success) revert SwapFailed();

            IERC20(tokenIn).approve(swapRouter, 0);

            uint256 received = IERC20(tokenOut).balanceOf(address(this)) - balanceBefore;
            if (received < amountOutMinimum) revert InsufficientProfit(received, amountOutMinimum);
            return received;
        }

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

    function _setFeeTier(address tokenA, address tokenB, uint24 fee) internal {
        feeTiers[tokenA][tokenB] = fee;
        feeTiers[tokenB][tokenA] = fee;
    }

    function _getFeeTier(address tokenA, address tokenB) internal view returns (uint24) {
        uint24 fee = feeTiers[tokenA][tokenB];
        return fee == 0 ? Constants.FEE_3000 : fee;
    }

    receive() external payable {}
}
