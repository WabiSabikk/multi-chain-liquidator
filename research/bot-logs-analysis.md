# Bot Logs Analysis — 2026-03-10

**Period:** 2026-03-08 17:19 UTC — 2026-03-10 06:01 UTC (~36.5 hours)
**Version:** v0.5.0 (turbo mode), LIVE mode
**Server:** Tokyo (i3.xlarge), pm2 process `multi-chain-liq`
**Log file:** `~/.pm2/logs/multi-chain-liq-out.log` (181 MB, 636,605 lines)

---

## 1. Bot Status

| Metric | Value |
|--------|-------|
| Current status | **ONLINE** |
| Uptime at check | 14 minutes (just restarted) |
| Total restarts (pm2 ↺) | **7** (within 36.5h period) |
| Restart count | 12 startup banners in logs |
| Successful liquidations | **0** |
| Total profit | **$0.00** |

### Restart Timeline
| Time (UTC) | Notes |
|------------|-------|
| 2026-03-08 17:19 | First startup |
| 2026-03-08 17:20 | Immediate restart (~1 min) |
| 2026-03-08 17:20 | Immediate restart again |
| 2026-03-08 17:45 | Restart after 25 min |
| 2026-03-08 17:50 | Restart after 5 min |
| 2026-03-08 23:10 | Restart after 5h20m |
| 2026-03-09 04:50 | Restart after 5h40m |
| 2026-03-09 05:38 | Restart after 48 min |
| 2026-03-09 05:40 | Restart after 2 min |
| 2026-03-09 05:42 | Restart after 2 min |
| 2026-03-09 09:03 | Restart after 3h21m (long stable run) |
| 2026-03-10 05:43 | Restart after 20h40m (longest run) |

---

## 2. Chain/Protocol Activity Summary

### HyperEVM/HyperLend (MOST ACTIVE)
- Borrowers monitored: **3,604**
- Positions at risk: **83-86** (HF 1.00-1.10)
- Liquidatable (HF < 1.0): **0** (none crossed threshold)
- Dust positions filtered: **852**
- Scan frequency: every ~5 min
- Speculative fires: **33,086**
- Speculative TXs sent: **2,470**
- Speculative reverts (HF > 1.0): **1,770**

### HyperEVM/HypurrFi Pool (DORMANT)
- Borrowers monitored: **43**
- At risk: **0-1**
- Liquidatable: **0**
- Status: dormant since Jan 2026, no opportunities

### Ink/Tydro (MONITORING)
- Borrowers monitored: **16,599**
- At risk: **101-358** (increasing over time)
- Liquidatable: **0**
- Blacklisted: 1 address (`0x2C1c97d6` — bad debt MustNotLeaveDust)

### Mantle/Aave V3 (OFFLINE since Mar 9)
- Borrowers monitored: **184**
- At risk: **43**
- Last STATS: 2026-03-09 00:06 UTC
- **Status: DEAD** — "Block poll failed: backend connection task has stopped"
- Alchemy WSS: 429 Too Many Requests on every restart
- Fell back to HTTP, then connection dropped entirely around 04:48 Mar 9
- NOT recovered after restarts (keeps getting 429)

### Mantle/Lendle (OFFLINE since Mar 9)
- Same as above — shares Alchemy RPC, same 429 errors

### HyperEVM/Morpho (STUCK IN INIT)
- Found **53 liquidatable positions** + **329 at risk** on first scan (Mar 8, 23:09)
- **106 MORPHO CANDIDATE entries** logged
- All candidates are **DUST** — borrowed amounts: 0.00, 0.01, 0.03, 0.05, 0.22
- Pairs: UETH/USDHL (38), UBTC/USDHL (26), various/USDT0 (42)
- After restart, Morpho re-scans CreateMarket from block 4,000,000 to current
- **getLogs errors on blocks 6,986,000-7,016,000+** (invalid block range)
- This means Morpho init scan runs for hours, gets stuck, then bot restarts and starts over

---

## 3. Speculative Execution Analysis

