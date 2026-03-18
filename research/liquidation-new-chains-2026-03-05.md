# Deep Research: Liquidation на нових EVM Chains

**Дата:** 2026-03-05
**Статус:** ДОСЛІДЖЕНО — рішення нижче
**Контекст:** Solo dev, existing Aave V3 liquidation bot (F1 + v3) на Arbitrum, Mantle monitor deployed

## TL;DR Verdict

| Chain/Protocol | TVL | Flash Loans | Competition | Verdict |
|---|---|---|---|---|
| **Ink/Tydro (Aave V3)** | ~$446M (L2BEAT: $527M) | YES (5 bps) | **5 ботів, ~2 ліквідації/день** | **GO — low effort, low competition** |
| **HyperEVM/Morpho** | $431M supply, $119M borrowed | YES (HyperLend) | Unknown + **SVR live** | MEDIUM — SVR + нова кодова база |
| **HyperEVM/HyperLend** | **$650M+ supply**, $172M borrowed | YES (4 bps) | Unknown + **SVR live** | **MEDIUM — Aave V3 FORK! (was LOW)** |
| **Euler V2 (Sonic)** | **$1.01M** (DOWN від $53M) | No native | - | **KILLED — TVL обвалився -98%** |
| **Euler V2 (Berachain)** | **$1.51M** (DOWN від $22M) | No native | - | **KILLED — TVL обвалився -93%** |
| **Euler V2 (BOB)** | **$5M** (DOWN від $23M) | No native | - | **KILLED — TVL мізерний** |

**Рекомендація: Ink/Tydro — перший deploy (1-2 дні). HyperEVM/Morpho — другий (після валідації SVR impact та вивчення протоколу).**

---

## 1. INK / TYDRO (AAVE V3) — GO

### Chain Infrastructure (on-chain verified)

| Параметр | Значення | Джерело |
|---|---|---|
| Chain ID | 57073 (0xdef1) | on-chain `eth_chainId` |
| Stack | OP Stack (Optimism Superchain) via Gelato RaaS | docs |
| Block time | 1 секунда | on-chain verified |
| Gas price | 0.01 gwei (slow/avg/fast identical) | on-chain |
| Gas token | ETH | |
| Gas cost per liquidation | **$0.00 — $0.19** | on-chain TX analysis |
| Total addresses | 8,107,928 | explorer |
| Daily transactions | ~708,000 | explorer |
| Network utilization | 6% | L2BEAT |
| TVL (L2BEAT) | $527.6M ($37M canonical + $490M bridged) | L2BEAT |
| TVL (DefiLlama) | ~$400M+ | DefiLlama |
| Sequencer | Centralized (Gelato), FCFS ordering | docs |
| Challenge period | 7 days | L2BEAT |

### RPC Endpoints

**Public (free):**
- `https://rpc-gel.inkonchain.com` (Gelato, primary + WSS)
- `https://rpc-qnd.inkonchain.com` (QuickNode + WSS)
- `https://ink.drpc.org` (dRPC + WSS)
- `https://ink-public.nodies.app`

**Paid:**
- Alchemy: `https://ink-mainnet.g.alchemy.com/v2/<key>`
- QuickNode, Tenderly also available

**Explorer:** `https://explorer.inkonchain.com` (Blockscout)

### Tydro (Aave V3 on Ink) — Повні адреси

**Pool 1 (MAIN) — PoolAddressesProvider: `0x4172E6aAEC070ACB31aaCE343A58c93E4C70f44D`**

| Contract | Address |
|----------|---------|
| **Pool (L2Pool)** | `0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba` |
| **PoolDataProvider** | `0x96086C25d13943C80Ff9a19791a40Df6aFc08328` |
| **AaveOracle** | `0x4758213271BFdC72224A7a8742dC865fC97756e1` |
| **ACLManager** | `0x86e2938dae289763d4e09a7e42c5ccca62cf9809` |
| **PoolConfigurator** | `0x4f221e5c0b7103f7e3291e10097de6d9e3bfc02d` |
| Pool Implementation | `0x7F6036c2A9244E766F9CcD8dE78D8f79F80e5408` (L2PoolInstance) |

