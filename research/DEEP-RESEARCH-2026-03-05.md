# Deep Research: Autonomous Profit Discovery Protocol

**Дата:** 2026-03-05
**Досліджено:** 30+ напрямків через 5 паралельних research agents
**Контекст:** Solo dev, $500-2000, 1 EC2 (t3.medium / i4i.xlarge), 100-500ms latency

## Зведена таблиця: ТОП-7 живих ідей

| # | Ідея | Capital | Expected ROI | Competition | Feasibility |
|---|------|---------|-------------|-------------|------------|
| 1 | Hyperliquid Funding Rate Arb | $1-2K | 15-30% APR | Medium | HIGH |
| 2 | Morpho/Euler Liquidation (нові chains) | $0-200 | Variable | Low | HIGH |
| 3 | Ink (Kraken L2) Liquidation | $0-200 | Variable | Very Low | HIGH |
| 4 | Liquidation Cascade Prediction | $500-2K | 20-40%/mo* | Medium | MEDIUM |
| 5 | Hyperliquid Custom Vault | $100-500 | 10% profit share | Low entry | MEDIUM |
| 6 | Mantle Aave Liquidation (expand) | $0-200 | Variable | Unknown | HIGH |
| 7 | Governance Buyback Trading | $500-2K | 4-20%/mo* | Medium | MEDIUM |

*Потребує backtest

## KILLED (30+ напрямків)

| Напрямок | Причина смерті |
|----------|---------------|
| 15-min crypto arb (Polymarket) | 2.7 сек windows, sub-100ms боти, latency gate |
| Kalshi trading | US KYC (SSN required) |
| Cross-platform arb (Poly-Kalshi) | US KYC для Kalshi |
| CoW Protocol Solver | $50K+ бонд |
| 1inch Fusion Resolver | KYC/KYB, ліміт 10, великий капітал |
| UniswapX Filler | 90% volume у 2 гравців (SCP + Wintermute) |
| Bridge arbitrage | $10-20K pre-positioned inventory, олігополія |
| Token launch sniping | Rug pull risk, Banana Gun домінує |
| AI Agent tokens | Narrative dying, Virtuals DAW -86%, insider manipulation |
| L2 MEV (Base/OP) | $50K+ capital, $3K/mo infra, institutional game |
| Whale copy trading | 80% copy traders lose, true WR 55-62% |
| RWA tokenization arb | KYC barriers, low DEX liquidity |
| Aevo DEX | Dead ($4.8M 24h volume) |
| Jupiter perps funding | No funding rates (borrow fee system) |
| EigenLayer restaking yield | 3.8-6% APR, institutional game |
| LRT arbitrage | Marginal spreads, liquidity dries up in volatility |
| Scroll | Losing 85% TVL (ether.fi migration) |
| MegaETH | Too early, $40/day DEX volume |
| Aztec | 72 sec block time, no DeFi |
| Berachain | Missed window, $3.35B TVL = high competition |
| Wormhole NTT | No pools = no price discrepancy by design |
| Polymarket market making | $1K too small, sub-10ms latency needed |
| Sports prediction (Polymarket) | Needs separate model, professional betting bots |
| Across Relayer | $20-100/mo with $2K, marginal |
| Stablecoin depeg (Arbitrum) | Max 4 bps, arb bots perfect |
| Cross-L2 arb (Arb-OP) | Spread <= bridge fee |
| Weekend effect | p > 0.05, statistically insignificant |
| Stablecoin signal | Corr 0.21, WR 33% |
| Cascade detection (trading) | Detection 5/5, WR 40% as trading strategy |
| Funding CEX-CEX | 5-10% APR, needs $125K for $1K/mo |

---

## ІДЕЯ #1: Hyperliquid vs CEX Funding Rate Arbitrage

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Hyperliquid funding rates систематично вищі ніж CEX (Binance/Bybit) через bullish bias on-chain трейдерів. Funding кожну годину (vs 8h CEX). Funding cap: 4%/год (vs нижчий cap на Binance).

**Конкретні числа:**
- SOL: ~15.6% APR spread (BitMEX data, H1 2025)
- AVAX: ~15.7% APR spread
- ETH: 28.1% spread зафіксовано (грудень 2024)
- З 2-3x leverage: 25-30%+ APR delta-neutral

**Чому існує:** On-chain трейдери Hyperliquid — переважно retail з bullish bias. Вони систематично переплачують за long. Funding rates self-correct через quadratic model, але приплив нових retail тримає rates високими.

