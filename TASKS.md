# Tasks — Multi-Chain Liquidator

## Phase 4: Production Readiness — DONE

### 4.1 — Alchemy RPC ✅
- [x] Mantle: `mantle-mainnet.g.alchemy.com`
- [x] Ink: `ink-mainnet.g.alchemy.com`
- [x] HyperEVM: `hyperliquid-mainnet.g.alchemy.com`

### 4.2 — Wallets + Gas ✅
- [x] Wallet: `0xaa979a7FC2C112448638aB88518231cF82Ec3F3b`
- [x] Mantle: 2.95 MNT | Ink: 0.0015 ETH | HyperEVM: 0.093 HYPE

### 4.3 — Deploy Contracts ✅
- [x] Mantle: `0xF2E6e7F255c46CC2353a8fD1D502f0c1920E1D43` (MantleFlashLoanLiquidator)
- [x] Ink: `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` (InkFlashLoanLiquidator v2 Slipstream)
- [x] HyperEVM: `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` (HyperEVMFlashLoanLiquidator)
- [x] HyperEVM: `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` (MorphoLiquidator)

### 4.4 — Lendle V2 Compatibility ✅
- [x] Borrow event topic V2 vs V3
- [x] eMode skip для V2
- [x] `base_currency_decimals` per-pool (18 для V2 Lendle, 8 для V3)
- [x] Lendle cross-pool flash loan + cross-token (USDT via USDC)

### 4.5 — LIVE Mode ✅
- [x] `DRY_RUN=false`, LIVE mode
- [x] Telegram alerts + web dashboard
- [ ] **Перша ліквідація** — чекаємо crash/depeg

---

## Phase 5: Competitive Edge — DONE

### 5.1 — Morpho Blue Module ✅
### 5.2 — DEX Routing ✅
### 5.3 — Speed Optimization ✅
### 5.4 — Web Dashboard ✅

---

## Phase 6: Advanced Speed + Audit — DONE

### 6.1 — Block Subscription (newHeads) ✅
### 6.2 — Pre-Computed Liquidation TX (deferred — optimization, not critical)
### 6.3 — Parallel Execution ✅
### 6.4 — Odos Aggregator Routing ✅ (Mantle)
### 6.5 — Ink Velodrome Slipstream Fix ✅
### 6.6 — HyperEVM Fee Tier Fixes ✅
### 6.7 — Lendle Cross-Pool Liquidator ✅
### 6.8 — Odos Decimal Fix ✅
### 6.9 — Lendle Token Compatibility ✅
### 6.10 — Parallel Execute Odos ✅
### 6.11 — Rust cross-token executor logic ✅

### 6.12 — Full Audit + Fixes ✅
- [x] **Morpho seized_assets** — compute_max_seized() від debt*LIF, не all collateral
- [x] **V2 close factor** — is_v2 check (Lendle завжди 50%, не dynamic як V3)
- [x] **LendleCrossPoolLiquidator v3** — mode flag dispatch замість params.length>320
  - New contract: `0xf4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91`
  - USDT/USDC fee=100 в constructor
  - Odos router approved
- [x] **parse_revert_reason** — match "reverted: XX" замість contains("XX")
- [x] **USDT/USDC fee tier** — setFeeTier on-chain (verified: FusionX pool@100 exists, @3000 = zero)

---

## Phase 7: Competitive Intelligence + Analytics — TODO

### 7.1 — On-Chain Competitor Liquidation Tracker
**Мета:** Записувати ВСІ ліквідації конкурентів для аналізу: хто ліквідує, як швидко, які пари, gas strategy.

**Реалізація:**
- [ ] Фоновий listener на `LiquidationCall` events для кожного протоколу
  - Lendle: topic `0xe413a321...` на pool `0xCFa5aE7c...`
  - Mantle Aave V3: topic `0xe413a321...` на pool `0x458F2934...`
  - Ink/Tydro: topic `0xe413a321...` на pool `0x2816cf15...`
  - HyperLend: topic `0xe413a321...` на pool `0x00A89d7a...`
  - Morpho: topic `0xa4946ede...` на `0x68e37dE8...`
- [ ] Запис в `data/competitor-liqs.jsonl`:
  ```json
  {
    "timestamp": 1772812345,
    "block": 92345678,
    "chain": "Mantle/Lendle",
    "tx_hash": "0xabc...",
    "liquidator": "0x13418a...",
    "collateral_asset": "0xcDA86A...",
    "collateral_symbol": "mETH",
    "debt_asset": "0xdEAD...",
    "debt_symbol": "WETH",
    "debt_covered": "1500000000000000000",
    "debt_covered_usd": 3200.50,
    "collateral_seized": "1575000000000000000",
    "is_our_bot": false
  }
  ```
- [ ] Парсинг event data: decode indexed topics + data відповідно до V2/V3/Morpho формату
- [ ] Перевірка `liquidator == our_wallet` → `is_our_bot: true`
- [ ] Інтеграція в scan loop: після кожного `incremental_discover`, перевіряти нові LiquidationCall events в тих же блоках
- [ ] **Не потребує окремий RPC call** — можна комбінувати з Borrow event scan (same block range)

