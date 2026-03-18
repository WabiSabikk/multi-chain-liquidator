# Competitor Contract Analysis — HyperEVM (2026-03-11)

**Method:** Bytecode decompilation, function selector analysis, TX receipt decoding, storage reads via RPC
**Chain:** HyperEVM (999)
**Data source:** On-chain bytecode + liquidation events from `research/data-2026-03-10/`

---

## Executive Summary

Five independent teams operate the top HyperLend liquidation bots. All use Aave V3 multi-asset flash loans (`executeOperation(address[],uint256[],uint256[],address,bytes)`). The key differentiators are:

1. **Flash loan source:** Morpho (FREE, 0 fee) vs HyperLend (4 bps) vs V3 Pool flash (variable fee)
2. **Routing:** HyperSwap V3 Router (`0x4e29...`) vs Alt Router (`0x1ebd...`) vs direct pool `swap()`
3. **Architecture:** Single contract vs 2-contract (dispatcher+proxy) system
4. **Batch capability:** Top1 liquidates 3 users in one TX during crashes
5. **Calldata efficiency:** Top1 uses only 196 bytes input vs 772 bytes for 0xdd8692

---

## Contract Deep Dive

### Bot #1: `0x5d5ecb84e243ef4820ca70871c2b8db8a4194953` (56 liqs/30d, 24.2%)

**Bytecode:** 6,198 bytes
**Owner EOA:** `0x02a0abdf509acfe2a397d8ee1ff294be69513a17` (bal=3.97 HYPE, nonce=63)
**Creator:** Unknown (not on-chain verified)

**Contract State (via eth_call):**
| Function | Value | Notes |
|----------|-------|-------|
| `POOL()` | `0x00A89d7a...` | HyperLend Pool |
| `ADDRESSES_PROVIDER()` | `0x72c98246...` | HyperLend AddressesProvider |
| `SWAP_ROUTER()` | `0x4e2960a8...` | HyperSwap V3 Router (standard factory) |
| `owner()` | `0x02a0abdf...` | Operator EOA |

**Function Selectors:**
| Selector | Signature | Purpose |
|----------|-----------|---------|
| `0x1ceb8307` | UNKNOWN (main entry) | Liquidation trigger, custom encoding |
| `0x1b11d0ff` | `executeOperation(multi)` | Aave V3 flash loan callback |
| `0x0542975c` | `ADDRESSES_PROVIDER()` | View |
| `0x7535d246` | `POOL()` | View |
| `0xc6005893` | `SWAP_ROUTER()` | View |
| `0x57376198` | `rescueTokens(address,uint256)` | Admin token sweep |
| `0x8da5cb5b` | `owner()` | View |
| `0xf2fde38b` | `transferOwnership(address)` | Admin |

**Strategy:** Aave V3 multi-asset flash loan from HyperLend itself. Uses standard HyperSwap V3 Router. Batch-capable: liquidated 3 users in block 27043201 in a single TX.

**TX Analysis (0x4f6ea597..., block 27043872):**
- Selector: `0x1ceb8307` (custom, 196 bytes input — very compact)
- Gas used: 604,075 (moderate)
- Gas price: 0.55 gwei (LOW — early block during crash, no competition)
- Flow: HyperLend flash → V3 pool flash → liquidate (USDe collateral, wHYPE debt) → swap wHYPE→USDe → repay
- Profit sent to owner EOA

**Key advantage:** Batch liquidations during crashes. 56 liqs mostly concentrated in Feb 13-14 crash cluster (block ~27,042,000-27,044,000).

---

### Bot #2: `0x1ea48bb03ac81108ddf0ba4455d56dad58909d26` (50 liqs/30d, 21.6%)

**Bytecode:** 6,373 bytes
**Owner EOA:** `0x651088c41a98a7527618d70eb47985e5039f9fb7` (bal=2.46 HYPE, nonce=567)
**Hardcoded address:** `0x4d202bc4919b57341902c940589ff5dfdbc188be` (secondary EOA)

