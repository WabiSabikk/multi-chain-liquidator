// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

/// @title HypurrFiFlashLoanLiquidator
/// @notice HypurrFi Pool (Aave V3 fork) liquidations on HyperEVM via flash loans
/// @dev Same logic as HyperEVMFlashLoanLiquidator but with HypurrFi Pool address
contract HypurrFiFlashLoanLiquidator {
    // ═══════════════════════════════════════════════════════════════
    //                    HYPURRFI POOL ADDRESS
    // ═══════════════════════════════════════════════════════════════
    address internal constant AAVE_POOL = 0xceCcE0EB9DD2Ef7996e01e25DD70e461F918A14b;
    // HyperSwap V3 Router
    address internal constant HYPERSWAP_ROUTER = 0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D;

    // Fee tiers
    uint24 internal constant FEE_100  = 100;
    uint24 internal constant FEE_500  = 500;
    uint24 internal constant FEE_3000 = 3_000;
    uint24 internal constant FEE_10000 = 10_000;

    error NotOwner();
    error NotPool();
    error NotSelf();
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
    }

    function executeLiquidation(
        address collateralAsset, address debtAsset, address user,
        uint256 debtToCover, uint256 minProfit, bytes calldata swapData, address swapRouter
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, user, minProfit, swapData, swapRouter);
        IPool(AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    function executeLiquidation(
        address collateralAsset, address debtAsset, address user,
        uint256 debtToCover, uint256 minProfit
    ) external onlyOwner {
        bytes memory params = abi.encode(collateralAsset, user, minProfit, bytes(""), address(0));
        IPool(AAVE_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    function setFeeTier(address tokenA, address tokenB, uint24 fee) external onlyOwner { _setFeeTier(tokenA, tokenB, fee); }
    function setApprovedRouter(address router, bool approved) external onlyOwner { approvedRouters[router] = approved; }
    function setAuthorizedCaller(address caller, bool authorized) external { if (msg.sender != OWNER) revert NotOwner(); authorizedCallers[caller] = authorized; }
    function rescueTokens(address token) external onlyOwner { uint256 b = IERC20(token).balanceOf(address(this)); if (b > 0) { if (!IERC20(token).transfer(OWNER, b)) revert TransferFailed(); } }

    function executeOperation(address asset, uint256 amount, uint256 premium, address initiator, bytes calldata params) external returns (bool) {
        if (msg.sender != AAVE_POOL) revert NotPool();
        if (initiator != address(this)) revert NotSelf();
        _executeLiquidationCallback(asset, amount, premium, params);
        return true;
    }

    function _executeLiquidationCallback(address asset, uint256 amount, uint256 premium, bytes calldata params) internal {
        (address collateralAsset, address user, uint256 minProfit, bytes memory swapData, address swapRouter) = abi.decode(params, (address, address, uint256, bytes, address));
        uint256 totalOwed = amount + premium;
        IERC20(asset).approve(AAVE_POOL, amount + totalOwed);
        IPool(AAVE_POOL).liquidationCall(collateralAsset, asset, user, amount, false);
        if (collateralAsset != asset) { _swapCollateral(collateralAsset, asset, totalOwed, swapData, swapRouter); }
        _verifyAndDistributeProfit(asset, totalOwed, minProfit, collateralAsset, user, amount);
    }

    function _swapCollateral(address collateralAsset, address debtAsset, uint256 minAmountOut, bytes memory swapData, address swapRouter) internal {
        uint256 bal = IERC20(collateralAsset).balanceOf(address(this));
        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);
            IERC20(collateralAsset).approve(swapRouter, 0);
            IERC20(collateralAsset).approve(swapRouter, bal);
            (bool ok,) = swapRouter.call(swapData);
            if (!ok) revert SwapFailed();
            IERC20(collateralAsset).approve(swapRouter, 0);
        } else {
            uint24 fee = _getFeeTier(collateralAsset, debtAsset);
            IERC20(collateralAsset).approve(HYPERSWAP_ROUTER, bal);
            ISwapRouter(HYPERSWAP_ROUTER).exactInputSingle(ISwapRouter.ExactInputSingleParams({
                tokenIn: collateralAsset, tokenOut: debtAsset, fee: fee,
                recipient: address(this), deadline: block.timestamp,
                amountIn: bal, amountOutMinimum: minAmountOut, sqrtPriceLimitX96: 0
            }));
        }
    }

    function _verifyAndDistributeProfit(address asset, uint256 totalOwed, uint256 minProfit, address collateralAsset, address user, uint256 debtCovered) internal {
        uint256 bal = IERC20(asset).balanceOf(address(this));
        if (bal < totalOwed) revert InsufficientProfit(0, minProfit);
        uint256 profit = bal - totalOwed;
        if (profit < minProfit) revert InsufficientProfit(profit, minProfit);
        if (profit > 0) { if (!IERC20(asset).transfer(OWNER, profit)) revert TransferFailed(); }
        emit LiquidationExecuted(collateralAsset, asset, user, debtCovered, profit);
    }

    function _setFeeTier(address a, address b, uint24 fee) internal { feeTiers[a][b] = fee; feeTiers[b][a] = fee; }
    function _getFeeTier(address a, address b) internal view returns (uint24) { uint24 f = feeTiers[a][b]; return f == 0 ? FEE_3000 : f; }

    receive() external payable {}
}