**Pool 2 (secondary):** `0x70c88e98578bc521a799de0b1c65a2b12d6f99e4` (6 reserves, 0 liquidations)

**Flash Loans:** YES, built-in. **Premium: 5 bps (0.05%)**.
**Oracle:** Chaos Labs Edge (НЕ Chainlink!) + CAPO для correlated assets → **SVR НЕ впливає!**

### 12 Reserve Assets (on-chain verified, Pool 1)

| # | Token | Address |
|---|-------|---------|
| 0 | WETH | `0x4200000000000000000000000000000000000006` |
| 1 | kBTC | `0x73e0c0d45e048d25fc26fa3159b0aa04bfa4db98` |
| 2 | USDT0 | `0x0200c29006150606b650577bbe7b6248f58470c1` |
| 3 | USDG | `0xe343167631d89b6ffc58b88d6b7fb0228795491d` |
| 4 | GHO | `0xfc421ad3c883bf9e7c4f42de845c4e4405799e73` |
| 5 | USDC | `0x2d270e6886d130d724215a266106e6832161eaed` |
| 6 | weETH | `0xa3d68b74bf0528fdd07263c60d6488749044914b` |
| 7 | wrsETH | `0x9f0a74a92287e323eb95c1cd9ecdbeb0e397cae4` |
| 8 | ezETH | `0x2416092f143378750bb29b79ed961ab195cceea5` |
| 9 | sUSDe | `0x211cc4dd073734da055fbf44a2b4667d5e5fe5d2` |
| 10 | USDe | `0x5d3a1ff2b6bab83b63cd9ad0787074081a52ef34` |
| 11 | SolvBTC | `0xae4efbc7736f963982aacb17efa37fcbab924cb3` |

### DEX Infrastructure on Ink

**Primary DEX: Velodrome Slipstream (Concentrated Liquidity)**
- CLFactory: `0x04625B046C69577EfC40e6c0Bb83CDBAfab5a55F`
- Multiple SwapRouter contracts deployed
- 50+ ICHI vault liquidity positions
- Key pairs: WETH/USDT0, WETH/USDC.e, kBTC/WETH, kBTC/USDG

**DEX Aggregator: SuperSwap** (superswap.ink)
- 0.3% swap fee
- **$5.15M daily volume** (entire Ink DEX)

**NOT present:** 1inch V6, 0x Exchange

**КРИТИЧНО:** $5.15M/день total DEX volume = тонка ліквідність. Ліквідації >$50K матимуть significant slippage. Потрібно перевірити pool-level depth.

### Конкуренція (ON-CHAIN VERIFIED)

**23 LiquidationCall events за 11.5 днів = ~2 ліквідації/день**

**5 унікальних ліквідаторів:**

| Address | Count | Method | Gas Price |
|---------|-------|--------|-----------|
| `0xef6e43...bf7f0e` | 6 | Direct Pool call | 4K-40K wei |
| `0x59c818...d86264e` | 2 | Via `0xf057...` | ~9.5M wei |
| `0xab5c1a...2e92` | 2 | Via `0xf057...` | ~37-67M wei |
| `0x8194486e...bc61` | 1 | Via `0xf057...` | ~156M wei |
| `0x4d2858...e7a2` | 1 | Via `0xf057...` | ~711K wei |

**`0xf0570ec4...a00004`** — спільний liquidation bot contract (unverified), використовується 4 з 5 ліквідаторів. Ймовірно комерційний бот.

**MEV protection:** Gelato sequencer (FCFS). Mempool NOT public. Latency matters.

### Чому Ink = GO

1. **Aave V3 = 95% code reuse.** Той самий Pool ABI, flash loans, oracle interface. Зміни: адреси + DEX router.
2. **Flash loans 5 bps** — cheap. $0 capital.
3. **Gas $0.00-$0.19** — навіть failed TX не коштує нічого.
4. **Chaos Labs oracle (NOT Chainlink)** — SVR не впливає! Весь liquidation profit для нас.
5. **5 конкурентів** — це LOW для $446M TVL. На Arbitrum їх сотні. 4 з 5 використовують один контракт = ймовірно один сервіс.
6. **~2 ліквідації/день** — може здатись мало, але це low volatility період. При market dump → десятки/сотні.
7. **$527M TVL росте** — Kraken IPO, нові юзери, нові leveraged позиції.