**Пріоритет:** P1 — критично для розуміння конкурентного ландшафту.

### 7.2 — Missed Opportunity Detection
**Мета:** Коли наш бот бачить кандидата (HF < 1.0) але не ліквідує — записати що сталось і хто забрав.

**Реалізація:**
- [ ] При `sim_failed` або `cooldown_skip`: зберегти candidate snapshot в `data/missed-opps.jsonl`:
  ```json
  {
    "timestamp": 1772812345,
    "chain": "Mantle/Lendle",
    "target": "0xabc...",
    "health_factor": 0.9823,
    "debt_usd": 5420.0,
    "pair": "USDT -> mETH",
    "reason": "SWAP_FAILED (DEX liquidity issue)",
    "our_action": "sim_failed",
    "estimated_profit": 28.50
  }
  ```
- [ ] Через 30с (next full_scan): перевірити чи позиція ще існує
  - Якщо HF > 1.0 або position gone → хтось ліквідував → cross-reference з `competitor-liqs.jsonl`
  - Записати `competitor_liquidated_by: "0x13418a..."`, `competitor_block`, `latency_blocks`
- [ ] Tracking причин пропуску:
  - `sim_failed` — чому? (HF recovered, swap route, gas)
  - `cooldown_skip` — чи був cooldown justified?
  - `min_profit_skip` — яку суму ми пропустили?
  - `no_contract` — протокол без deployed контракту (HypurrFi)

**Пріоритет:** P1 — без цього не знаємо скільки грошей залишаємо на столі.

### 7.3 — Near-Miss Dashboard Widget
**Мета:** Показувати на дашборді позиції які "ось-ось" стануть liquidatable.

**Реалізація:**
- [ ] Для кожної at-risk позиції (HF < 1.2): обчислити price drop % до HF=1.0
  - Вже є: `compute_liquidation_thresholds()` в monitor.rs
- [ ] Додати на дашборд: "X positions within Y% of liquidation"
- [ ] Alert в Telegram при HF < 1.02 (2% від ліквідації)
- [ ] Показувати estimated profit для near-miss позицій

**Пріоритет:** P2 — nice to have, але не критично.

### 7.4 — Competitor Analysis Report
**Мета:** Автоматичний щоденний/тижневий звіт по конкурентам.

**Реалізація:**
- [ ] Скрипт аналізу `data/competitor-liqs.jsonl`:
  - Top liquidators by volume (address, count, total USD)
  - Average response time (block delta between HF drop and liquidation)
  - Preferred pairs (which collateral/debt combos get liquidated most)
  - Gas strategy (gasPrice distribution)
  - Our win rate vs competitors (коли ми і конкурент бачили того ж кандидата)
- [ ] Вивід в `data/reports/weekly-YYYY-WW.md`
- [ ] Опціонально: Telegram summary

**Пріоритет:** P3 — корисно після набору даних (1-2 тижні).

### 7.5 — Historical Backfill
**Мета:** Завантажити історичні ліквідації для baseline аналізу.

**Реалізація:**
- [ ] One-time scan: всі LiquidationCall events від borrow_start_block до поточного
  - Lendle: ~4019 liqs, від block 91_500_000
  - HyperLend: ~3091 liqs, від block 20_000_000
  - Morpho: ~3555 liqs, від block 4_000_000
- [ ] Записати в `data/historical-liqs-{chain}.jsonl`
- [ ] Аналіз: розподіл ліквідацій по годинах/днях, clustering під час crashes

**Пріоритет:** P2 — разова задача, дає baseline для порівняння.

---

## Live Status (2026-03-06)

**Bot v0.5.3** | LIVE | pm2 `multi-chain-liq` | AWS `3.17.130.7`

| Monitor | Borrowers | At-Risk | Mode | Contract |
|---------|-----------|---------|------|----------|
| Mantle/Aave V3 | 173 | 44 | subscription | `0xF2E6...` ✅ |
| Mantle/Lendle | 37 | 2 | subscription | `0xf4C1...` ✅ v3 |
| Ink/Tydro | 16,514 | 99 | subscription | `0xD472...` ✅ |
| HyperEVM/HyperLend | 658+ | - | polling (init) | `0x17B7...` ✅ |
| HyperEVM/HypurrFi | 1,202+ | - | polling (init) | no contract |
| HyperEVM/Morpho | init | - | polling | `0xD472...` ✅ |

**Dashboard:** https://multi-chain-liquidator.ggdi.vision/

## Known Issues
1. **HyperEVM monitors initializing** — 9-19M блоків scan, скидається при рестарті
2. **HyperEVM no WSS** — `rpc.hyperliquid.xyz` не підтримує WSS
3. **Most protocols dormant** — тільки Lendle має активні ліквідації, решта чекає crash
4. **HypurrFi no contract** — монітор працює, але executor не може виконати (liquidator_contract: None)
5. **Odos тільки Mantle** — Ink/HyperEVM без агрегатор routing
