// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Constants
/// @notice Arbitrum mainnet addresses and Uniswap V3 fee tiers
library Constants {
    // ═══════════════════════════════════════════════════════════════
    //                      AAVE V3 (Arbitrum)
    // ═══════════════════════════════════════════════════════════════
    address internal constant AAVE_POOL = 0x794a61358D6845594F94dc1DB02A252b5b4814aD;
    address internal constant POOL_DATA_PROVIDER = 0x69FA688f1Dc47d4B5d8029D5a35FB7a548310654;
    address internal constant AAVE_ORACLE = 0xb56c2F0B653B2e0b10C9b928C8580Ac5Df02C7C7;
    address internal constant POOL_ADDRESSES_PROVIDER = 0xa97684ead0e402dC232d5A977953DF7ECBaB3CDb;

    // ═══════════════════════════════════════════════════════════════
    //                      BALANCER (Arbitrum)
    // ═══════════════════════════════════════════════════════════════
    address internal constant BALANCER_VAULT = 0xBA12222222228d8Ba445958a75a0704d566BF2C8;

    // ═══════════════════════════════════════════════════════════════
    //                      UNISWAP V3 (Arbitrum)
    // ═══════════════════════════════════════════════════════════════
    address internal constant UNISWAP_V3_ROUTER = 0xE592427A0AEce92De3Edee1F18E0157C05861564;

    // ═══════════════════════════════════════════════════════════════
    //                      TOKENS (Arbitrum)
    // ═══════════════════════════════════════════════════════════════
    address internal constant WETH = 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1;
    address internal constant USDC = 0xaf88d065e77c8cC2239327C5EDb3A432268e5831; // native USDC
    address internal constant USDC_E = 0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8; // bridged USDC.e
    address internal constant WBTC = 0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f;
    address internal constant USDT = 0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9;
    address internal constant DAI = 0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1;
    address internal constant ARB = 0x912CE59144191C1204E64559FE8253a0e49E6548;

    // ═══════════════════════════════════════════════════════════════
    //                      UNISWAP V3 FEE TIERS
    // ═══════════════════════════════════════════════════════════════
    uint24 internal constant FEE_100 = 100; // 0.01%
    uint24 internal constant FEE_500 = 500; // 0.05%
    uint24 internal constant FEE_3000 = 3_000; // 0.3%
    uint24 internal constant FEE_10000 = 10_000; // 1%
}