### Devil's Advocate: Чому це може НЕ спрацювати

1. **$5.15M/день DEX volume = головний ризик.** Якщо ліквідація $50K kBTC → USDC, slippage може з'їсти весь profit. Потрібна перевірка pool depth ПЕРЕД deploy.
2. **~2 ліквідації/день = низька стеля доходу.** При avg $50 profit/liquidation = $100/день = $3K/місяць в кращому випадку. Реалістично: $30-100/день з peaks при volatility.
3. **Chaos Labs oracle = unknown update timing.** Якщо oracle лагає — ліквідації запізнюються. Якщо оновлюється швидко — конкуренція за speed.
4. **Velodrome ≠ Uniswap V3.** Наш контракт має Uniswap V3 SwapRouter. Velodrome Slipstream може мати інший ABI. Потрібна адаптація або `swapData` + `swapRouter` flow (вже підтримується контрактом).
5. **Gelato sequencer = single point.** Якщо Gelato має фаворитів або priority lane — ми програємо.

### Deployment Plan

**Day 1 — Setup:**
1. ~~Знайти DEX router~~ → Velodrome Slipstream SwapRouter (знайти exact address)
2. Перевірити DEX pool depth для WETH/USDC, kBTC/USDC пар
3. Створити `liquidator/ink/` directory
4. Fork `mantle/src/config.ts` → `ink/src/config.ts` з Ink адресами
5. Адаптувати FlashLoanLiquidator → або deploy з `swapData` routing (вже підтримується), або нові Constants
6. Deploy контракт через Foundry (`forge create --rpc-url https://rpc-gel.inkonchain.com`)
7. Verify на explorer.inkonchain.com

**Day 2 — Monitor:**
8. Scan borrowers через PoolDataProvider
9. Calculate HF для всіх позицій
10. DRY_RUN mode 24h → log candidates
11. Decision: GO LIVE або KILL

**Kill criteria:**
- DEX depth < $100K для основних пар → KILL (не можна свопити collateral)
- 0 позицій HF < 1.2 через 48h → PAUSE (wait for volatility)
- Velodrome SwapRouter ABI incompatible і немає workaround → DELAY

---

## 2. HYPEREVМ / MORPHO BLUE — MEDIUM PRIORITY

### Chain Infrastructure

| Параметр | Значення |
|---|---|
| Chain ID | 999 (mainnet), 998 (testnet) |
| RPC | `https://rpc.hyperliquid.xyz/evm` (public) + Chainstack, Alchemy, dRPC |
| Gas token | HYPE (native). wHYPE: `0x5555555555555555555555555555555555555555` |
| Gas cost | ~0.111 Gwei base, ~0.147 Gwei priority. < $0.01/tx |
| Block architecture | DUAL: fast (~1s, 2M gas) + slow (~60s, 30M gas) |
| EVM version | Cancun (without blobs) |
| getLogs range | Max 1000 blocks |
| TVL chain | ~$2B+ |
| Addresses | 940K+ |
| Weekly DEX volume | ~$1B |

### Morpho Blue on HyperEVM

| Параметр | Значення |
|---|---|
| Morpho TVL | $431M supply, $119M borrowed |
| Morpho Core | `0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb` (universal) |
| Morpho status | #3 dApp on HyperEVM |
| Liquidation bonus | ~5% (LIF varies by LLTV, from 1.5% to 15%) |
| Liquidation coverage | Up to 100% of debt |
| Pre-liquidation | Available if configured |
| **Flash loans needed** | **НІ** — Morpho має callback mechanism |
| **Bot config ready** | **ТАК** — chain 999 вже в config.ts бота |

