# Market Opportunity Sizing — 2026-03-11

**Phase 7.5.6 — How much money is in each protocol's liquidation market?**

## Summary

| Protocol | All-Time Liqs | 30d Est. | Bots | Profit Pool/mo | Profit/Bot | Competition | Our Priority |
|----------|-------------|----------|------|----------------|------------|-------------|-------------|
| **HyperLend** | 2,373 | ~231 | 50 | ~$3,000 | ~$60 | Medium | **PRIMARY** |
| **Lendle** | 3,530 | ~108 | 88 | ~$1,080 | ~$12 | HIGH | SECONDARY |
| **Tydro** | 1,015 | ~60 | 17 | ~$600 | ~$35 | LOW (monopoly) | LOW |
| **HypurrFi** | 4,000+ est | ~0 (dormant) | 12 | $0 (dormant) | — | — | DORMANT |
| **Morpho** | ~3,000 est | ~35 | 55 | ~$350 | ~$6 | HIGH | TERTIARY |
| **Mantle Aave V3** | 0 | 0 | — | $0 | — | — | IGNORE |

**Total addressable monthly profit: ~$5,030**
**Realistic capture (10% of HyperLend + 3% of rest): ~$300-400/month**

## Per-Protocol Deep Dive

### HyperLend — $3,000/month profit pool

**Volume:** ~231 liqs/30d × avg ~$200 debt = ~$46K liquidation volume
**Avg profit per liq:** ~$13 (based on on-chain analysis from research/)
**Top bot profit:** `0x8715ed` (25.5% × $3K) = ~$765/month
**Market concentration:** Top 2 = 45%, Top 5 = 60%

**Why PRIMARY:**
1. Highest volume on HyperEVM
2. We have deployed contract + local node in Tokyo
3. Fixed DEX routing (v0.8.0)
4. Detection speed is the main bottleneck (not gas/routing)
5. Morpho FL can save 4 bps

**Crash multiplier:** During crashes (Feb 13-14), 350+ liqs in 4K blocks (~67 min). That's ~10x normal rate. Crash-ready = capture 50-100 liqs in one event.

### Lendle — $1,080/month profit pool

**Volume:** ~108 liqs/30d × avg ~$10 = ~$1,080 monthly
**Market concentration:** Top 1 = 11.2%, very fragmented (88 bots)
**Gas:** MNT (~$0.00001/tx) — negligible cost

**Why SECONDARY:**
- Very fragmented market (88 bots, hard to get share)
- FCFS ordering confirmed — speed matters
- Small average liquidation size
- Contract already deployed

### Tydro (Ink) — $600/month, monopoly by 0xf0570e

**Volume:** ~60 liqs/30d × avg $10 = ~$600
**Monopoly:** `0xf0570e` takes 78.2%
**Sequencer:** Gelato (centralized, FCFS)

**Why LOW:**
- Near-monopoly makes entry hard
- Small volume
- Need to understand how `0xf0570e` achieves dominance before competing

### HypurrFi — Dormant since Jan 2026

**Historical peak:** Dec 17-18: 555 liqs in 2 days, Sep 24-25: 246
**Current:** 0 liqs/30d

**Opportunity:** Deploy contract + be ready for next crash. When HypurrFi reactivates, 12 old bots will compete. Low competition = high capture rate IF we have a working contract.

### Morpho — ~$350/month, high fragmentation

**Volume:** ~35 liqs/30d × avg ~$10 = ~$350
**Bots:** 55 — most competitive market
**Our advantage:** Already have callback contract

## Revenue Projections

### Conservative scenario (after P0-P1 fixes)

| Protocol | Our Share | Monthly Liqs | Avg Profit | Monthly Revenue |
|----------|----------|-------------|------------|----------------|
| HyperLend | 5% | 12 | $13 | $156 |
| Lendle | 3% | 3 | $10 | $30 |
| Morpho | 2% | 1 | $10 | $10 |
| **Total** | | | | **$196/month** |

### Optimistic scenario (after Phase 10 oracle prediction)

| Protocol | Our Share | Monthly Liqs | Avg Profit | Monthly Revenue |
|----------|----------|-------------|------------|----------------|
| HyperLend | 15% | 35 | $13 | $455 |
| Lendle | 5% | 5 | $10 | $50 |
| Morpho | 5% | 2 | $10 | $20 |
| HypurrFi (crash) | 10% | 5 | $15 | $75 |
| **Total** | | | | **$600/month** |

### Crash bonus (one 3-day crash event per month)

| Protocol | Crash Liqs | Our Share | Avg Profit | Bonus |
|----------|-----------|----------|------------|-------|
| HyperLend | 350 | 10% | $13 | $455 |
| Lendle | 200 | 5% | $10 | $100 |
| HypurrFi | 200 | 10% | $15 | $300 |
| **Total crash bonus** | | | | **$855** |

## Cost Structure

| Cost | Monthly |
|------|---------|
| AWS Tokyo (i3.xlarge) | ~$100 |
| Gas (HYPE + MNT + ETH) | ~$5-10 |
| RPC (dRPC free tier) | $0 |
| **Total** | **~$110** |

## Break-Even Analysis

**Current (v0.8.0, $0 revenue):** -$100/month

**After P0-P1 fixes (conservative):** $196 - $110 = **+$86/month**

**After Phase 10 + crash bonus:** $600 + $855 - $110 = **+$1,345/month**

**Break-even requires:** Either oracle prediction working OR regular crash events.

## Risks/Counterarguments

1. **Market may stay calm** — no crashes = no liquidations = no revenue
2. **Competition increasing** — 50 bots on HyperLend, growing
3. **SVR/OEV** — Chainlink SVR may capture 65% of OEV, reducing bot profits
4. **Protocol changes** — Aave governance can change parameters
5. **Gas cost spiral** — if gas bidding becomes norm, margins shrink
6. **Server cost** — $500/mo for i3.xlarge is significant vs expected revenue

## Recommendation

**Continue development with focus on HyperLend + oracle prediction (Phase 10).** The market is small (~$5K/month total) but:
- Marginal cost is near zero (bot already running)
- Crash events can generate $855+ bonus
- Oracle prediction is a transferable skill (applies to Arbitrum F1 too)
- If SVR doesn't capture 100% of OEV, there's still opportunity

**Kill criteria:** If after 30 days post-fixes, we have <5 successful liquidations during a crash event → re-evaluate the server cost.

**Alternative:** Downsize to t3.medium (~$100/mo) if i3.xlarge not needed after validating local node isn't critical.
