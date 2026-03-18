# Contract & DEX Liquidity Analysis — HyperEVM

**Date:** 2026-03-10
**Status:** READ-ONLY analysis, no code changes

---

## Part 1: Our Contract Analysis

### HyperEVMFlashLoanLiquidator (`0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c`)

**Source:** `contracts/src/HyperEVMFlashLoanLiquidator.sol` (150 lines)

#### Capabilities

- Flash loan via HyperLend Pool (`flashLoanSimple`) — 4bps (0.04%) premium
- Single-hop swap via HyperSwap V3 Router (`exactInputSingle`)
- Custom swap data bypass (`swapData` + `swapRouter` parameters)
- Configurable fee tiers per token pair (`setFeeTier`)
- Approved router whitelist (`approvedRouters`)
- Profit verification and auto-transfer to owner
- Token rescue function

#### Limitations (CRITICAL)

1. **SINGLE-HOP ONLY (default path)**
   - `_swapCollateral()` at line 129 calls `ISwapRouter.exactInputSingle()`
   - The `ISwapRouter` interface DOES define `exactInput()` (multi-hop) at line 22-32 of `interfaces/ISwapRouter.sol`
   - But the contract NEVER calls `exactInput()` — only `exactInputSingle()`
   - For pairs without a direct pool (e.g., kHYPE -> UBTC), liquidation FAILS

2. **WRONG DEX FACTORY**
   - Contract hardcodes HyperSwap V3 Router: `0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D`
   - HyperSwap factory: `0xB1c0fa0B789320044A6F623cFe5eBda9562602E3`
   - There is a SECOND factory (`0xFf7B3e8C00e57ea31477c32A5B52a58Eea47b072`, verified as `UniswapV3Factory`) with **10-20x MORE liquidity** for key pairs
   - Competitor `0xdd8692bc` uses the alt factory pools exclusively
   - Our contract does NOT have the alt factory's router (`0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B`) approved

3. **AAVE FLASH LOAN PREMIUM (4bps)**
   - Uses `flashLoanSimple()` on HyperLend Pool — costs 0.04% of borrowed amount
   - Competitor uses **Uniswap V3 flash swap** (pool.swap()) — costs only the swap fee (1bp on fee=100 pools)
   - On a 25 HYPE liquidation: our premium = 0.01 HYPE ($0.30) vs competitor's swap fee = 0.0025 kHYPE ($0.08)

4. **WRONG FEE TIERS**
   - Constructor sets wHYPE/kHYPE fee to FEE_3000 (line 68, via default fallback at line 147)
   - But the only liquid kHYPE/wHYPE pool is at fee=100
   - Fee tier 3000 for kHYPE/wHYPE has ZERO liquidity on HyperSwap factory
   - This means: even if a direct pool existed, our contract would route to the wrong one

5. **NO wHYPE/USDC DIRECT POOL ON HYPERSWAP FACTORY**
   - The HyperSwap factory (`0xB1c0fa...`) has ZERO pools for wHYPE/USDC at ANY fee tier
   - This means our contract CANNOT swap wHYPE to USDC via its default router
   - The alt factory has wHYPE/USDC at fee=500 (liq=1.66e19)

6. **swapData WORKAROUND EXISTS BUT IS SUBOPTIMAL**
   - The `swapData` parameter (lines 119-124) allows passing arbitrary calldata to an approved router
   - This COULD be used for multi-hop by manually encoding `exactInput()` calldata
   - However, our Rust executor currently does NOT encode multi-hop calldata
   - And the alt factory router is NOT in the `approvedRouters` mapping

### MorphoLiquidator (`0xD47223b7a191643ecdddC4715C948D88D5a13Bdd`)

**Source:** `contracts/src/MorphoLiquidator.sol` (231 lines)

Same limitations as above:
- `exactInputSingle` only (line 187)
- Same HyperSwap Router hardcoded
- Same wrong default fee tiers
- Same missing alt factory router

---

## Part 2: DEX Pool Existence Matrix

### Two Factories on HyperEVM

| Factory | Address | Router | Verified |
|---------|---------|--------|----------|
| **HyperSwap V3** | `0xB1c0fa0B789320044A6F623cFe5eBda9562602E3` | `0x4E2960a8cd19B467b82d26D83fAcb0fAE26b094D` | Unknown |
| **Alt Factory (UniswapV3Factory)** | `0xFf7B3e8C00e57ea31477c32A5B52a58Eea47b072` | `0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B` (SwapRouter) | Yes |

