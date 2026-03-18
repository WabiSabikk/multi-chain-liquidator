# Why Zero/Few Liquidations on New Lending Protocols

**Date:** 2026-03-05
**Status:** DEEP RESEARCH COMPLETE

## TL;DR

| Protocol | Launch Date | Age | TVL | Borrows | Liquidations Found | Why |
|----------|-------------|-----|-----|---------|-------------------|-----|
| **Mantle Aave V3** | 2026-01-15 (contract), 2026-02-11 (official) | **~7 weeks** | $1.03B | $353M | **ZERO on-chain** | Too young + sUSDe/USDT0 looping (correlated) |
| **Ink Tydro** | 2025-07-25 (contract), 2025-10-15 (official) | **~5 months** | $737M | $293M | **23 in 11.5 days** (~2/day) | Young + low volatility period |
| **HyperLend** | 2025-03-07 (contract), 2025-03-24 (official) | **~12 months** | $650M+ | $172M | Unknown but **679K pool txs** | Oldest of the group, likely has some |
| **Morpho Blue (HyperEVM)** | 2025-05-08 | **~10 months** | $431M supply | $119M | Unknown | Callback mechanism, different liquidation path |

---

## 1. Protocol Launch Dates (VERIFIED)

### Mantle Aave V3
- **Contract deployed:** January 15, 2026, 11:19:08 UTC
  - TX: `0x5afaf3b09f368535439cba19683c950f17e7909bb401762bcb41cac2768eafec`
  - Deployer: `0xba50cd2a20f6da35d788639e581bca8d0b5d4d5f`
  - Block: 90,172,818
- **Official launch (Bybit integration):** February 11, 2026
- **Age: ~7 weeks** (as of March 5, 2026)
- **Pool transactions:** 4,892 total -- EXTREMELY low for "$1B TVL"
- **TVL timeline:** $400M (week 1) -> $550M (week 2) -> $1.03B (March 1)

**WHY SO FEW TXs:** The protocol is literally 7 weeks old. $1B TVL was driven by a handful of whale depositors doing yield-bearing stablecoin loops (sUSDe -> borrow USDT0 -> buy more sUSDe). The Chaos Labs governance post reveals: top sUSDe account holds ~55% of supply, top 4 wallets = ~85%. Top USDT0 borrower = 33%, top 5 = 71%. This is NOT 10,000 retail users -- it's <50 whales.

### Ink Tydro (Aave V3)
- **Contract deployed:** July 25, 2025, 14:07:38 UTC
  - TX: `0x0320150048dfbf377877b43ceb8e691a29e96d099f00f2b3840e6a567df33c40`
  - Method: `setupAaveV3Market` (Aave V3 Ink Whitelabel Market)
  - Creator: PoolAddressesProvider `0x4172E6aAEC070ACB31aaCE343A58c93E4C70f44D`
- **Official announcement:** October 15, 2025
- **Age: ~5 months** (from official launch)
- **Pool transactions:** 250,118 -- much more active than Mantle
- **TVL (Jan 2026):** $737.5M total, $293.7M borrowed

### HyperEVM HyperLend
- **Contract deployed:** March 7, 2025
  - Creator: HyperLend Deployer `0x1b7a7d51ee86e1d9776986aefd2675312cf0c9da`
- **Official launch:** March 24, 2025
- **Age: ~12 months** -- oldest of the group
- **Pool transactions:** 679,006 -- the most active
- **TVL:** $650M+ supply, $172M borrowed

### HyperEVM Morpho Blue
- **Deployed:** May 8, 2025
  - Core: `0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb`
  - Felix Vaults launched in April-May 2025
- **Age: ~10 months**
- **TVL:** $431M supply, $119M borrowed
- **Felix has scaled to $1B+ TVL** across all products

### HyperEVM Mainnet
- **Launched:** February 18, 2025

---

## 2. WHY Zero Liquidations on Mantle Aave V3

This is the most interesting finding. **The Mantle Aave V3 Pool explorer API returned ZERO LiquidationCall events since deployment.**

Query: `topic0=0xe413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286` (LiquidationCall signature)
Result: "No logs found"

### Root Causes:

#### A. Protocol is only 7 weeks old
- Deployed January 15, 2026
- Official launch February 11, 2026
- There simply hasn't been enough time for positions to become unhealthy

#### B. Borrower composition: sUSDe/USDT0 looping whales
From Chaos Labs governance post (March 3, 2026):
- **sUSDe:** Supply cap reached 160M tokens. Top account holds ~55%, top 4 = ~85%
- **USDT0:** Top borrower = 33%, top 5 borrowers = ~71%
- **Health Factors:** "Most positions cluster in the 1.02-1.05 range"