**Whitelisted Vaults (з config.ts бота):**
- Felix USDC: `0x8A862fD6c12f9ad34C9c2ff45AB2b6712e8CEa27`
- Felix USDT: `0xFc5126377F0efc0041C0969Ef9BA903Ce67d151e`
- Felix HYPE: `0x2900ABd73631b2f60747e687095537B673c06A76`

**Profit check:** ВИМКНЕНИЙ в config → бот виконує ВСІ ліквідації без перевірки прибутковості

### HyperEVM Read Precompiles (УНІКАЛЬНО)

Contracts can read HyperCore order book prices directly:
- Precompile at `0x0000000000000000000000000000000000000800+`
- Gas: `2000 + 65 * (input_len + output_len)`
- CoreWriter at `0x3333333333333333333333333333333333333333` — write to HyperCore (orders, transfers)
- **Order actions are delayed several seconds** (MEV protection)

### Chainlink SVR — Impact Assessment

| Factor | Impact |
|---|---|
| SVR live on HyperEVM | YES |
| SVR affects Morpho | **UNKNOWN** — depends on oracle type |
| If Morpho uses Chainlink | 65% OEV to protocol, 35% to Chainlink |
| If Morpho uses HyperCore precompile | SVR does NOT apply |
| **ГІПОТЕЗА** | Morpho likely uses Chainlink on HyperEVM (their standard setup) |

### Morpho Blue Liquidation Bot

```
morpho-org/morpho-blue-liquidation-bot
├── Stack: Node.js + pnpm + Foundry
├── Detection: RPC-based scanning (NO events, NO subgraph)
├── Executor: Custom gated contract (deploy per chain)
├── Liquidity: UniswapV3/V4, 1inch, ERC4626, Pendle
├── Config: apps/config/config.ts (per-chain)
├── Env: RPC_URL_<chainId>, LIQUIDATION_PRIVATE_KEY_<chainId>
└── Deploy: `pnpm deploy:executor`
```

### DEXes on HyperEVM

| DEX | TVL | Type |
|---|---|---|
| HyperSwap | $80M | AMM (first native DEX) |
| KittenSwap | ~$2M | ve(3,3) AMM |
| Hypertrade | Aggregator | Routes across HyperSwap, KittenSwap, HL Spot |
| GlueX | Aggregator | Cross-pool routing |
| LiquidSwap | Aggregator | Best rates from multiple sources |

### Pre-mortem

- **SVR capture (40%)** — якщо Morpho на HyperEVM використовує Chainlink feeds
- **New protocol learning curve (20%)** — Morpho != Aave, different bot, different mechanics
- **getLogs 1000 block limit (15%)** — monitoring потребує pagination або event-based approach
- **Dual-block complexity (15%)** — transactions routed to different mempools
- **MEV bots already exist (10%)** — documented arb bot 0xe2c...8888

### Next Steps (AFTER Ink deployment)

1. Clone morpho-blue-liquidation-bot, run locally
2. Determine Morpho oracle on HyperEVM (Chainlink vs HyperCore)
3. Configure for chain 999
4. Deploy executor contract
5. Monitor 48h in DRY_RUN
6. Assess SVR impact on liquidation profit

---

## 3. HYPEREVМ / HYPERLEND — MEDIUM PRIORITY (ОНОВЛЕНО!)

**КРИТИЧНЕ ОНОВЛЕННЯ:** HyperLend = **Aave V3 FORK** (той самий interface: Pool, PoolConfigurator, hTokens). Наш FlashLoanLiquidator.sol ПІДХОДИТЬ!

### Факти (agent-verified)

| Параметр | Значення |
|---|---|
| TVL | **$650M+** (kHYPE market alone $306M) |
| Active loans | $172M+ |
| Flash loans | YES, **0.04% fee** |
| Protocol type | **Aave V3 Fork** (same ABI!) |
| Pool | `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` |
| PoolAddressesProvider | `0x72c98246a98bFe64022a3190e7710E157497170C` |
| Oracle | `0xC9Fb4fbE842d57EAc1dF3e641a281827493A630e` |
| ProtocolDataProvider | `0x5481bf8d3946E6A3168640c1D7523eB59F055a29` |
| ACLManager | `0x10914Ee2C2dd3F3dEF9EFFB75906CA067700a04A` |
| UiPoolDataProvider | `0x3Bb92CF81E38484183cc96a4Fb8fBd2d73535807` |

