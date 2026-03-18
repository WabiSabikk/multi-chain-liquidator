// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IPool} from "./interfaces/IPool.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";

/// @title LendleCrossPoolLiquidator v3
/// @notice Flash loan from Aave V3 on Mantle → liquidate on Lendle (Aave V2 fork) → swap → repay
/// @dev Uses uint8 mode flag to dispatch same-token vs cross-token (fixes params.length bug in v2)
contract LendleCrossPoolLiquidator {
    address internal constant AAVE_V3_POOL = 0x458F293454fE0d67EC0655f3672301301DD51422;
    address internal constant LENDLE_POOL = 0xCFa5aE7c2CE8Fadc6426C1ff872cA45378Fb7cF3;
    address internal constant FUSIONX_ROUTER = 0x5989FB161568b9F133eDf5Cf6787f5597762797F;

    address internal constant WETH  = 0xdEAddEaDdeadDEadDEADDEAddEADDEAddead1111;
    address internal constant WMNT  = 0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8;
    address internal constant USDC  = 0x09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9;
    address internal constant USDT  = 0x201EBa5CC46D216Ce6DC03F6a759e8E766e956aE;
    address internal constant USDT0 = 0x779Ded0c9e1022225f8E0630b35a9b54bE713736;
    address internal constant mETH  = 0xcDA86A272531e8640cD7F1a92c01839911B90bb0;

    uint24 internal constant FEE_100 = 100;
    uint24 internal constant FEE_500 = 500;
    uint24 internal constant FEE_3000 = 3_000;

    uint8 internal constant MODE_SAME_TOKEN = 0;
    uint8 internal constant MODE_CROSS_TOKEN = 1;

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

        _setFeeTier(WETH, USDC, FEE_500);
        _setFeeTier(WETH, WMNT, FEE_3000);
        _setFeeTier(WMNT, USDC, FEE_3000);
        _setFeeTier(mETH, WETH, FEE_500);
        _setFeeTier(USDT, USDC, FEE_100);  // Lendle USDT ↔ USDC

        approvedRouters[FUSIONX_ROUTER] = true;
    }

    /// @notice Same-token: flash loan debtAsset from Aave V3, liquidate on Lendle
    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit
    ) external onlyOwner {
        bytes memory params = abi.encode(MODE_SAME_TOKEN, collateralAsset, user, minProfit, bytes(""), address(0));
        IPool(AAVE_V3_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    /// @notice Same-token with custom swap routing (Odos)
    function executeLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        uint256 minProfit,
        bytes calldata swapData,
        address swapRouter
    ) external onlyOwner {
        bytes memory params = abi.encode(MODE_SAME_TOKEN, collateralAsset, user, minProfit, swapData, swapRouter);
        IPool(AAVE_V3_POOL).flashLoanSimple(address(this), debtAsset, debtToCover, params, 0);
    }

    /// @notice Cross-token: flash loan flashLoanAsset, swap to debtAsset, liquidate, swap back
    function executeCrossTokenLiquidation(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        address flashLoanAsset,
        uint256 flashLoanAmount,
        bytes calldata preSwapData,
        address preSwapRouter,
        bytes calldata postSwapData,
        address postSwapRouter
    ) external onlyOwner {
        bytes memory params = abi.encode(
            MODE_CROSS_TOKEN, collateralAsset, debtAsset, user, debtToCover,
            preSwapData, preSwapRouter, postSwapData, postSwapRouter
        );
        IPool(AAVE_V3_POOL).flashLoanSimple(address(this), flashLoanAsset, flashLoanAmount, params, 0);
    }

    /// @notice Aave V3 flash loan callback
    /// @dev Dispatches by mode flag (first uint8 in params), not by params.length
    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        if (msg.sender != AAVE_V3_POOL) revert NotPool();
        if (initiator != address(this)) revert NotSelf();

        uint256 totalOwed = amount + premium;

        // First 32 bytes = mode flag (uint8 padded to uint256)
        uint8 mode = uint8(uint256(bytes32(params[:32])));

        if (mode == MODE_CROSS_TOKEN) {
            _executeCrossToken(asset, amount, totalOwed, params);
        } else {
            _executeSameToken(asset, amount, totalOwed, params);
        }

        IERC20(asset).approve(AAVE_V3_POOL, totalOwed);

        uint256 balance = IERC20(asset).balanceOf(address(this));
        if (balance < totalOwed) revert InsufficientProfit(0, 0);
        uint256 profit = balance - totalOwed;
        if (profit > 0) {
            if (!IERC20(asset).transfer(OWNER, profit)) revert TransferFailed();
        }

        return true;
    }

    function _executeSameToken(address asset, uint256 amount, uint256 totalOwed, bytes calldata params) internal {
        // Skip mode byte (first 32 bytes)
        (
            , // uint8 mode (already read)
            address collateralAsset,
            address user,
            uint256 minProfit,
            bytes memory swapData,
            address swapRouter
        ) = abi.decode(params, (uint8, address, address, uint256, bytes, address));

        IERC20(asset).approve(LENDLE_POOL, amount);
        IPool(LENDLE_POOL).liquidationCall(collateralAsset, asset, user, amount, false);

        if (collateralAsset != asset) {
            _swapCollateral(collateralAsset, asset, totalOwed, swapData, swapRouter);
        }

        emit LiquidationExecuted(collateralAsset, asset, user, amount, 0);
    }

    function _executeCrossToken(address flashAsset, uint256, uint256, bytes calldata params) internal {
        // Decode in two steps to avoid stack-too-deep
        // Step 1: decode fixed params (skip mode byte at position 0)
        address collateralAsset;
        address debtAsset;
        address user;
        uint256 debtToCover;
        {
            (, collateralAsset, debtAsset, user, debtToCover,,,,) =
                abi.decode(params, (uint8, address, address, address, uint256, bytes, address, bytes, address));
        }

        // Step 2: decode dynamic params
        bytes memory preSwapData;
        address preSwapRouter;
        bytes memory postSwapData;
        address postSwapRouter;
        {
            (,,,,, preSwapData, preSwapRouter, postSwapData, postSwapRouter) =
                abi.decode(params, (uint8, address, address, address, uint256, bytes, address, bytes, address));
        }

        // 1. Swap flashAsset → debtAsset
        _swapCollateral(flashAsset, debtAsset, debtToCover, preSwapData, preSwapRouter);

        // 2. Liquidate on Lendle
        IERC20(debtAsset).approve(LENDLE_POOL, debtToCover);
        IPool(LENDLE_POOL).liquidationCall(collateralAsset, debtAsset, user, debtToCover, false);

        // 3. Swap collateral → flashAsset
        {
            uint256 colBalance = IERC20(collateralAsset).balanceOf(address(this));
            if (colBalance > 0 && collateralAsset != flashAsset) {
                _swapCollateral(collateralAsset, flashAsset, 0, postSwapData, postSwapRouter);
            }
        }

        // 4. Swap any debtAsset remainder → flashAsset
        {
            uint256 debtRemainder = IERC20(debtAsset).balanceOf(address(this));
            if (debtRemainder > 0 && debtAsset != flashAsset) {
                _swapCollateral(debtAsset, flashAsset, 0, bytes(""), address(0));
            }
        }

        emit LiquidationExecuted(collateralAsset, debtAsset, user, debtToCover, 0);
    }

    function _swapCollateral(
        address tokenIn,
        address tokenOut,
        uint256 minAmountOut,
        bytes memory swapData,
        address swapRouter
    ) internal {
        uint256 bal = IERC20(tokenIn).balanceOf(address(this));
        if (swapData.length > 0) {
            if (!approvedRouters[swapRouter]) revert RouterNotApproved(swapRouter);
            IERC20(tokenIn).approve(swapRouter, 0);
            IERC20(tokenIn).approve(swapRouter, bal);
            (bool ok,) = swapRouter.call(swapData);
            if (!ok) revert SwapFailed();
            IERC20(tokenIn).approve(swapRouter, 0);
        } else {
            uint24 fee = _getFeeTier(tokenIn, tokenOut);
            IERC20(tokenIn).approve(FUSIONX_ROUTER, bal);
            ISwapRouter(FUSIONX_ROUTER).exactInputSingle(ISwapRouter.ExactInputSingleParams({
                tokenIn: tokenIn,
                tokenOut: tokenOut,
                fee: fee,
                recipient: address(this),
                deadline: block.timestamp,
                amountIn: bal,
                amountOutMinimum: minAmountOut,
                sqrtPriceLimitX96: 0
            }));
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
