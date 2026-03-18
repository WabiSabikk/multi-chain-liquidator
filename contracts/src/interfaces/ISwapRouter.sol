// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface ISwapRouter {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 deadline;
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
        uint256 deadline;
        uint256 amountIn;
        uint256 amountOutMinimum;
    }

    /// @notice Swaps amountIn of tokenIn for as much as possible of tokenOut through a multi-hop path
    /// @param params The parameters for the multi-hop swap
    /// @return amountOut The amount of the final tokenOut received
    function exactInput(ExactInputParams calldata params) external payable returns (uint256 amountOut);
}
