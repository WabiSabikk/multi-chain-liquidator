// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ChainConfig
/// @notice Protocol deployment addresses for supported chains
/// @dev Constants for multi-chain deployment of the flash loan liquidator.
///      Each chain section provides Aave V3, Compound V3, Uniswap V3, and core token addresses.
///      When deploying to a new chain, pass the appropriate constants to the constructor.
library ChainConfig {
    // ═══════════════════════════════════════════════════════════════
    //                      ARBITRUM (Chain ID: 42161)
    // ═══════════════════════════════════════════════════════════════

    // Protocols
    address internal constant ARB_AAVE_POOL = 0x794a61358D6845594F94dc1DB02A252b5b4814aD;
    address internal constant ARB_COMPOUND_USDC = 0x9c4ec768c28520B50860ea7a15bd7213a9fF58bf;
    address internal constant ARB_UNISWAP_ROUTER = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

    // Tokens
    address internal constant ARB_WETH = 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1;
    address internal constant ARB_USDC = 0xaf88d065e77c8cC2239327C5EDb3A432268e5831;

    // ═══════════════════════════════════════════════════════════════
    //                      OPTIMISM (Chain ID: 10)
    // ═══════════════════════════════════════════════════════════════

    // Protocols
    address internal constant OP_AAVE_POOL = 0x794a61358D6845594F94dc1DB02A252b5b4814aD;
    address internal constant OP_COMPOUND_USDC = 0xD98BE00b69D00553B70B640326ea183186a12f37;
    address internal constant OP_UNISWAP_ROUTER = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

    // Tokens
    address internal constant OP_WETH = 0x4200000000000000000000000000000000000006;
    address internal constant OP_USDC = 0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85;

    // ═══════════════════════════════════════════════════════════════
    //                      BASE (Chain ID: 8453)
    // ═══════════════════════════════════════════════════════════════

    // Protocols
    address internal constant BASE_AAVE_POOL = 0xA238Dd80C259a72e81d7e4664a9801593F98d1c5;
    address internal constant BASE_COMPOUND_USDC = 0xb125E6687d4313864e53df431d5425969c15Eb2F;
    // SwapRouter02 (no `deadline` in ExactInputSingleParams struct — use ISwapRouter02 or swapData)
    address internal constant BASE_UNISWAP_ROUTER = 0x2626664c2603336E57B271c5C0b26F421741e481;
    // Legacy SwapRouter (compatible with ISwapRouter deadline struct) — not officially deployed on Base
    // address internal constant BASE_UNISWAP_ROUTER_LEGACY = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

    // Tokens
    address internal constant BASE_WETH = 0x4200000000000000000000000000000000000006;
    address internal constant BASE_USDC = 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913;

    // ═══════════════════════════════════════════════════════════════
    //                      POLYGON (Chain ID: 137)
    // ═══════════════════════════════════════════════════════════════

    // Protocols
    address internal constant POLY_AAVE_POOL = 0x794a61358D6845594F94dc1DB02A252b5b4814aD;
    address internal constant POLY_UNISWAP_ROUTER = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

    // Tokens
    address internal constant POLY_WETH = 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619;
    address internal constant POLY_USDC = 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359;
}