### Assets (15+)

wHYPE, wstHYPE, kHYPE, UBTC, UETH, USOL, USDe, sUSDe, USDT0, USDC, USDHL, USDH, USR, beHYPE, PT-kHYPE (various maturities)

### Oracles

- Pyth HYPE/USD: `0xdE8d22d022261c9Fb4b5338DA8ceFb029175D0F5`
- Chainlink HYPE/USD: `0xf1CeE6FD8464a059B6d2F3e8A0754cD530e78c17`
- Chainlink USDT/USD: `0x5d5EE47c6bCf6B05B2a3F65c4e37312Dc978d30D`

### Unique Edge: HyperCore Liquidation Path

HyperLend підтримує 3 шляхи ліквідації:
1. **Flash Loan + DEX** (standard Aave pattern — наш підхід)
2. **Cross-chain через HyperCore** — bridge collateral на L1 orderbook (глибша ліквідність, ~$5 vs ~$27 + MEV на ETH)
3. **SpotSend** — bridge tokens через spotSend mechanism

### Verdict: MEDIUM PRIORITY (was LOW)

**Причини підвищення:**
1. **Aave V3 fork** → FlashLoanLiquidator.sol підходить з мінімальними змінами!
2. **$650M+ TVL** — більше ніж Ink
3. **Flash loans 0.04%** — дешевше ніж Ink (0.05%)
4. **15+ assets** — більше volatile пар
5. **HyperCore liquidation path** — унікальний edge (ордербук > AMM для великих свопів)

**Залишаються проблеми:**
1. Chainlink SVR LIVE → може забирати OEV
2. Dual-block architecture → складніше monitoring
3. getLogs max 1000 blocks → pagination
4. No open-source HyperLend-specific liquidation bot

---

## 4. EULER V2 — KILLED

### Факти (DefiLlama API, March 2026)

| Chain | TVL зараз | TVL peak (Q1 2025) | Падіння |
|---|---|---|---|
| Sonic | **$1.01M** | $53.54M | **-98.1%** |
| Berachain | **$1.51M** | $22.08M | **-93.2%** |
| BOB | **$5M** | $23.47M | **-78.7%** |
| Base | ~$100M | - | Stable |
| Ethereum | $732M | - | Stable (competitive) |

### Bot Assessment

| Aspect | Details |
|---|---|
| Repo | `euler-xyz/liquidation-bot-v2` |
| Stack | Python 3 + Flask + web3.py + Foundry |
| Default config | Only Mainnet (1) + Arbitrum (42161) |
| Flash loans | NOT supported |
| Detection | AccountStatusCheck events from EVC |
| Liquidation | Dutch auction-style |
| Chain addresses | 20+ chains deployed (incl. 146/Sonic, 80094/Berachain, 999/HyperEVM, 57073/Ink) |

### Kill Reason

TVL collapsed 78-98% on target chains. At $1-5M TVL:
- Near-zero positions to liquidate
- Expected 0-1 liquidations per week
- Even with 0 competition — nothing to liquidate

---

## Comparative Matrix: Final (UPDATED)

| Factor | Ink/Tydro | HyperEVM/Morpho | HyperEVM/HyperLend | Euler (all) |
|---|---|---|---|---|
| **Code reuse** | 95% (Aave V3) | 60% (bot ready!) | **80% (Aave V3 fork!)** | KILLED |
| **Deploy time** | 1-2 days | 2-3 days | **2-3 days** | - |
| **TVL** | $446-527M | $431M supply | **$650M+ supply** | $1-5M |
| **Flash loans** | YES (5 bps) | NO (callback) | YES (4 bps) | NO |
| **Oracle** | Chaos Labs (no SVR) | Unknown (SVR risk) | Pyth + Chainlink (SVR risk) | - |
| **Competition** | 5 bots, ~2 liq/day | Low (profit check OFF) | Unknown | - |
| **Gas cost** | $0.00-$0.19 | <$0.01 | <$0.01 | - |
| **DEX volume** | $5.15M/day (LOW) | ~$1B/week | ~$1B/week | - |
| **Revenue ceiling** | $30-100/day (calm) | Unknown, $119M borrowed | Unknown, $172M borrowed | $0 |
| **Risk** | LOW | MEDIUM (SVR + new stack) | MEDIUM (SVR + dual-block) | - |

