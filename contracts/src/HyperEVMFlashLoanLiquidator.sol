// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

/// @title HyperEVMFlashLoanLiquidator
/// @notice HyperLend (Aave V3 fork) liquidations on HyperEVM via flash loans + HyperSwap V3
contract HyperEVMFlashLoanLiquidator {
    // ═══════════════════════════════════════════════════════════════
    //                    HYPEREVM ADDRESSES
    // ═══════════════════════════════════════════════════════════════
    address internal constant AAVE_POOL = 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b;
    // HyperSwap V3 Router
    address internal constant HYPERSWAP_ROUTER = 0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D;

    // Tokens (verified via Routescan API)
    address internal constant WHYPE  = 0x5555555555555555555555555555555555555555;
    address internal constant UBTC   = 0x9FDBdA0A5e284c32744D2f17Ee5c74B284993463;
    address internal constant UETH   = 0xBe6727B535545C67d5cAa73dEa54865B92CF7907;
    address internal constant USDe   = 0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34;
    address internal constant sUSDe  = 0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2;
    address internal constant USDT0  = 0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb;
    address internal constant USDC   = 0xb88339CB7199b77E23DB6E890353E22632Ba630f;
    address internal constant USDHL  = 0xb50A96253aBDF803D85efcDce07Ad8becBc52BD5;

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

        // HyperSwap V3 fee tiers
        _setFeeTier(WHYPE, USDe, FEE_3000);
        _setFeeTier(WHYPE, sUSDe, FEE_3000);
        _setFeeTier(WHYPE, USDC, FEE_3000);
        _setFeeTier(WHYPE, USDT0, FEE_3000);
        _setFeeTier(WHYPE, UBTC, FEE_3000);
        _setFeeTier(WHYPE, UETH, FEE_3000);
        _setFeeTier(USDC, USDT0, FEE_100);
        _setFeeTier(USDC, USDe, FEE_500);
        _setFeeTier(USDC, USDHL, FEE_100);
        _setFeeTier(sUSDe, USDC, FEE_500);
        _setFeeTier(sUSDe, USDT0, FEE_500);

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
        // Approve for both: liquidationCall pulls `amount` + flash loan repayment pulls `totalOwed`
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