**Чому досі не закрита:** Потребує капітал на 2 біржах одночасно + моніторинг + auto-rebalancing. Retail не вміє, інституціонали фокусуються на більших venue.

### 2. КОНКУРЕНТИ
- **Hummingbot vault users** — готова стратегія funding_rate_arb, але більшість використовують manual setup
- **Dedicated arb funds** — фокусуються на Binance-OKX-Bybit, менше на Hyperliquid
- **Наш gap:** Hyperliquid ще не захоплений інституціоналами як GMX/dYdX. First-mover на менш конкурентному venue.

### 3. EDGE
**Формула:** Long на дешевшій біржі (Binance/Bybit) + Short на дорожчій (Hyperliquid). Delta-neutral. Збираєш funding spread кожну годину.

**Очікуваний ROI:**
- Conservative (без leverage): 15% APR = $150/рік на $1K = $12.50/міс
- Moderate (2x leverage): 25% APR = $250/рік на $1K = $20.83/міс
- З Pendle Boros lock: 5.98-11.4% Fixed APR (guaranteed, no flip risk)

**Від чого залежить:** Стабільність funding rate spread. Якщо Hyperliquid retail bias зникне — spread зменшиться.

### 4. PRE-MORTEM
- **Funding rate flip (50%)** — rates можуть flip-нути, особливо в bear market. Pendle Boros частково вирішує.
- **Hyperliquid liquidation risk (20%)** — якщо ціна різко рухається і margin не вистачає на одній стороні
- **Execution risk (15%)** — відкриття/закриття позицій на 2 біржах не атомарне, slippage

### 5. МІНІМАЛЬНИЙ ТЕСТ (1-2 дні)
**День 1:** Зібрати 24h funding rate data з `app.hyperliquid.xyz/fundingComparison` + Binance API. Порівняти top-10 pairs. Рахувати net spread після fees.
**День 2:** Paper trade: відкрити delta-neutral позиції на 3 парах з найвищим spread. Моніторити 24h P&L.
**Success metric:** Net spread > 10% APR після fees на 3+ парах протягом 24h
**Kill metric:** Spread flip-нувся 3+ рази за 24h або net < 5% APR

**Інструменти:** Hummingbot (готова стратегія), CoinGlass FrArbitrage, LorisTools

---

## ІДЕЯ #2: Liquidation Bot на нових EVM chains (Morpho/Euler/HyperLend)

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Нові EVM chains (HyperEVM, Base, Sonic, Ink) мають lending протоколи з $380M-$1B TVL, але молоду екосистему ліквідаторів. Flash loans доступні — $0 капіталу.

**Конкретні numbers:**
- HyperLend (HyperEVM): $380M TVL, flash loans, молодий протокол
- Morpho Blue: $3.9B TVL, deployed на HyperEVM, Base, Monad, Katana, Unichain
- Euler V2: $1B+ TVL (850% growth), deployed на Sonic, Berachain, Swell, BOB
- Ink/Tydro (Aave V3): $380M TVL, Kraken-backed, 15 місяців

**Чому існує:** Нові chains = мало ліквідаторів. Morpho і Euler мають open-source liquidation bots. First mover на новому chain отримує всі ліквідації.

**Чому досі не закрита:** Кожен новий chain потребує окремого deployment + RPC + gas tokens. Більшість ліквідаторів сфокусовані на Ethereum/Arbitrum.

### 2. КОНКУРЕНТИ
- На Ethereum Morpho/Euler — жорстка конкуренція
- **На HyperEVM/Sonic/Ink — НЕ ЗНАЙДЕНО даних про активних ліквідаторів**
- Наш gap: швидкий deploy open-source ботів на нові chains ПЕРШИМИ

### 3. EDGE
**Механізм:** Flash loan → liquidate under-collateralized position → swap collateral → repay loan → keep profit. $0 capital risk (flash loan revert якщо збитковий).

**Очікуваний ROI:** Залежить від market volatility. На зрілих ринках (Aave Arbitrum): $5-250/day. На нових chains з $380M+ TVL і 0 конкуренції — potentially вищий.

**Від чого залежить:** Volatility (більше = більше ліквідацій), TVL (більше = більші позиції), конкуренція (менше = більше прибутку).

### 4. PRE-MORTEM
- **Конкуренція прийде швидко (60%)** — як тільки перший прибуток видимий, інші ліквідатори прийдуть
- **Flash loan infrastructure (25%)** — не всі нові chains мають надійні flash loan провайдери
- **RPC/infrastructure (15%)** — нові chains можуть мати нестабільні RPC