**Contract State:**
| Function | Value | Notes |
|----------|-------|-------|
| `aavePool()` | `0x00A89d7a...` | HyperLend Pool |
| `morpho()` | `0x68e37dE8...` | Morpho Core (!!) |
| `swapper()` | `0x7b235010...` | Custom swapper contract (4,367 bytes) |
| `owner()` | `0x651088c4...` | Operator EOA |

**Function Selectors:**
| Selector | Signature | Purpose |
|----------|-----------|---------|
| `0xb2e36a5f` | UNKNOWN (main entry) | Liquidation trigger |
| `0x1b11d0ff` | `executeOperation(multi)` | Aave V3 flash callback |
| `0xd8fbc833` | `morpho()` | View — Morpho integration |
| `0xa03e4bc3` | `aavePool()` | View |
| `0x2b3297f9` | `swapper()` | View — external swap contract |
| `0x9c82f2a4` | `setSwapper(address)` | Admin — can update swap router |
| `0x8da5cb5b` | `owner()` | View |

**Strategy:** Uses MORPHO FLASH LOANS (FREE, 0 fee) to liquidate on HyperLend. This is the optimal capital strategy — Morpho `flashLoan()` charges 0 bps vs HyperLend's 4 bps.

**TX Analysis (0x44fdf2a0..., block 28595382):**
- Selector: `0xb2e36a5f` (644 bytes input)
- Gas used: 438,471 (moderate)
- Gas price: 300 gwei
- Flow: Morpho FL (free) → wHYPE to contract → liquidate on HyperLend (wHYPE→wHYPE) → V3 pool flash for swap → profit to owner → repay Morpho
- **Competes head-to-head with Top3:** block 28595382 vs 28595384 (2-second gap)

**Key advantage:** Morpho flash loans = $0 capital cost. Separate swapper contract allows routing updates without redeploying main contract. Most sophisticated architecture among top 3.

---

### Bot #3: `0x8715edfdf0caddc4dddd4490282cd9fc01581cd3` (46 liqs/30d, 19.9%)

**Bytecode:** 10,145 bytes (LARGEST of top 3 — more complex logic)
**Owner EOA:** `0x85b195e137dbf189a6958ded9bd9c948bdd9d4e6` (bal=7.86 HYPE, nonce=831)

**Contract State:**
| Function | Value | Notes |
|----------|-------|-------|
| `pool()` | `0x00A89d7a...` | HyperLend Pool |
| `owner()` | `0x85b195e1...` | Operator EOA |

**Function Selectors:**
| Selector | Signature | Purpose |
|----------|-----------|---------|
| `0x16f0115b` | `pool()` | View |
| `0x1b11d0ff` | `executeOperation(multi)` | Aave V3 flash callback |
| `0x45f1d86c` | UNKNOWN | Custom logic |
| `0x59f613a4` | UNKNOWN | Custom logic |
| `0x715018a6` | UNKNOWN | Custom logic |
| `0xc25fac10` | UNKNOWN | Custom logic |
| `0xe5c1a348` | UNKNOWN | Custom logic |
| `0x8da5cb5b` | `owner()` | View |
| `0xf2fde38b` | `transferOwnership(address)` | Admin |

**Strategy:** Uses HyperLend flash loans (standard). Most complex bytecode (10KB) suggests sophisticated routing or profit optimization logic. Highest owner nonce (831) indicates many contract iterations.

---

### Bot `0xdd8692Bc25972DBa5906201960e2dbe783D460Fa` (9 liqs/30d, 3.9%)

**Bytecode:** 8,091 bytes (dispatcher) + 21,896 bytes (flash proxy `0x5c5c3a...`)
**Owner EOA:** `0xac5b4731...` (bal=124.73 HYPE — highest capital)
**Creator:** `0x77db0E5f7fe2e2b5c61f29a3c42b41e25a1d53a9` (340 days ago)
**Multiple operators:** `0x3d784C19...`, `0xac5b4731...`, `0x6516EFe8...`
**Total TXs:** 618

**Architecture: 2-Contract System**
- **Dispatcher (0xdd8692, 8KB):** Receives params via `a17922e2(address proxy, bytes params, ...)`, delegates to flash proxy
- **Flash Proxy (0x5c5c3a, 22KB):** Full DEX + lending integration