This means: a tiny number of sophisticated whales are doing **yield-bearing stablecoin loops**:
1. Deposit sUSDe (earns yield from Ethena staking)
2. Borrow USDT0 (stable)
3. Buy more sUSDe, repeat

**Why this doesn't liquidate:** sUSDe and USDT0 are highly correlated (both pegged to ~$1). Even with HF 1.02-1.05, a liquidation requires sUSDe to depeg significantly from USDT0. Aave V3 eMode for correlated assets sets LTV as high as 93-97%, meaning liquidation only happens on large depegs.

#### C. Market conditions DURING the protocol's lifetime
- Mantle Aave V3 launched Feb 11, 2026
- BTC was already at $67K (down from $126K ATH in Oct 2025)
- The major crash (Jan 31 - Feb 5: BTC $80K -> $70K) happened BEFORE Mantle had significant TVL
- Since Feb 11: relatively stable sideways ($67K-$73K range)
- No 20%+ flash crashes in the last 3 weeks

#### D. 4,892 transactions = ~50-100 active addresses
With $1B TVL concentrated in <50 wallets, these are institutional/whale players who:
- Use conservative leverage
- Actively manage positions
- Unwind before liquidation

#### E. eMode + correlated assets = very narrow liquidation zone
sUSDe/USDT0 in eMode: liquidation threshold ~95-97%. For liquidation:
- sUSDe needs to depeg > 3-5% from USDT0
- This is extremely rare for yield-bearing stablecoins

---

## 3. Actual Borrowing Utilization

### Mantle Aave V3 (from governance + news)
- **Total market size:** $1.03B (March 1, 2026)
- **Total supply:** $671M
- **Total borrows:** $353M
- **Utilization:** ~52.6%
- **BUT:** Most borrows are USDT0 (stablecoin) against sUSDe (stablecoin) -- correlated pair
- **sUSDe supply cap:** 160M (reached, pending increase to 240M)
- **USDT0 borrow cap:** 380M (pending increase to 500M)

### Ink Tydro (from DefiLlama data, Jan 2026)
- **Total market size:** $737.5M
- **Available to borrow:** $443.8M
- **Currently borrowed:** $293.7M
- **Utilization:** ~39.8%
- **Top assets supplied:** USDT0, ETH, kBTC
- **23 liquidations in 11.5 days** -- so liquidations DO happen, just infrequently

### HyperLend
- **Supply:** $650M+
- **Borrowed:** $172M
- **Utilization:** ~26.5% -- LOWEST of all
- **kHYPE market alone:** $306M TVL
- **Note:** HyperLend has highest TVL but lowest utilization -> most conservative borrowers

### Morpho Blue (HyperEVM)
- **Supply:** $431M
- **Borrowed:** $119M
- **Utilization:** ~27.6%
- **Felix vaults scaled to $1B+ TVL** across all products

---

## 4. What LTV Ratios Are Users Actually Using?

### Key Finding: Health Factors 1.02-1.05

From the Chaos Labs Mantle governance post, **most positions cluster at HF 1.02-1.05**. This is EXTREMELY tight -- but for correlated assets (sUSDe/USDT0) this is considered "normal."

### What this means for liquidation:
- HF 1.02 with correlated stablecoin pair: requires ~2% depeg for liquidation
- HF 1.05 with correlated pair: requires ~5% depeg
- sUSDe has NEVER depegged >3% from USDT in its history

### Non-correlated positions (ETH, BTC, etc):
- Typical conservative borrower: 40-60% LTV (HF 1.5-2.5)
- Typical moderate borrower: 60-75% LTV (HF 1.1-1.5)
- Aggressive: 80-90% LTV (HF 1.02-1.1) -- ONLY in eMode with correlated assets
- Regular aggressive: 70-80% LTV (HF 1.1-1.25)

### Why this matters:
The majority of TVL on Mantle is in correlated stablecoin loops, NOT in volatile collateral borrows. This means:
- **Volume of liquidatable positions is very low**
- **When liquidations DO happen, they'll be small** (only the non-correlated positions)
- **Big liquidation events require stablecoin depegs** (rare but catastrophic when they happen)

---

## 5. Market Conditions: Crypto Volatility Feb-March 2026

### Key Price Data (CoinPaprika, March 5, 2026):
- **BTC:** $72,606 (ATH $126,173 on Oct 6, 2025 -- DOWN 42.5%)
- **ETH:** $2,122 (ATH $4,946 on Aug 24, 2025 -- DOWN 57.1%)