### 5. МІНІМАЛЬНИЙ ТЕСТ (1-2 дні)
**День 1:**
- Clone Morpho liquidation bot (github.com/morpho-org/morpho-blue-liquidation-bot)
- Clone Euler liquidation bot (github.com/euler-xyz/liquidation-bot-v2)
- Check HyperEVM RPC endpoints, gas costs, flash loan availability
- List all Morpho markets on HyperEVM з TVL + health factors

**День 2:**
- Deploy bot на HyperEVM testnet/mainnet
- Monitor: скільки позицій з HF < 1.1? Скільки ліквідацій за 24h? Хто їх виконує?
- Порівняти з Arbitrum Aave (де ми вже маємо дані)

**Success metric:** 5+ liquidatable positions per day на HyperEVM/Ink, 0-2 competing bots
**Kill metric:** 0 liquidatable positions або 5+ competing bots вже активні

---

## ІДЕЯ #3: Ink (Kraken L2) — Aave V3 Liquidation

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Ink — L2 від Kraken (15 місяців), Tydro = Aave V3 white-label з $380M TVL. Deposits зростають з $215M до $301M за кілька днів. Конкуренція MEV ботів НЕ задокументована.

**Чому існує:** Молода екосистема, Kraken IPO в Q1 2026 стимулює зростання. Нові юзери → нові позиції → нові ліквідації. Мало специалізованих ліквідаторів на Kraken L2.

### 2. КОНКУРЕНТИ
- **Не знайдено задокументованих liquidation ботів на Ink**
- Kraken може мати внутрішній liquidation engine
- Aave built-in liquidation infra може привабити pros

**Наш gap:** Юзер вже має працюючий Aave V3 liquidation bot (v3 + F1 на Arbitrum). Адаптація під Ink = deployment existing codebase.

### 3. EDGE
**Механізм:** Той самий Aave V3 liquidation flow як на Arbitrum. Flash loans + Uniswap V3 (або FusionX) swap.

**Від чого залежить:** Наявність DEX ліквідності на Ink для свопу collateral, gas costs, flash loan availability.

### 4. PRE-MORTEM
- **Немає flash loans на Ink (40%)** — потрібно перевірити
- **Недостатня DEX ліквідність (30%)** — свопи можуть мати високий slippage
- **Kraken внутрішній liquidator (20%)** — можуть мати власний бот

### 5. МІНІМАЛЬНИЙ ТЕСТ (1 день)
- Check Ink RPC endpoints
- Verify Tydro/Aave V3 contract addresses on Ink
- Check flash loan availability (Aave flash loans built-in)
- Check DEX availability (which DEX, liquidity depth)
- List all positions with HF < 1.2
- Count active liquidators (recent Liquidation events)

**Success metric:** Flash loans available, DEX liquidity > $1M, < 3 competing liquidators
**Kill metric:** No flash loans, < $100K DEX liquidity, or Kraken internal liquidator dominating

---

## ІДЕЯ #4: Liquidation Cascade Prediction Model

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Leveraged crypto market має передбачувані cascade patterns. Сигнали (OI зміни, funding extremes > 0.0182%, liquidation cluster density) з'являються за 7-20 днів до піків ліквідацій. AI моделі показують 55-65% accuracy.

**Конкретні events:**
- October 10-11, 2025: $19B ліквідацій за 24h (найбільший в історії)
- November 2025: $2B ліквідацій, 396K трейдерів
- В симуляціях AI виявив October cascade за 112 хвилин до початку

### 2. КОНКУРЕНТИ
- CoinGlass heatmaps — publicly available, використовуються manual трейдерами
- Dedicated quant funds — мають кращі моделі, але фокус на HFT, не на cascade prediction
- **Мало хто автоматизує cascade prediction → auto-trade**

### 3. EDGE
**Формула:** Коли Funding Rate > 0.0182% + rising OI + liquidation clusters нижче ціни = high cascade probability → short з stop loss.

**Очікуваний ROI:** При 60% WR і 2:1 RR, 3-5 signals/місяць: ~20-40% monthly (ПОТРЕБУЄ BACKTEST)

**Від чого залежить:** Prediction accuracy, число сигналів, R:R ratio.

### 4. PRE-MORTEM
- **Overfitting (50%)** — модель може підлаштуватись під historical cascades але не передбачити нові
- **Зміна ринкового режиму (30%)** — в bull market cascades рідші, в bear — частіші
- **Малий sample size (20%)** — major cascades трапляються 5-10 разів на рік