**Flash Proxy Function Selectors (key ones):**
| Selector | Signature | Purpose |
|----------|-----------|---------|
| `0x10d1e85c` | `executeOperation(single)` | Aave V3 single-asset flash callback |
| `0x1b11d0ff` | `executeOperation(multi)` | Aave V3 multi-asset flash callback |
| `0xfa461e33` | `uniswapV3FlashCallback` | V3 pool flash callback |
| `0xfa85398b` | `uniswapV3SwapCallback` | V3 swap callback |
| `0x23a69e75` | `uniswapV3SwapCallback` (alt) | Alt swap callback |
| `0xc04b8d59` | `exactInput(V3Router)` | Multi-hop swap |
| `0x38ed1739` | `swapExactTokensForTokens` | V2 Router swap |
| `0xbe4c43fb` | `multicall(bytes[])` | Batch operations |
| `0xcd786059` | `setApprovedTarget(address,bool)` | Whitelist management |
| `0x9b33f9a1` | `sweepToken(address)` | Admin token sweep |

**Hardcoded Addresses in Flash Proxy:**
- `0x4e2960a8...` = HyperSwap V3 Router
- `0x1ebdfc75...` = Alt V3 Router (higher liquidity pools)
- `0xb1c0fa0b...` = HyperSwap V3 Factory
- `0x888888888889758f76e7103c6cbf23abbf58f946` = PendleRouterV4

**Strategy:** V3 pool flash (NOT Aave flash loan) for kHYPE/wHYPE pairs. Uses own capital (124 HYPE balance). Dual-router support (HyperSwap + Alt factory). Has Pendle integration for yield token routing.

**TX Analysis (0x25f40968..., block 29069488):**
- EOA → dispatcher (0xa17922e2) → flash proxy → V3 pool flash → liquidation → 2x swaps → profit
- Gas used: 1,072,524 (HIGH — most expensive of all competitors)
- Gas price: 5,206.82 gwei (VERY HIGH — aggressive priority bidding)
- Swap path: kHYPE --[fee=100]--> wHYPE (via kHYPE/wHYPE pool `0xbe352daf...`)
- 24 event logs (most complex flow)

**Key advantage:** Versatile — supports V2 + V3 swaps, multicall, Pendle routing. Highest capital.
**Key weakness:** Extremely gas-heavy (2x gas of Top2), complex 2-contract architecture adds overhead.

---

### Bot `0xa7d0485a177236b19096a690210278271e133a0f` (10 liqs/30d, 4.3%)

**Bytecode:** 16,748 bytes (single contract)
**Owner EOA:** `0xAe4343Ea...` (bal=41.36 HYPE, nonce=511)
**Creator:** `0xAe4343Ea...` (82 days ago, block 28113246)
**Total TXs:** 479

**Function Selectors:**
| Selector | Signature | Purpose |
|----------|-----------|---------|
| `0x90be430a` | UNKNOWN (main HyperLend entry) | Custom liquidation trigger |
| `0x7d92f998` | UNKNOWN | Custom |
| `0xcf7ea196` | UNKNOWN | Custom |
| `0xd4fac45d` | `getBalance(address,address)` | Balance check |
| `0xfa461e33` | `uniswapV3FlashCallback` | V3 flash callback |
| `0xfa85398b` | `uniswapV3SwapCallback` | V3 swap callback |
| `0x128acb08` | `swap(V3Pool)` | Direct pool swap |
| `0x62acbb97` | `onMorphoLiquidate` | Morpho callback |
| `0x90b8ec18` | `onMorphoFlashLoan` | Morpho flash callback |
| `0xb12d13eb` | `liquidationCall` | Aave V3 liquidation |
| `0xd8eabcb8` | `flashLoan(Aave)` | Aave flash loan |
| `0x4697f05d` | `onFlashLoan(ERC-3156)` | ERC-3156 flash callback |
| `0xf5061d1e` | `onMorphoRepay` | Morpho repay callback |
| `0x2e1a7d4d` | `withdraw(uint256)` | WETH unwrap |