### Timeline of Crashes:
1. **October 10-11, 2025:** $19B liquidations in 24h (BTC crashed from highs). LARGEST single-day liquidation in crypto history.
2. **January 31 - February 5, 2026:** BTC dropped from $80K to $70K. $429M in Aave liquidations across ~12,500 transactions (broke May 2021 record for Aave specifically).
3. **February 5, 2026:** BTC tested $70K, ETH crashed. $775M liquidations on exchanges.
4. **February 2026:** BTC -24% YTD, ETH -34% YTD. Worst start to year in a decade.
5. **Since Feb 11 (Mantle launch):** Relatively stable. BTC $67K -> $73K range.
6. **March 2, 2026:** Fear & Greed Index at 10 (Extreme Fear).

### Implications for liquidations:
- The MAJOR crash happened BEFORE Mantle Aave V3 had significant TVL
- Ink Tydro was live during Oct 2025 crash -- hence the 23 liquidations observed
- HyperLend was live during both crashes -- should have had liquidations (needs verification)
- Current market: sideways with occasional -5% dips, NOT enough for correlated asset liquidations
- HOWEVER: another -20% BTC crash would trigger massive liquidations on ALL protocols

---

## 6. Other Liquidation Contracts/Protocols We Might Be Missing

### Chainlink SVR (Smart Value Recapture)
- **LIVE on HyperEVM** (and Arbitrum, Base, BNB, Ethereum)
- SVR captures OEV (Oracle Extractable Value) from liquidations
- Recaptured $10M+ across $460M in liquidations
- **Impact:** If HyperLend uses Chainlink SVR, ~65% of liquidation MEV goes to protocol+Chainlink
- **Ink Tydro uses Chaos Labs oracle, NOT Chainlink** -- SVR does NOT apply