### 5. МІНІМАЛЬНИЙ ТЕСТ (2 дні)
**День 1:**
- Зібрати historical cascade events 2024-2026 з CoinGlass API (liquidation + OI + funding)
- Визначити features: OI change rate, funding extremes, cluster density, volume spikes
- Minimum 20 cascade events для backtesting

**День 2:**
- Simple rule-based model (не ML): funding > threshold + OI rising + cluster proximity
- Walk-forward backtest на 60+ днів
- Calculate: WR, Sharpe, MaxDD, Profit Factor

**Success metric:** WR > 58%, Sharpe > 1.3, Profit Factor > 1.5 on walk-forward
**Kill metric:** WR < 55%, Sharpe < 1.0, або < 2 signals per month

---

## ІДЕЯ #5: Hyperliquid Custom Vault (Funding Arb Strategy)

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Hyperliquid дозволяє будь-кому створити vault (мін. 100 USDC). Vault leader отримує 10% від прибутку депозиторів. Це спосіб масштабувати стратегію з чужим капіталом.

**Модель:** Запустити vault з funding arb стратегією → показати track record → залучити depositors → 10% profit share.

### 2. КОНКУРЕНТИ
- AceVault Hyper01: TVL $14.33M, APR 127% за кращий місяць
- HLP: TVL $380M, ~22% APR lifetime
- Growi HF: найкращий risk-adjusted return

**Наш gap:** Більшість vault strategies — directional або market making. Delta-neutral funding arb vault = lower volatility = attractive для risk-averse depositors.

### 3. EDGE
**Механізм:** 100 USDC start → run funding arb → show 1-month track record → attract depositors → 10% profit share scales.

**Сценарій:** При $100K deposited і 20% APR стратегії = $20K прибуток depositors → $2K profit share/рік.

### 4. PRE-MORTEM
- **5% lock requirement (40%)** — при vault growth до $100K потрібно $5K locked capital
- **Reputation risk (30%)** — одна погана trade = depositors leave
- **Strategy capacity (20%)** — funding arb може не масштабуватись на великі суми

### 5. МІНІМАЛЬНИЙ ТЕСТ (1 день)
- Створити vault на Hyperliquid з 100 USDC
- Запустити manual funding arb на 1 парі (SOL або AVAX)
- Трекати performance 7 днів
- Якщо позитивний — автоматизувати через Hummingbot

**Success metric:** Positive P&L за 7 днів, > 10% APR
**Kill metric:** Negative P&L або funding flip

---

## ІДЕЯ #6: Mantle Aave V3 Liquidation (Expand Existing)

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** Mantle Aave V3 market досяг $1B за 19 днів. TVL +66% weekly. Юзер вже має deployed liquidation contract (`0xF2E6e7F255c46CC2353a8fD1D502f0c1920E1D43`).

**Чому зараз:** Вибухове зростання TVL = нові позиції від incentive programs = більше under-collateralized позицій при volatility.

### 2. КОНКУРЕНТИ
Невідомо. Потрібно перевірити on-chain.

### 3. EDGE
Existing infrastructure — контракт deployed, потрібно тільки підключити monitoring бот.

### 4. PRE-MORTEM
- **Конкуренція вже є (50%)** — $1B TVL може мати активних ліквідаторів
- **MethLab liquidation-free (20%)** — деякі позиції можуть бути захищені
- **DEX liquidity (15%)** — Mantle DEX може мати недостатню ліквідність для свопів

### 5. МІНІМАЛЬНИЙ ТЕСТ (пів дня)
- SSH на AWS → перевірити pm2 status mantle-monitor
- Переглянути останні Liquidation events на Mantle Aave
- Count competing liquidators
- Check DEX liquidity depth

**Success metric:** < 5 active liquidators, 10+ liquidatable positions/day
**Kill metric:** 10+ competing liquidators або 0 liquidatable positions

---

## ІДЕЯ #7: Governance Buyback Monitor + Auto-Trade

### 1. НЕЕФЕКТИВНІСТЬ
**Що саме:** DeFi протоколи витратили $1.4B на token buybacks у 2025. Buyback announcements дають +2-5% price impact. Schedules передбачувані (Aave: weekly $250K-1.75M, Hyperliquid: continuous 97% fees, Optimism: monthly).

**Чому існує:** Governance proposals = public data, але мало хто автоматизує monitoring → auto-buy workflow.

### 2. КОНКУРЕНТИ
- Manual governance watchers (Discord/forum readers)
- **Не знайдено automated governance trading bots**
- Наш gap: повна автоматизація від proposal detection до trade execution