The alt factory consistently has 10-100x more liquidity. Competitor `0xdd8692bc` routes exclusively through alt factory pools.

### Pool Matrix — HyperSwap Factory (`0xB1c0fa...`)

| Pair | Fee 100 | Fee 500 | Fee 3000 | Fee 10000 | Best |
|------|---------|---------|----------|-----------|------|
| wHYPE/USDC | - | - | - | - | **NO POOL** |
| wHYPE/USDT0 | - | - | - | - | **NO POOL** |
| wHYPE/UBTC | - | - | - | 1.81e14 | fee=10000, low liq |
| wHYPE/UETH | 1.15e14 | 3.70e18 | **2.65e21** | 2.21e17 | fee=3000 |
| wHYPE/kHYPE | **3.86e24** | 0 | 0 | 0 | fee=100 |
| wHYPE/wstHYPE | **1.62e20** | 9.48e17 | 3.80e18 | 1.34e17 | fee=100 |
| wHYPE/beHYPE | **3.45e19** | - | - | - | fee=100 |
| wHYPE/USDe | - | 1.85e17 | **3.30e20** | 5.65e16 | fee=3000 |
| wHYPE/sUSDe | - | - | - | - | **NO POOL** |
| wHYPE/USOL | - | 0 | 9.88e14 | 0 | fee=3000, very low |
| kHYPE/UBTC | - | 0 | 6.08e11 | 0 | fee=3000, dust |
| kHYPE/USDC | - | 0 | 2.16e15 | 0 | fee=3000, low |
| kHYPE/wstHYPE | 0 | 0 | 0 | - | **ALL EMPTY** |
| kHYPE/UETH | - | - | 0 | - | **NO LIQ** |
| USDC/USDT0 | **9.53e12** | 0 | - | - | fee=100 |
| USDC/USDe | - | - | - | - | **NO POOL** |
| USDe/USDT0 | exists (0 liq?) | - | - | - | unclear |
| UBTC/USDC | - | - | 0 | - | **NO LIQ** |
| UETH/USDC | - | - | - | - | **NO POOL** |
| sUSDe/USDe | - | 0 | - | - | **NO LIQ** |
| sUSDe/USDC | - | - | - | - | **NO POOL** |

### Pool Matrix — Alt Factory (`0xFf7B3e8C...`) — MUCH HIGHER LIQUIDITY

| Pair | Fee 100 | Fee 500 | Fee 3000 | Fee 10000 | Best |
|------|---------|---------|----------|-----------|------|
| wHYPE/USDC | 0 | **1.66e19** | 2.71e17 | 1.33e10 | fee=500 |
| wHYPE/USDT0 | 3.62e12 | **1.58e18** | 2.60e16 | 7.03e11 | fee=500 |
| wHYPE/UBTC | - | **1.23e13** | 1.73e17 | - | fee=3000 |
| wHYPE/UETH | - | 0 | **3.61e22** | - | fee=3000 |
| wHYPE/kHYPE | **7.61e25** | 2.36e18 | 4.00e21 | 0 | fee=100, 19.7x more than HyperSwap |
| wHYPE/wstHYPE | **3.56e23** | 0 | 8.04e17 | 0 | fee=100, 2196x more |
| wHYPE/beHYPE | **8.28e21** | 0 | 0 | - | fee=100, 240x more |
| kHYPE/UBTC | 0 | 0 | **4.80e15** | 2.03e13 | fee=3000 |
| kHYPE/USDC | - | 0 | **7.73e15** | 0 | fee=3000 |
| kHYPE/wstHYPE | 0 | - | **5.10e23** | - | fee=3000, MASSIVE |
| kHYPE/UETH | - | 0 | **2.17e21** | - | fee=3000 |
| USDC/USDT0 | **7.65e14** | - | 5.14e6 | - | fee=100 |
| USDC/USDe | **4.41e18** | - | - | - | fee=100 |
| USDe/USDT0 | **1.56e19** | 0 | - | - | fee=100 |
| USOL/wHYPE | - | 0 | 1.17e18 | - | fee=3000 |

### Liquidity Comparison Summary

For the most common liquidation pairs, the alt factory dominates:

| Pair | HyperSwap Liq | Alt Factory Liq | Ratio |
|------|--------------|----------------|-------|
| kHYPE/wHYPE (fee=100) | 3.86e24 | **7.61e25** | **19.7x** |
| wHYPE/wstHYPE (fee=100) | 1.62e20 | **3.56e23** | **2,196x** |
| wHYPE/beHYPE (fee=100) | 3.45e19 | **8.28e21** | **240x** |
| wHYPE/UETH (fee=3000) | 2.65e21 | **3.61e22** | **13.6x** |
| wHYPE/USDC | **NO POOL** | 1.66e19 (fee=500) | **infinity** |
| wHYPE/USDT0 | **NO POOL** | 1.58e18 (fee=500) | **infinity** |
| kHYPE/wstHYPE | **ALL EMPTY** | 5.10e23 (fee=3000) | **infinity** |

---

## Part 3: Pairs Requiring Multi-Hop Routing

### Pairs with NO direct pool on either factory:

| Collateral | Debt | Required Route |
|------------|------|----------------|
| sUSDe | ANY | sUSDe has **ZERO liquid pools** on both factories. No swap possible via V3 |
| USOL | USDC | USOL -> wHYPE (fee=3000, alt) -> USDC (fee=500, alt) |
| UETH | USDC | UETH -> wHYPE (fee=3000, alt) -> USDC (fee=500, alt) |
| UBTC | USDC | UBTC -> wHYPE (fee=3000, alt) -> USDC (fee=500, alt) |
| beHYPE | USDC | beHYPE -> wHYPE (fee=100, alt) -> USDC (fee=500, alt) |
| wstHYPE | USDC | wstHYPE -> wHYPE (fee=100, alt) -> USDC (fee=500, alt) |
| kHYPE | UBTC | kHYPE -> wHYPE (fee=100, alt) -> UBTC (fee=3000, alt) |

### Pairs with a direct pool on alt factory but NOT on HyperSwap:

| Collateral | Debt | Alt Factory Pool | Fee | Liquidity |
|------------|------|-----------------|-----|-----------|
| wHYPE | USDC | `0x6c9A33E3b...` | 500 | 1.66e19 |
| wHYPE | USDT0 | `0xBd19E19E4...` | 500 | 1.58e18 |
| kHYPE | wstHYPE | `0x1a4623841...` | 3000 | 5.10e23 |

---

## Part 4: Competitor Contract Analysis

### Competitor `0xdd8692Bc25972DBa5906201960e2dbe783D460Fa`

**EOA:** `0xac5b4731229265627Cd3Fce62BB826493B38a66f`
**Contract verified:** No
**Bytecode size:** 16,185 hex chars (~8KB compiled)
**Method selector:** `0xa17922e2`

#### TX Analysis: Block 29134404 (kHYPE/wHYPE liquidation, $57.61 net profit)

**Token flow (decoded from logs):**
```
1. Flash borrow 25.009 wHYPE from kHYPE/wHYPE pool (0xbe352d, ALT factory fee=100)
   Source: NOT Aave flashLoanSimple — uses Uniswap V3 flash swap (pool.swap())

2. Contract 0xdd8692 receives wHYPE, approves HyperLend Pool

3. liquidationCall on HyperLend Pool:
   - Collateral: kHYPE (0xfd739d4e...)
   - Debt: wHYPE (0x555555...)
   - User: 0xb83Dfb1D...
   - Debt covered: 25.009388 wHYPE
   - Collateral seized: 26.883796 kHYPE (7.49% bonus)

4. Swap kHYPE back to wHYPE via the SAME pool (two swaps):
   - Swap 1: 24.689 kHYPE -> 25.009 wHYPE (repay flash borrow)
   - Swap 2: 2.195 kHYPE -> 2.223 wHYPE (profit)

5. Withdraw 2.223 wHYPE as native HYPE to EOA

Profit: 2.223 wHYPE (~$66.70 at $30/HYPE)
Gas: 1,110,853 @ 272 gwei = 0.303 HYPE ($9.10)
Net profit: ~$57.61
```

**Flash loan intermediary:** `0x5c5c3a1b17f911932720821ee4d07680e8de7d93` (unverified contract, 43KB bytecode — likely a custom flash loan/swap aggregator)

#### What competitor can do that we CANNOT:

| Capability | Competitor | Our Contract |
|-----------|-----------|-------------|
| Multi-hop swaps | YES (swap steps array in calldata) | NO (exactInputSingle only) |
| Alt factory routing | YES (uses 0xFf7B... pools with 20x more liq) | NO (hardcoded HyperSwap) |
| Flash swap (0% premium) | YES (borrows from V3 pool) | NO (Aave flash loan, 4bps) |
| Correct fee tiers | YES (uses fee=100 for kHYPE/wHYPE) | NO (defaults to fee=3000) |
| Custom intermediary | YES (0x5c5c3a proxy for complex routes) | NO (direct router call only) |