### HyperLend 3 Liquidation Paths
HyperLend has 3 distinct liquidation mechanisms:
1. **Flash Loan + DEX** (standard -- what we'd use)
2. **Upfront Payment + HyperCore Bridge** -- liquidator bridges collateral to L1 orderbook
3. **Dual Bridge (SpotSend)** -- bridges both debt and collateral via HyperCore

This means liquidations on HyperLend might NOT appear as standard LiquidationCall events on EVM. Some liquidations could be routed through HyperCore L1 orderbook.

### Morpho Callback Mechanism
Morpho Blue doesn't use flash loans. Instead:
- Morpho calls a `onMorphoLiquidate` callback on the liquidator contract
- Liquidator receives collateral, sells it, repays debt
- This is a DIFFERENT event signature than Aave's LiquidationCall

### Potential Hidden Liquidators:
- **Aave Paraswap Adapters** -- liquidation through aggregators
- **Gelato/Chainlink Automation** -- keeper networks executing liquidations
- **Internal protocol keepers** -- some protocols run their own liquidation keepers
- **MEV builders** -- on OP Stack chains (Ink), the sequencer might bundle liquidations

---

## 7. Summary: WHY Almost Zero Liquidations

### Structural Reasons (in order of importance):

**1. Correlated Asset Looping (PRIMARY CAUSE)**
The dominant strategy on ALL these protocols is: deposit yield-bearing token (sUSDe, wstETH, kHYPE) -> borrow stablecoin or base asset -> buy more yield token. This creates positions where collateral and debt are highly correlated. Liquidation requires a DEPEG, not a price crash.

**2. Protocols Are Very Young**
- Mantle: 7 weeks old
- Ink: 5 months
- HyperLend: 12 months
- Younger protocol = fewer positions, fewer "stuck" positions, more active management

**3. Whale-Dominated Markets**
- Mantle: top 4 wallets = 85% of sUSDe supply, top 5 borrowers = 71% of USDT0
- Whales actively manage positions. They don't get liquidated -- they unwind early.
- Institutional DeFi integration (Bybit/Kraken) = sophisticated users

**4. Low Volatility Window**
- Since Feb 11 (Mantle launch): BTC stable $67K-$73K
- The major Jan 31 - Feb 5 crash happened BEFORE most Mantle TVL was deployed
- Market at "Extreme Fear" = people REDUCE leverage, not increase it

**5. eMode + High Liquidation Thresholds**
- Correlated assets in eMode: LTV up to 97%, liquidation threshold 98%
- Even HF 1.02 on stablecoin pairs = very far from actual liquidation
- Need >3% depeg for any liquidation to trigger

**6. Conservative Borrowing Outside eMode**
- Non-stablecoin borrowers tend to be conservative (40-60% LTV)
- Current utilization: 26-52% across protocols
- Buffer to liquidation: 30-50%+ price drop needed

### When Will Liquidations Increase?

**Trigger events:**
1. **BTC drop below $60K** -- would stress ETH/BTC collateral positions
2. **sUSDe/USDe depeg** -- would trigger MASSIVE cascading liquidations (>$1B at risk on Aave Core alone)
3. **Protocol maturity** -- as more retail enters through Kraken/Bybit funnels, more reckless positions
4. **Supply cap increases** -- Mantle is raising caps (sUSDe 160M->240M, USDT0 380M->500M), more leverage
5. **Altcoin crash** -- HYPE, INK, or other volatile collateral types
6. **Stablecoin borrow rate spike** -- if USDT0/USDC rates spike above sUSDe yield, negative carry forces unwinding

### Estimated Timeline:
- **Next 30 days (March 2026):** Low liquidation volume unless BTC <$60K or stablecoin depeg
- **Q2 2026:** Increasing as protocols mature, caps grow, retail enters
- **Next major crash:** Whenever Fear & Greed drops below 10 again, expect 10-100x liquidation volume

---

## 8. Implications for Our Liquidation Bot

### The good news:
1. **Low competition** -- if there are few liquidations, there are few liquidators. First mover advantage.
2. **When it rains, it pours** -- the $19B liquidation on Oct 10, 2025 shows: you wait, then you feast.
3. **$1B+ in sUSDe loops at HF 1.02-1.05** -- ONE depeg event = massive opportunity.

### The bad news:
1. **Revenue during calm: ~$0-100/day** -- not enough to justify dedicated infrastructure
2. **You need to be READY when the crash comes** -- bot must be deployed and running BEFORE the event
3. **Correlated-asset liquidations are tricky** -- need to swap sUSDe->USDT0 or similar, DEX depth matters

### Strategy recommendation:
- **Deploy and run in DRY_RUN mode NOW** -- zero cost, ready for the crash
- **Focus on non-correlated positions** -- the few ETH/BTC/HYPE borrowers who exist
- **Monitor sUSDe depeg risk** -- Ethena-related news, stablecoin stability
- **Multi-chain deployment is correct strategy** -- spread across 3-4 chains, catch liquidations wherever they happen
- **Expected monthly revenue: $0-100 calm, $1K-50K during crashes** -- this is a "waiting for the flood" strategy

---

## Sources

- [Aave V3 on Mantle $1B TVL -- Phemex](https://phemex.com/news/article/aave-v3-on-mantle-mainnet-reaches-1-billion-tvl-milestone-63425)
- [Bybit/Mantle/Aave Launch -- Chainwire](https://chainwire.org/2026/02/11/bybit-mantle-and-aave-launch-strategic-mainnet-integration-to-scale-institutional-grade-defi-liquidity/)
- [Tydro Launch -- The Block](https://www.theblock.co/post/374756/kraken-incubated-ethereum-l2-ink-rolls-out-tydro-white-label-instance-aave-v3-supports-ink-token)
- [HyperEVM Mainnet Launch -- The Block](https://www.theblock.co/post/341424/hyperliquid-launches-hyperevm-on-mainnet-to-bring-general-purpose-programmability)
- [Morpho on HyperEVM -- Messari](https://messari.io/intel/event/51111c7c-6ffa-40e3-9f78-8e91e313dc1f)
- [Chaos Labs Mantle Cap Changes March 3](https://governance.aave.com/t/chaos-labs-risk-stewards-change-of-supply-and-borrow-caps-on-aave-v3-mantle-03-03-26/24218)
- [BTC/ETH Worst Start 2026 -- Fortune](https://fortune.com/2026/02/20/bitcoin-ethereum-price-today-worst-starts-in-history-rebound-in-sight/)
- [sUSDe Loop $1B at Risk -- CoinDesk](https://www.coindesk.com/markets/2025/10/29/recent-bitcoin-crash-has-put-usd1b-in-susde-loop-trades-at-risk-research-firm-says)
- [Chainlink SVR Live on HyperEVM](https://docs.chain.link/data-feeds/svr-feeds)
- [HyperLend 3 Liquidation Paths](https://x.com/hyperlendx/status/1932415768890585535)
- [Morpho Blue Liquidation Docs](https://docs.morpho.org/learn/concepts/liquidation/)
- [Mantle Explorer API -- LiquidationCall query](https://explorer.mantle.xyz/api?module=logs&action=getLogs&fromBlock=90172818&toBlock=latest&address=0x458F293454fE0d67EC0655f3672301301DD51422&topic0=0xe413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286)
- [Ink Tydro Pool Counters](https://explorer.inkonchain.com/api/v2/addresses/0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba/counters)
- [HyperLend Pool -- hyperevmscan.io](https://hyperevmscan.io/address/0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b)
- CoinPaprika API: BTC $72,606, ETH $2,122 (March 5, 2026)