### 3. EDGE
**Механізм:** Monitor governance forums (Snapshot + on-chain) → detect buyback proposal → auto-buy token при proposal passing threshold → sell after price impact.

**Очікуваний ROI:** 2-5% per event x 2-4 events/місяць = 4-20% monthly (НЕ ПІДТВЕРДЖЕНО)

### 4. PRE-MORTEM
- **Price impact already priced in (50%)** — ринок може реагувати на proposal ще до vote
- **Execution timing (25%)** — коли саме купувати? При proposal creation? При vote passing?
- **Sample size (15%)** — major buyback proposals = 10-20/рік across all protocols

### 5. МІНІМАЛЬНИЙ ТЕСТ (1 день)
- Backtest: зібрати всі buyback proposals 2025-2026 (Aave, OP, JUP, RAY, HYPE)
- Для кожного: ціна при proposal creation, ціна при vote passing, ціна +7d
- Calculate: WR, avg return, Sharpe

**Success metric:** WR > 60%, avg return > 3% per event, 20+ datapoints
**Kill metric:** WR < 55% або avg return < 1.5% або < 10 datapoints

---

## EXECUTION PLAN — Пріоритизація

### Tier 1: Негайно (паралельно, цей тиждень)

**1A. Hyperliquid Funding Rate Data Collection** (2 години)
- Зібрати 24h+ funding rates з Hyperliquid API + Binance API
- Top-10 pairs по spread
- Net spread після fees
- Порівняти з вже зібраними GMX V2 vs CEX даними

**1B. Morpho/Euler Bot Deployment Research** (2 години)
- Clone repos, перевірити HyperEVM compatibility
- Check flash loan availability
- List Morpho markets on HyperEVM

**1C. Mantle Aave Liquidation Check** (30 хв)
- SSH → перевірити mantle-monitor
- Recent Liquidation events count
- Competing liquidators count

**1D. Ink Liquidation Feasibility** (1 година)
- Ink RPC endpoints
- Tydro/Aave V3 contract addresses
- Flash loan + DEX availability

### Tier 2: Після Tier 1 results (наступний тиждень)

**2A.** Deploy winning liquidation bot(s) на new chain(s)
**2B.** Start Hyperliquid funding arb paper trade (Hummingbot)
**2C.** Cascade prediction model backtest (if Tier 1 tasks show limited opportunity)

### Tier 3: Long-term (2+ тижні)

**3A.** Hyperliquid vault creation (after 7d track record)
**3B.** Governance monitor automation (after backtest)
**3C.** Pendle Boros fixed yield lock (after funding arb validated)

---

## Sources (50+ web searches across 5 agents)

### Prediction Markets
- Yahoo Finance: Arbitrage Bots Dominate Polymarket
- Phemex: Trading Bots Earn $5-10k Daily
- The Block: Polymarket Taker Fees
- Medium: Beyond Simple Arbitrage 2026
- Kalshi API Documentation (docs.kalshi.com)
- Azuro Protocol (gem.azuro.org)
- Overtime Markets (BookieBuzz)

### Solver/Intent Architecture
- UniswapX Docs (docs.uniswap.org)
- arxiv: Execution Welfare Across Solver-based DEXes
- CoW Protocol CIP-44
- Across Protocol Relayer (docs.across.to)
- ERC-7683 (erc7683.org)
- Flashbots: MEV and Limits of Scaling

### Perp DEX & Yield
- BitMEX: Harvest Funding on Hyperliquid
- Hummingbot: Funding Rate Arbitrage
- Pendle Boros: Cross-Exchange Funding Rate Arbitrage
- Morpho Blue Liquidation Bot (GitHub)
- Euler V2 Liquidation Bot (GitHub)
- Drift Protocol Docs (docs.drift.trade)
- CoinGlass FrArbitrage

### Cross-chain/L2
- The Block: 2026 Layer 2 Outlook
- Ink/Tydro (Blocmates, The Block)
- Mantle + Aave $1B (PR Newswire)
- arxiv: Cross-Chain Arbitrage
- MegaETH (The Defiant)
- RWA.io: Tokenization Trends

### On-chain Data
- CoinDesk: Aave $50M Buyback
- DWF Labs: Token Buybacks in Web3
- SSRN: October 2025 Liquidation Cascade
- BeinCrypto: Predict Bitcoin Crash
- CoinGlass: Liquidation Heatmap
- Gate.io: Derivatives Market Signals 2026