**KEY INSIGHT:** HyperLend = Aave V3 fork, not custom protocol! This makes HyperEVM a much stronger target:
- Ink OR HyperLend can reuse 80-95% of existing code
- Combined HyperEVM opportunity: Morpho ($431M) + HyperLend ($650M) = **$1.08B TVL on one chain**
- DEX liquidity $1B/week vs Ink $5.15M/day → HyperEVM has 28x more DEX volume

---

## REVISED EXECUTION PLAN (after agent findings)

**Ревізія пріоритетів:** HyperEVM став значно привабливішим після виявлення що HyperLend = Aave V3 fork. Два конкуруючих підходи:

**Підхід A (Low Risk):** Ink першим → HyperEVM
- Простіший: 1 chain, standard OP Stack, 5 відомих конкурентів
- Менший DEX volume ($5.15M/day)

**Підхід B (Higher Reward):** HyperEVM першим (Morpho + HyperLend)
- Складніший: dual-block, SVR, нова екосистема
- $1.08B combined TVL, $1B/week DEX volume, 2 протоколи одночасно

**Рекомендація: Підхід A** — Ink першим (lower risk, faster validation), потім HyperEVM з набутим досвідом. Але якщо DEX depth на Ink недостатній — pivot на HyperEVM.

### Phase 1: Ink/Tydro — НЕГАЙНО (Day 1-2)

```
Day 1:
├── [1] Find Velodrome SwapRouter exact address on Ink
├── [2] Check DEX pool depth: WETH/USDC, kBTC/USDC, USDT0/USDC
├── [3] Create liquidator/ink/ directory
├── [4] Fork mantle config → ink config.ts with addresses above
├── [5] Adapt FlashLoanLiquidator.sol or deploy with swapData routing
├── [6] forge create --rpc-url https://rpc-gel.inkonchain.com
└── [7] Verify on explorer.inkonchain.com

Day 2:
├── [8] Scan all borrowers via PoolDataProvider
├── [9] Calculate HF for all positions
├── [10] DRY_RUN 24h → log candidates
└── [11] Decision: GO LIVE or KILL
```

**Kill:** DEX depth < $100K || 0 positions HF < 1.2 after 48h

### Phase 2: HyperEVM — PARALLEL TWO PROTOCOLS (Day 3-7)

**2A: Morpho (ready-made bot)**
```
├── [1] Clone morpho-blue-liquidation-bot (config for chain 999 ALREADY EXISTS)
├── [2] pnpm install && pnpm deploy:executor
├── [3] Configure env: RPC_URL_999, LIQUIDATION_PRIVATE_KEY_999
├── [4] Fund wallet with HYPE for gas (<$1)
├── [5] Run DRY mode 48h → assess liquidation frequency + SVR impact
└── [6] GO/KILL decision
```

**2B: HyperLend (Aave V3 fork — reuse our code)**
```
├── [1] Create liquidator/hyperlend/ (fork ink config)
├── [2] Update Constants: Pool 0x00A89d...8b, Oracle 0xC9Fb...e, etc.
├── [3] Deploy FlashLoanLiquidator.sol on HyperEVM
│   └── Use HyperSwap V3 Router: 0x4e2960...094d as swap router
├── [4] Set up HyperLend position monitoring
├── [5] DRY_RUN 48h
└── [6] GO/KILL decision
```

**Kill:** SVR captures 80%+ profit || 5+ active bots || dual-block causes issues

### Phase 3: Optimization — Week 2

