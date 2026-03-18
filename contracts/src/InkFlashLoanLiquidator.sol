// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISlipstreamRouter} from "./interfaces/ISlipstreamRouter.sol";

/// @title InkFlashLoanLiquidator
/// @notice Aave V3 (Tydro) liquidations on Ink chain via flash loans + Velodrome Slipstream swaps
contract InkFlashLoanLiquidator {
    // ═══════════════════════════════════════════════════════════════
    //                      INK ADDRESSES
    // ═══════════════════════════════════════════════════════════════
    address internal constant AAVE_POOL = 0x2816cf15F6d2A220E789aA011D5EE4eB6c47FEbA;
    // Velodrome Slipstream SwapRouter on Ink
    address internal constant VELODROME_ROUTER = 0x63951637d667f23D5251DEdc0f9123D22d8595be;

    // Tokens (checksummed)
    address internal constant WETH   = 0x4200000000000000000000000000000000000006;
    address internal constant USDC   = 0x2D270e6886d130D724215A266106e6832161EAEd;
    address internal constant USDT0  = 0x0200C29006150606B650577BBE7B6248F58470c1;
    address internal constant USDe   = 0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34;
    address internal constant sUSDe  = 0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2;
    address internal constant kBTC   = 0x73E0C0d45E048D25Fc26Fa3159b0aA04BfA4Db98;
    address internal constant weETH  = 0xA3D68b74bF0528fdD07263c60d6488749044914b;
    address internal constant wrsETH = 0x9f0a74A92287E323Eb95c1cd9eCdBEb0e397cAe4;
    address internal constant ezETH  = 0x2416092f143378750bb29b79eD961ab195CcEea5;
    address internal constant GHO    = 0xfc421aD3C883Bf9E7C4f42dE845C4e4405799e73;

    // Velodrome Slipstream tick spacings (NOT Uniswap V3 fee tiers!)
    // ts=1 ≈ stable pairs, ts=50 ≈ 10bps, ts=100 ≈ 30bps, ts=200 ≈ 100bps
    int24 internal constant TS_1   = 1;
    int24 internal constant TS_50  = 50;
    int24 internal constant TS_100 = 100;
    int24 internal constant TS_200 = 200;

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
    mapping(address => mapping(address => int24)) public tickSpacings;
    mapping(address => bool) public approvedRouters;
    mapping(address => bool) public authorizedCallers;

    modifier onlyOwner() {
        if (msg.sender != OWNER && !authorizedCallers[msg.sender]) revert NotOwner();
        _;
    }

    constructor() {
        OWNER = msg.sender;

        // Velodrome Slipstream tick spacings (verified on-chain via CLFactory.getPool)
        _setTickSpacing(WETH, USDC, TS_100);
        _setTickSpacing(WETH, USDT0, TS_100);
        _setTickSpacing(WETH, kBTC, TS_100);
        _setTickSpacing(WETH, weETH, TS_1);
        _setTickSpacing(WETH, wrsETH, TS_1);
        _setTickSpacing(WETH, ezETH, TS_1);
        _setTickSpacing(USDC, USDT0, TS_1);
        _setTickSpacing(USDC, GHO, TS_1);

        approvedRouters[VELODROME_ROUTER] = true;
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

    function setTickSpacing(address tokenA, address tokenB, int24 ts) external onlyOwner { _setTickSpacing(tokenA, tokenB, ts); }
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
            int24 ts = _getTickSpacing(collateralAsset, debtAsset);
            IERC20(collateralAsset).approve(VELODROME_ROUTER, bal);
            ISlipstreamRouter(VELODROME_ROUTER).exactInputSingle(ISlipstreamRouter.ExactInputSingleParams({
                tokenIn: collateralAsset, tokenOut: debtAsset, tickSpacing: ts,
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

    function _setTickSpacing(address a, address b, int24 ts) internal { tickSpacings[a][b] = ts; tickSpacings[b][a] = ts; }
    function _getTickSpacing(address a, address b) internal view returns (int24) { int24 ts = tickSpacings[a][b]; return ts == 0 ? TS_100 : ts; }

    receive() external payable {}
}