### Competitor `0xa7d0485a` (full address unknown)

- No TXs found via Routescan API — likely uses a different full address than expected
- Mentioned in HANDOFF.md as "multi-protocol" competitor
- Unable to analyze without finding actual TXs

### Competitor `0xac51C585` (Morpho specialist)

- Mentioned in HANDOFF.md
- Not analyzed in this session

---

## Part 5: Recommended Swap Routes

### Optimal routes using alt factory (`0xFf7B3e8C...`)

| Liquidation Pair | Optimal Route | Hops | Pool(s) | Fee(s) |
|-----------------|---------------|------|---------|--------|
| kHYPE col / wHYPE debt | kHYPE -> wHYPE | 1 | `0xbe352d` | 100 |
| wstHYPE col / wHYPE debt | wstHYPE -> wHYPE | 1 | `0xff0A1d` | 100 |
| beHYPE col / wHYPE debt | beHYPE -> wHYPE | 1 | `0x6801B1` | 100 |
| UETH col / wHYPE debt | UETH -> wHYPE | 1 | `0xaf8023` | 3000 |
| kHYPE col / USDC debt | kHYPE -> wHYPE -> USDC | 2 | `0xbe352d`, `0x6c9A33` | 100, 500 |
| kHYPE col / UBTC debt | kHYPE -> wHYPE -> UBTC | 2 | `0xbe352d`, `0x0D6ECB` | 100, 3000 |
| kHYPE col / USDT0 debt | kHYPE -> wHYPE -> USDT0 | 2 | `0xbe352d`, `0xBd19E1` | 100, 500 |
| wstHYPE col / USDC debt | wstHYPE -> wHYPE -> USDC | 2 | `0xff0A1d`, `0x6c9A33` | 100, 500 |
| beHYPE col / USDC debt | beHYPE -> wHYPE -> USDC | 2 | `0x6801B1`, `0x6c9A33` | 100, 500 |
| UBTC col / wHYPE debt | UBTC -> wHYPE | 1 | `0x0D6ECB` | 3000 |
| UBTC col / USDC debt | UBTC -> wHYPE -> USDC | 2 | `0x0D6ECB`, `0x6c9A33` | 3000, 500 |
| USDC col / wHYPE debt | USDC -> wHYPE | 1 | `0x6c9A33` | 500 |
| USDe col / wHYPE debt | USDe -> wHYPE | 1 | `0x1c501a` (HyperSwap) or `0x...` (alt) | 3000 |
| USDe col / USDC debt | USDe -> USDC | 1 | `0xD2c34b` (alt) | 100 |
| USDC col / USDT0 debt | USDC -> USDT0 | 1 | `0x94291B` (alt) | 100 |

### sUSDe — UNSOLVABLE

sUSDe has **NO liquid pools** on either factory (all zero or non-existent). For sUSDe collateral:
- Option A: Check if there's an on-chain unwrap (sUSDe -> USDe), then swap USDe
- Option B: Use an aggregator (LiquidSwap, etc.) if they have a sUSDe route
- Option C: Skip sUSDe liquidations entirely until DEX liquidity develops

---

## Part 6: Action Items (Priority Order)

### BLOCKER #1: Deploy new contract with multi-hop + alt factory support

Changes needed in `HyperEVMFlashLoanLiquidator.sol`:
1. Add `exactInput()` call path in `_swapCollateral()` (the interface already has it!)
2. Approve alt factory router `0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B` in `approvedRouters`
3. Fix kHYPE/wHYPE fee tier from 3000 to 100
4. Add wHYPE/USDC fee=500, wHYPE/USDT0 fee=500 (for alt factory)
5. Consider flash swap pattern instead of Aave flash loan (saves 3bps per trade)

### BLOCKER #2: Update Rust executor to encode multi-hop paths

In `src/protocols/aave_v3/executor.rs`:
1. Build Uniswap V3 multi-hop path encoding: `tokenIn(20) + fee(3) + intermediate(20) + fee(3) + tokenOut(20)`
2. Route lookup: collateral -> optimal path (using pool map from this analysis)
3. Pass via `swapData` parameter to contract, with alt factory router as `swapRouter`

### IMPROVEMENT #3: Switch to alt factory as primary

The alt factory has 10-100x more liquidity for all key pairs. Either:
- A) Hardcode alt factory router as the default (simple)
- B) Add both routers and select per-pair based on liquidity (complex but better)
- C) Use flash swaps like the competitor (optimal but requires new contract)