**Hardcoded Addresses:**
- `0x68e37dE8...` = Morpho Core
- `0x5555...5555` = wHYPE
- `0xbd19e19e4b70eb7f248695a42208bc1edbbfb57d` = UNKNOWN (likely oracle or market config)

**Strategy:** DUAL PROTOCOL — liquidates both HyperLend AND Morpho from the same contract. Uses Morpho flash loans (free) for HyperLend liquidations. Has ERC-3156 flash loan callback support. Direct pool `swap()` calls (bypasses router overhead).

**TX Analysis (0xfd5f0a12..., Morpho liquidation, block 29155688):**
- Gas used: 279,692 (LOWEST of all — Morpho callback is cheapest)
- Gas price: 546 gwei
- Flow: Morpho callback → receive ThBillOFT collateral → swap ThBillOFT→USDT0 → repay Morpho → send profit (16.56 USDT0) to owner
- 11 event logs (leanest execution)

**Key advantage:** Dual protocol + direct pool swaps (no router overhead) = lowest gas. Morpho callbacks are inherently cheaper than flash loans.

---

## Comparative Analysis

### Architecture Comparison

| Bot | Size | Contracts | Flash Source | Router | Protocols |
|-----|------|-----------|-------------|--------|-----------|
| Top1 (56) | 6.2KB | 1 | HyperLend FL (4bps) | HyperSwap V3 | HyperLend only |
| Top2 (50) | 6.4KB | 1 + swapper | Morpho FL (FREE) | Custom swapper | HyperLend + Morpho |
| Top3 (46) | 10.1KB | 1 | HyperLend FL (4bps) | Unknown | HyperLend only |
| 0xdd8692 (9) | 30KB | 2 (dispatcher+proxy) | V3 Pool flash | HyperSwap + Alt + Pendle | HyperLend + HypurrFi |
| 0xa7d048 (10) | 16.7KB | 1 | Morpho FL (FREE) | Direct pool swap | HyperLend + Morpho |

### Gas Efficiency

| Bot | Gas Used | Gas Price | Input Size | Cost per Liq |
|-----|----------|-----------|------------|-------------|
| Top1 | 604,075 | 0.55 gwei | 196 B | Very low (crash blocks = no competition) |
| Top2 | 438,471 | 300 gwei | 644 B | Moderate |
| 0xa7d048 | 279,692 | 546 gwei | 420 B | Lowest gas, moderate priority |
| 0xdd8692 | 1,072,524 | 5,206 gwei | 772 B | HIGHEST (2x gas, 10x priority) |

### Speed/Competition Analysis

- **Only 1 block in 30 days had 2+ liquidators competing** (block 27043201, all Top1)
- Close-block sequences (1-5s gaps) show bots respond within seconds
- Top1 dominates crash windows by batch-liquidating (3 liqs in 1 block)
- Top2 vs Top3: direct competition at block 28595382 vs 28595384 (Top2 wins by 2 blocks)
- **Most liquidations have NO direct competition** — the first bot to detect wins

### Operator Independence

All 5 operators are different EOAs with different funding patterns:

| Operator | Balance | Nonce | Interpretation |
|----------|---------|-------|---------------|
| Top1 owner | 3.97 HYPE | 63 | Efficient, few deploys |
| Top2 owner | 2.46 HYPE | 567 | Many iterations |
| Top3 owner | 7.86 HYPE | 831 | Most contract versions |
| 0xdd8692 owner | 124.73 HYPE | 198 | Capital-heavy strategy |
| 0xa7d048 owner | 41.36 HYPE | 511 | Moderate capital |

---

## Key Findings for Our Bot

### 1. MONITORING SPEED IS THE #1 COMPETITIVE EDGE

Most liquidations have zero direct competition — the first bot to DETECT the underwater position wins. This is not a gas war. Investment in faster HF calculation and oracle monitoring yields more than gas optimization.

### 2. Morpho Flash Loans Are Free Capital

Top2 and 0xa7d048 both use Morpho flash loans (0 bps fee) to liquidate HyperLend positions. This saves 4 bps vs using HyperLend's own flash loans. We should prioritize this in our executor.