### Fire Statistics
| Target | Pair | Debt | HF | Fires | Outcome |
|--------|------|------|----|-------|---------|
| `0xeC23c052` | WHYPE -> WHYPE | $1,295-$1,466 | 1.0013 | **25,837** | All revert/nonce fail |
| `0xB9F633d2` | USDe -> USD0 | $9,286 | 1.0046 | **7,015** | All revert/nonce fail |
| `0xf7449181` | UBTC -> kHYPE | $350 | 1.0047 | **128** | All fail |
| `0xb83Dfb1D` | USDH -> kHYPE | $3,269 | 1.0042 | **106** | All fail |

### Why all 33,086 speculative fires failed:

**Root cause breakdown:**
| Failure Type | Count | % |
|-------------|-------|---|
| Insufficient funds for gas | **21,506** | 65.0% |
| Nonce too low | **8,627** | 26.1% |
| Speculative REVERT (HF > 1.0) | **1,770** | 5.3% |
| RPC errors (502, -32602, -32603) | **604** | 1.8% |
| TX sent but no confirmation/success | **579** | 1.8% |

### Interpretation:

1. **65% — Insufficient gas funds**: Wallet `0xaa979a...` ran out of HYPE gas. The speculative system fires TX every ~3 seconds for near-liquidation targets (HF 1.0013). At nonce 10 → 2446 in 36h = 2,436 TXs attempted. Each reverted TX still costs gas. The wallet was drained.

2. **26% — Nonce too low**: Speculative sends use pre-computed nonces. After reverts, the nonce state becomes stale. The bot sends with nonce 2228 when chain expects 2305. This is a cascade effect of rapid speculative firing.

3. **5.3% — HF > 1.0 reverts**: Expected behavior. Speculative TX lands but position HF has recovered above 1.0 by the time TX is included. This confirms the system DOES fire before the position is truly liquidatable — it's just always too early.

4. **1.8% — RPC errors**: `invalid block height: 29288872` — the local hl-visor node is behind or pruned old blocks. **176,350 of these errors** in the log (background noise, ~1 every 250ms).

---

## 4. Specific Missed Opportunities

### 4.1 Target `0xeC23c052` — HyperLend (WHYPE -> WHYPE, $1,466)
- HF sitting at **1.0013** for 36+ hours
- Bot fires speculative TX every ~3 seconds
- **This is NOT a missed opportunity** — HF never dropped below 1.0
- The position is near-liquidation but hasn't crossed the threshold
- A 0.13% price move in HYPE would make it liquidatable

### 4.2 Target `0xB9F633d2` — HyperLend (USDe -> USD0, $9,286)
- HF at **1.0046** — stable, not crossing 1.0
- Even if liquidated: single-hop swap USDe->USD0 may have no direct pool
- **Not a missed opportunity** — needs price drop

### 4.3 Target `0xf7449181` — HyperLend (UBTC -> kHYPE, $350)
- HF at **1.0047** — not liquidatable
- **$350 debt is below most profitable thresholds**
- kHYPE collateral requires multi-hop swap (no direct UBTC/kHYPE pool)

### 4.4 Morpho candidates — All DUST
- 53 positions flagged as liquidatable (HF 0.32-0.92)
- ALL have borrowed amounts of **$0.00-$0.22**
- Profit after gas would be negative
- **Not worth pursuing**

### 4.5 Real missed opportunities (NOT from bot logs — from competitors)
- Competitors `0xdd8692bc` and `0xa7d0485a` are doing liquidations on HyperLend
- They use multi-hop swaps that our contract doesn't support
- **We can't estimate exact missed profit without on-chain analysis** of competitor TXs during this period

---

## 5. Critical Issues Found

### Issue #1: GAS EXHAUSTION (CRITICAL)
- **21,506 "insufficient funds" errors** — wallet has no HYPE for gas
- Speculative system burns gas on every attempt (~3s intervals)
- At $0.01/tx * 2,470 TXs sent = ~$25 in gas burned with $0 revenue
- **Fix needed**: Either disable speculative for HF > 1.005, or top up gas, or add gas check before sending

### Issue #2: MANTLE COMPLETELY OFFLINE (HIGH)
- Alchemy RPC returns 429 (rate limited) on WSS
- HTTP fallback also died ("backend connection task has stopped")
- **Mantle has not been monitored since Mar 9 00:06 UTC** (30+ hours blind)
- Lendle has ~11 liqs/7d = could have missed 4-5 liquidation opportunities
- **Fix needed**: Switch to dRPC or QuickNode for Mantle

