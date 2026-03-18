// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ISwapRouter02
/// @notice Uniswap V3 SwapRouter02 interface (used on Base, no `deadline` in struct)
/// @dev SwapRouter02 wraps deadline via `checkDeadline` modifier at the router level.
///      The ExactInputSingleParams struct differs from the legacy SwapRouter by omitting `deadline`.
interface ISwapRouter02 {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 sqrtPriceLimitX96;
    }

    /// @notice Swaps amountIn of tokenIn for as much as possible of tokenOut
    /// @param params The parameters for the swap
    /// @return amountOut The amount of tokenOut received
    function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut);

    struct ExactInputParams {
        bytes path;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
    }

    /// @notice Swaps amountIn of tokenIn for as much as possible of tokenOut through a multi-hop path
    /// @param params The parameters for the multi-hop swap
    /// @return amountOut The amount of the final tokenOut received
    function exactInput(ExactInputParams calldata params) external payable returns (uint256 amountOut);
}