### IMPROVEMENT #4: Correct fee tier mapping

Current wrong defaults and correct values:
| Pair | Current (wrong) | Correct (HyperSwap) | Correct (Alt Factory) |
|------|----------------|---------------------|----------------------|
| kHYPE/wHYPE | 3000 (fallback) | 100 | 100 |
| wstHYPE/wHYPE | 3000 (fallback) | 100 | 100 |
| beHYPE/wHYPE | 3000 (fallback) | 100 | 100 |
| wHYPE/USDC | 3000 | NO POOL | 500 |
| wHYPE/USDT0 | 3000 | NO POOL | 500 |

---

## Appendix A: Key Addresses

### Alt Factory (dominant liquidity)
| Contract | Address |
|----------|---------|
| UniswapV3Factory | `0xFf7B3e8C00e57ea31477c32A5B52a58Eea47b072` |
| SwapRouter | `0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B` |

### Key Pool Addresses (Alt Factory)
| Pair | Fee | Pool | Liquidity |
|------|-----|------|-----------|
| kHYPE/wHYPE | 100 | `0xbe352daF66af94ccF2012a154a67DAEF95FAcB91` | 7.61e25 |
| wHYPE/wstHYPE | 100 | `0xff0A1d682E614B8f913751aC97Fe2086a40C476A` | 3.56e23 |
| wHYPE/beHYPE | 100 | `0x6801B1Dbb4320C59AF532bA4E34B2365d4A638BE` | 8.28e21 |
| wHYPE/UETH | 3000 | `0xaf80230eB13222DB743C21762f65A046bb5F5437` | 3.61e22 |
| wHYPE/USDC | 500 | `0x6c9A33E3b592C0d65B3Ba59355d5Be0d38259285` | 1.66e19 |
| wHYPE/USDT0 | 500 | `0xBd19E19E4b70eB7F248695a42208bc1EdBBFb57D` | 1.58e18 |
| wHYPE/UBTC | 3000 | `0x0D6ECB912b6ee160e95Bc198b618Acc1bCb92525` | 1.73e17 |
| kHYPE/wstHYPE | 3000 | `0x1a4623841028C1b0a6f75E77D024D6dF3b51ce96` | 5.10e23 |
| kHYPE/UETH | 3000 | `0x40699d85809D10d416674390E698F04CF94DE61c` | 2.17e21 |
| kHYPE/UBTC | 3000 | `0x467364bd2a633208b4534f5b7eC11d24604546e4` | 4.80e15 |
| kHYPE/USDC | 3000 | `0x24a37584f98dADb64a0d1276266823AA8260bE54` | 7.73e15 |
| USDC/USDT0 | 100 | `0x94291BEA4c3aC9dBE81615083BB9A028722eeBeC` | 7.65e14 |
| USDC/USDe | 100 | `0xD2c34b86F9c2A1beB3D07eaFbA85a9D92Dc6A248` | 4.41e18 |
| USDe/USDT0 | 100 | `0x57a39276dA55040800eff10E4Bcaaa0DAFBb9a06` | 1.56e19 |
| USOL/wHYPE | 3000 | `0xC477f349F2912E034EAc45bC41Ec1F643580db2e` | 1.17e18 |

### Competitor Addresses
| Address | Role | Specialty |
|---------|------|-----------|
| `0xdd8692Bc25972DBa5906201960e2dbe783D460Fa` | Contract | HyperLend, kHYPE/wHYPE |
| `0xac5b4731229265627Cd3Fce62BB826493B38a66f` | EOA | Operator of 0xdd8692 |
| `0x5c5c3a1b17f911932720821ee4d07680e8de7d93` | Flash loan proxy | Used by 0xdd8692 (43KB bytecode) |

## Appendix B: Competitor TX Decoded

**TX:** `0x74d5b4e5f279b6fc4540b0b94a7fc5b8ff9a454cb54791961adf301880835e1e`
**Block:** 29134404 | **Gas:** 1,110,853 @ 273 gwei | **Net profit:** $57.61

```
Debt covered:        25.009388 wHYPE
Collateral seized:   26.883796 kHYPE (7.49% bonus)
Swap: 26.884 kHYPE -> 27.233 wHYPE (via pool 0xbe352d, fee=100)
Profit:              2.223 wHYPE ($66.70)
Gas cost:            0.303 HYPE ($9.10)
Net:                 $57.61

Strategy: Flash swap from V3 pool (zero premium)
          -> liquidate on HyperLend
          -> repay pool with seized kHYPE
          -> keep wHYPE profit
```