### Issue #3: MORPHO STUCK IN INIT LOOP (HIGH)
- Every restart: Morpho re-scans CreateMarket from block 4,000,000
- Local node returns "invalid block range" for blocks 6,986,000-7,016,000
- Scan takes hours, never completes before next restart
- **Only 2 successful Morpho scans** in entire 36.5h period
- Morpho on HyperEVM has 55 competing bots doing liquidations we're blind to
- **Fix needed**: Cache market discovery, skip problematic block ranges, or start from later block

### Issue #4: INVALID BLOCK HEIGHT SPAM (MEDIUM)
- **176,350 errors** for block 29288872 "invalid block height"
- Some component is subscribed to a block that the local node considers invalid
- ~3 errors per second, constant noise
- Doesn't break functionality but wastes resources and pollutes logs

### Issue #5: NONCE DESYNC (MEDIUM)
- Speculative system uses pre-computed nonces
- After failed TXs, nonce gets out of sync (e.g., tx nonce 2228 vs chain nonce 2305)
- Creates cascading failures where subsequent TXs also fail
- **77 nonces ahead** at one point = 77 TXs that will all fail

---

## 6. Root Cause Breakdown (Pie Chart)

```
FAILURE DISTRIBUTION (33,086 speculative fires, 0 successes):

  ████████████████████████████████  65.0%  Insufficient gas funds
  ████████████████                 26.1%  Nonce too low (cascade)
  ███                               5.3%  HF > 1.0 (expected)
  █                                 1.8%  RPC errors
  █                                 1.8%  TX sent, no confirmation
```

**Underlying causes:**
- **65% + 26% = 91%** of failures are because speculative system fires continuously on a target with HF 1.0013 (not yet liquidatable), draining gas and desyncing nonces
- **5.3%** are expected — speculative TX lands but position recovered
- **3.6%** infrastructure issues (RPC, node sync)

---

## 7. Estimated Missed Profit

### From Bot Logs: **$0.00**
No position in the logs ever reached HF < 1.0. All speculative fires targeted positions with HF 1.0013-1.0047. None were actually liquidatable.

### From Competitor Activity (estimated):
- HyperLend: ~132 liqs/30d = ~4.4/day. Over 36.5h = **~6-7 liquidations happened** (by competitors)
- We couldn't participate because:
  1. Gas exhausted (no HYPE left)
  2. Single-hop swap contract (can't do kHYPE->wHYPE->UBTC routes)
  3. Even if we had gas, HF 1.0013 targets weren't liquidatable
- **Estimated missed profit: $50-200** (based on avg $25-30 profit per HyperLend liquidation from competitor analysis)

### From Mantle downtime:
- Lendle: ~11 liqs/7d = 1.57/day. Over 30h offline = ~2 liquidations potentially missed
- **Estimated missed profit: $5-20** (Lendle liqs are typically small, $100-500 debt)

### From Morpho blindness:
- 53 liquidatable positions found but all dust ($0-$0.22 borrowed)
- **$0 missed** — nothing profitable was available

### Total estimated missed profit: **$55-$220**

---

## 8. Recommendations (Priority Order)

1. **STOP speculative firing on HF > 1.003** — Burning gas on positions that are NOT liquidatable. Set threshold to HF <= 1.001 or even <= 1.0005.

2. **Top up HYPE gas** — Wallet is empty. Need at minimum 1-2 HYPE for operations.

3. **Fix Mantle RPC** — Switch from Alchemy (rate limited) to dRPC or alternative. Mantle has been completely blind for 30+ hours.

4. **Fix Morpho init** — Cache CreateMarket results, don't re-scan from block 4M on every restart. Use a checkpoint file.

5. **Deploy multi-hop swap contract** — This is the #1 revenue blocker. Competitors profit from kHYPE/wHYPE/UBTC routes we can't access.

6. **Fix invalid block height spam** — Some subscription is stuck on block 29288872. Find and fix or add retry backoff.

7. **Add nonce recovery** — After "nonce too low", fetch fresh nonce from chain instead of incrementing stale value.