If any chain profitable:
- Add DEX aggregator routing (1inch/LiquidSwap for HyperEVM, SuperSwap for Ink)
- Multi-chain monitoring dashboard
- Telegram alerts per chain

---

## Sources

### On-chain Verified (by research agents)
- Ink Chain: RPC `rpc-gel.inkonchain.com`, Chain ID 57073
- Tydro Pool: `0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba` (12 reserves)
- Tydro Oracle: `0x4758213271BFdC72224A7a8742dC865fC97756e1` (Chaos Labs Edge)
- Tydro DataProvider: `0x96086C25d13943C80Ff9a19791a40Df6aFc08328`
- HyperEVM: RPC `rpc.hyperliquid.xyz/evm`, Chain ID 999
- DefiLlama API: Morpho HyperEVM ($431M), HyperLend ($349M), Euler Sonic ($1M)
- Ink LiquidationCall events: 23 in 11.5 days, 5 unique liquidator addresses
- Ink Velodrome: CLFactory `0x04625B046C69577EfC40e6c0Bb83CDBAfab5a55F`

### Web Research
- [Ink/Tydro Launch — The Block](https://www.theblock.co/post/374756/kraken-incubated-ethereum-l2-ink-rolls-out-tydro-white-label-instance-aave-v3-supports-ink-token)
- [Ink TVL $500M — The Defiant](https://thedefiant.io/news/blockchains/kraken-s-ink-layer-2-surpasses-usd500-million-in-tvl)
- [Ink L2BEAT](https://l2beat.com/scaling/projects/ink)
- [Tydro Documentation](https://docs.tydro.com/)
- [Tydro Smart Contracts](https://docs.tydro.com/developers/smart-contracts)
- [Tydro Oracle (Chaos Labs)](https://docs.tydro.com/primitives/oracle)
- [SuperSwap DEX](https://superswap.ink/)
- [Velodrome on Superchain](https://www.theblock.co/post/360689/velodrome-enables-native-cross-chain-swaps-for-its-dex-ecosystem-on-optimism-superchain)
- [Across Bridge on Ink](https://across.to/blog/across-is-live-on-ink-krakens-layer-2-chain)
- [Morpho $1B on Base — CryptoTimes](https://www.cryptotimes.io/2026/01/13/morpho-crosses-1-billion-in-active-loans-on-base-network/)
- [Morpho HyperEVM MIP-118](https://forum.morpho.org/t/mip-118-morpho-token-deployment-and-incentives-on-hyperliquid/2035)
- [HyperLend Flash Loans — TechBullion](https://techbullion.com/hyperlend-defi-lending-defi-borrowing-flash-loans-hyperloop-on-hyperevm-2026/)
- [Morpho Blue Liquidation Bot — GitHub](https://github.com/morpho-org/morpho-blue-liquidation-bot)
- [Euler V2 Liquidation Bot — GitHub](https://github.com/euler-xyz/liquidation-bot-v2)
- [Chainlink SVR — Docs](https://docs.chain.link/data-feeds/svr-feeds)
- [Chainlink acquires Atlas — The Block](https://www.theblock.co/post/386743/chainlink-acquires-transaction-ordering-solution-atlas-accelerating-rollout-of-its-non-toxic-mev-tool)
- [Euler V2 6 months — Blog](https://www.euler.finance/blog/euler-v2-6-months-in)
- [HyperEVM DEX — GlueX](https://gluex.xyz/chain/hyperevm)
- [HyperSwap — DexScreener](https://dexscreener.com/hyperevm/hyperswap)
- [Aave V3 Flash Loans](https://aave.com/docs/aave-v3/guides/flash-loans)
- [HyperEVM Precompiles — Ambit Labs](https://medium.com/@ambitlabs/demystifying-the-hyperliquid-precompiles-and-corewriter-ef4507eb17ef)
- [HyperEVM Architecture — RockNBlock](https://rocknblock.io/blog/how-does-hyperliquid-work-a-technical-deep-dive)
- [Morpho Liquidation Docs](https://docs.morpho.org/learn/concepts/liquidation/)
- [Ink RPC — ChainList](https://chainlist.org/chain/57073)