### 3. Batch Liquidation During Crashes Is Critical

80% of liquidations happen in 2-3 day crash clusters. Top1 liquidated 3 users in one block by batching. Our bot needs batch liquidation support to maximize crash-window capture.

### 4. Small, Efficient Contracts Win

Top1 (6.2KB, 196-byte calldata) and Top2 (6.4KB) outperform the complex 30KB 0xdd8692 system. Simpler contracts = less gas = faster execution. Our 2-contract approach with ODOS integration adds overhead.

### 5. Direct Pool Swaps Beat Router Calls

0xa7d048 calls `swap()` directly on V3 pools, bypassing router overhead. This saves ~50-100K gas per TX. For common pairs (wHYPE/USDT0, kHYPE/wHYPE), we should encode direct pool swap paths.

### 6. Configurable Swap Router Is Smart

Top2's `swapper()` pattern (external swapper contract + `setSwapper()`) allows updating routing without redeploying the main contract. When DEX liquidity shifts, they just deploy a new swapper.

### 7. The Market Is Profitable But Small

$3,118-8,000/month across all HyperLend bots. Top1 captures ~24% = ~$750-1,900/month. Not massive, but the marginal cost is near zero once the bot is operational.

---

## Recommended Actions (Priority)

### P0: Switch to Morpho Flash Loans for HyperLend

Estimated savings: 4 bps per liquidation (immediate).

Currently we use HyperLend flash loans. Top2 and 0xa7d048 prove Morpho FL works and is free.

### P1: Improve Detection Speed

The competition is NOT a gas war — it's a detection war. Invest in:
- Precompile 0x0807 oracle reading (HyperCore system oracle, faster than HyperLend oracle chain)
- CEX WebSocket feeds (Binance/OKX/Bybit) for 1-3s pre-block price awareness
- More frequent polling (every 500ms instead of 1-5s)

### P2: Add Batch Liquidation

During crashes, multiple positions go underwater simultaneously. Top1 captures 3 in one block. We need:
- Multicall pattern: scan all positions, batch-liquidate in single TX
- Priority: process highest-value positions first

### P3: Direct Pool Swaps

For common pairs (kHYPE/wHYPE fee=100, wHYPE/USDT0 fee=500), bypass the router and call `swap()` directly on the pool contract. Saves ~50-100K gas.

### P4: Compact Calldata Encoding

Top1 uses only 196 bytes of calldata vs our likely 500+ bytes. Tighter encoding reduces TX propagation time and gas cost. Consider:
- Packed struct encoding instead of ABI encoding
- Pre-registered swap paths (index-based instead of address-based)

---

## Full Address Reference

| Label | Address |
|-------|---------|
| Top1 contract | `0x5d5ecb84e243ef4820ca70871c2b8db8a4194953` |
| Top1 owner | `0x02a0abdf509acfe2a397d8ee1ff294be69513a17` |
| Top2 contract | `0x1ea48bb03ac81108ddf0ba4455d56dad58909d26` |
| Top2 swapper | `0x7b2350100e3bbf9d292e56058511f4ecd67195f6` |
| Top2 owner | `0x651088c41a98a7527618d70eb47985e5039f9fb7` |
| Top3 contract | `0x8715edfdf0caddc4dddd4490282cd9fc01581cd3` |
| Top3 owner | `0x85b195e137dbf189a6958ded9bd9c948bdd9d4e6` |
| Bot 0xdd8692 | `0xdd8692bc25972dba5906201960e2dbe783d460fa` |
| Flash proxy | `0x5c5c3a1b17f911932720821ee4d07680e8de7d93` |
| Bot 0xa7d048 | `0xa7d0485a177236b19096a690210278271e133a0f` |
| kHYPE/wHYPE pool | `0xbe352daf66af5a4c5eaea1f8e1ac8d7d3a48c6ec` |
| ThBillOFT/USDT0 pool | `0xaf4bf7bd1cd226741b3b5bee3c8027cefd3d9345` |
| PendleRouterV4 | `0x888888888889758f76e7103c6cbf23abbf58f946` |
