# Mantle Aave V3 — Position Analysis (2026-03-06)

## Overview
- **Total borrowers:** 53 (from 135 borrow transactions)
- **Active positions with debt:** 48
- **Total collateral:** $296M
- **Total debt:** $261M
- **Liquidations to date:** 0 (protocol deployed 11 Feb 2026)

## Why 0 Liquidations

Dominant strategy: stablecoin loops (sUSDe -> borrow USDT0 -> buy more sUSDe).
Top 7 positions = $252M collateral, $226M debt. All use eMode (LiqThreshold 92-95%).
At HF=1.023, collateral needs to fall ~2.3% relative to debt for liquidation.
For correlated stablecoin pairs, this requires sUSDe depeg of ~2.5%.

## Positions by Risk (sorted by Health Factor)

### Critical (HF < 1.05) - $256M at risk

| Address | Collateral | Debt | HF | LiqThr |
|---------|-----------|------|-----|--------|
| 0xA0CD1C... | $0 | $0 | 1.0036 | 45% |
| 0x0fe15b... | $14.4M | $13.0M | 1.0222 | 92% |
| 0x328c94... | $6.8M | $6.4M | 1.0223 | 95% |
| 0xd6c757... | $9.4M | $8.5M | 1.0224 | 92% |
| 0x502D22... | $26.8M | $24.1M | 1.0230 | 92% |
| 0xb3abe0... | $38.9M | $35.0M | 1.0232 | 92% |
| **0x920Eef...** | **$127.5M** | **$114.6M** | **1.0237** | **92%** |
| 0xdf9E6B... | $424K | $381K | 1.0245 | 92% |
| 0xf74625... | $28.6M | $26.5M | 1.0246 | 95% |
| 0xaD7866... | $18.9K | $16.9K | 1.0272 | 92% |
| 0xd8495B... | $24.2M | $22.3M | 1.0303 | 95% |
| 0x16B584... | $595K | $523K | 1.0459 | 92% |

### Near liquidation (HF 1.05-1.15)

| Address | Collateral | Debt | HF | LiqThr |
|---------|-----------|------|-----|--------|
| 0x78FA72... | $61 | $50 | 1.1247 | 92% |
| 0x587171... | $1 | $0 | 1.1290 | 45% |
| 0x7bD8D7... | $120K | $96.5K | 1.1461 | 92% |
| 0x6e6abB... | $251K | $201K | 1.1488 | 92% |

### At risk (HF 1.15-1.5) - ~$15M non-correlated

12 positions with HF 1.19-1.49, mostly non-stablecoin (ETH/BTC collateral, USD debt).
These liquidate FIRST on market crashes (not depeg events).

## Liquidation Scenarios

### Scenario 1: sUSDe depeg 3%
- All eMode positions (HF 1.02-1.05) become liquidatable
- Cascade: $256M+ collateral, $226M+ debt
- Single position 0x920Eef = $127.5M → liquidator bonus ~5% = **$6.4M potential**
- Flash loan max = Aave V3 pool reserves (check available liquidity per asset)

### Scenario 2: ETH drops 15%
- Non-correlated positions (HF 1.12-1.5) at risk
- ~$15M in these positions
- More frequent than depeg, smaller opportunity

### Scenario 3: BTC drops 20%
- FBTC collateral positions become vulnerable
- Limited FBTC exposure in current positions

## Key Whale: 0x920EefBCf1f5756109952E6Ff6dA1Cab950C64d7
- Collateral: $127.5M (43% of entire protocol TVL)
- Debt: $114.6M
- HF: 1.0237
- LiqThreshold: 92% (eMode)
- Likely strategy: sUSDe loop for yield farming
- This single position would generate more liquidation profit than entire Lendle protocol

## Recommendations
1. Monitor sUSDe/USDT0 and sUSDe/USDe price feeds closely
2. Bot should track Ethena protocol health (backing, redemption queue)
3. Set up alerts for HF < 1.01 on top 5 positions
4. Ensure flash loan liquidity is available (check Aave V3 reserve utilization)
5. This is a "wait for the earthquake" play — when it hits, it's massive
