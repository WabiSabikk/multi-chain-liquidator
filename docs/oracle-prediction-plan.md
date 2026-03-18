# Oracle Prediction Plan — Phase 10

## Контекст

Наш multi-chain liquidation bot (v0.5.0 TURBO) на HyperEVM (chain 999) виявляє ліквідації **ПІСЛЯ** того, як ціни оновлюються на on-chain оракулах. Поточний пайплайн:

1. WSS newHeads -> новий блок (0ms)
2. Multicall `getUserAccountData` / `position()` для at-risk позицій (~100-150ms RPC)
3. Побудова кандидата + відправка TX (~50ms)
4. **Загальна затримка: ~200ms після блоку**

Проблема: конкуренти (наприклад `0xdd8692bc` на HyperLend, `0x097bfc` на HypurrFi) бачать ту ж інформацію одночасно. Перемагає той, хто швидше. Наша Phase 9.1 (speculative candidates для HF < 1.005) частково вирішує це, але тільки для позицій які вже майже ліквідуються.

**Ключове спостереження:** на Arbitrum F1 бот читає sequencer feed і знає ціни ДО того, як вони потрапляють в блок. На HyperEVM немає sequencer feed, але є інші механізми для pre-block awareness.

### HyperLend Oracle Architecture (критично)

HyperLend oracle (`0xC9Fb4fbE842d57EAc1dF3e641a281827493A630e`) читає дані з **Hyperliquid System Oracle** через ланцюжок:
- **SystemOracle** (HyperCore L1 validator consensus) -> **Aggregator.sol** -> **AssetOracleProxy.sol** (Chainlink-compatible `latestRoundData()`)
- Для перп-listed активів (HYPE, BTC, ETH, SOL, тощо): **perp oracle price від L1 валідаторів** (weighted median з Binance, OKX, Bybit, Kraken, Kucoin, Gate IO, MEXC, Hyperliquid — ваги 3,2,2,1,1,1,1,1)
- Для non-perp активів: **keepers submit data**, EMA як ціна
- Валідатори публікують ціни **кожні 3 секунди**
- HyperEVM блоки: 1s (fast, 2M gas) + 60s (slow, 30M gas)

**Отже:** є ~2 секундне вікно між тим, як валідатори отримають нову ціну від бірж, і тим, як ця ціна потрапить в HyperEVM блок через SystemOracle -> Aggregator -> Pool.

## Мета

Скоротити detection-to-TX latency на HyperEVM з ~200ms (після блоку) до **pre-block awareness**: знати, що позиція стане ліквідуемою ЩЕ ДО того, як ціна потрапить в HyperLend oracle контракт. Конкретно:
- **Виявляти потенційних кандидатів на 1-3 секунди раніше** ніж on-chain oracle update
- **Мати TX готову до відправки** в момент, коли блок з оновленою ціною з'явиться
- **Збільшити win rate** проти конкурентів з ~0% (front-run by 3s) до >30%

## Результат

Після імплементації бот повинен:
1. Підтримувати off-chain shadow state всіх at-risk позицій з передбаченими цінами
2. При виявленні ціни яка зробить HF < 1.0 — мати pre-built TX готову
3. Відправляти TX на тому ж блоці де oracle ціна оновиться (або навіть раніше через speculative execution)
4. Фоллбек на поточний пайплайн (Phase 8/9) якщо pre-block prediction не спрацює

---

## Підхід 1: Off-chain Oracle Prediction (Pyth/Hermes + CEX Feed)

### Дослідження

**Як працює:** Замість очікування on-chain oracle update, підписуємось на off-chain ціни тих самих джерел, які використовують валідатори HyperCore (Binance, OKX, Bybit тощо), і/або на Pyth Hermes streaming API. Коли бачимо зміну ціни, pre-compute HF для всіх at-risk позицій локально.

**Pyth Hermes API:**
- SSE endpoint: `GET https://hermes.pyth.network/v2/updates/price/stream?ids[]=<price_feed_id>`
- Кожен feed оновлюється **кожні 400ms**
- Price feed IDs (hex): BTC/USD = `0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43`, ETH/USD = `0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace`, SOL/USD = `0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d`
- HYPE/USD feed ID потрібно знайти через API: `GET https://hermes.pyth.network/v2/price_feeds?query=HYPE`
- Latency: sub-second (~100-400ms від CEX trade до Hermes)

**Pyth Lazer (premium):**
- `pyth-lazer-client` Rust crate — ultra-low latency: оновлення **кожну 1ms**
- 4 concurrent WebSocket connections, TTL-based dedup, auto-failover
- Потрібен API key (paid tier)
- Crate: `pyth-lazer-client` на crates.io

**Але:** HyperLend oracle НЕ використовує Pyth напряму. HyperLend читає **HyperCore System Oracle** (validator consensus). Pyth/Chainlink використовуються тільки як secondary fallback. Отже, Pyth ціни != HyperLend ціни. Вони будуть КОРЕЛЬОВАНІ (обидва базуються на CEX spot), але не ідентичні.

**Правильний підхід:** моніторити ті ж самі CEX що й валідатори:
- Binance WS (вага 3): `wss://stream.binance.com:9443/ws/<symbol>@trade`
- OKX WS (вага 2): `wss://ws.okx.com:8443/ws/v5/public`
- Bybit WS (вага 2): `wss://stream.bybit.com/v5/public/spot`
- Крім того: Kraken, Kucoin, Gate IO, MEXC (ваги по 1)

Далі обчислюємо weighted median так само, як валідатори, і отримуємо оцінку майбутньої oracle ціни.

### Технічна реалізація

```
Новий модуль: src/core/price_predictor.rs

1. PricePredictor struct:
   - Підключення до CEX WebSocket feeds (Binance, OKX, Bybit — покривають ваги 3+2+2=7 з 12)
   - Локальний кеш останніх цін по кожній біржі
   - Функція compute_predicted_oracle(asset) -> f64 — weighted median

2. Інтеграція з monitor:
   - На кожне price update від CEX:
     a) Перерахувати HF для всіх at-risk позицій використовуючи predicted_oracle ціну
     b) Якщо predicted HF < 1.0 для будь-якої позиції:
        - Pre-build LiquidationCandidate
        - Додати в speculative_candidates черту
   - Коли приходить новий блок — перевірити чи oracle ціна змінилась як передбачено

3. HF pre-computation:
   - Для Aave V3: HF = (collateral_base * LT / 10000) / debt_base
   - collateral_base та debt_base вже закешовані з last full_scan
   - Потрібно лише замінити ціну одного активу і перерахувати
   - Для Morpho: HF = (collateral * oracle_price * LLTV) / (borrowed * ORACLE_SCALE * WAD)
```

**Rust crates:**
- `tokio-tungstenite` — WebSocket client (вже в екосистемі tokio)
- `serde_json` — парсинг CEX messages (вже в проекті)
- `pyth-lazer-client` — якщо вирішимо використовувати Pyth Lazer (optional)
- `simple_pyth_client_rs` або `pyth-hermes-rs` — для Hermes SSE (простіший варіант)

### Файли для зміни

| Файл | Зміна |
|------|-------|
| `src/core/price_predictor.rs` | **НОВИЙ** — CEX WS feed aggregator, weighted median, HF pre-computation |
| `src/core/mod.rs` | Додати `pub mod price_predictor;` |
| `src/main.rs` | Створити PricePredictor, передати в run_aave_chain/run_morpho_chain |
| `src/protocols/aave_v3/monitor.rs` | Додати метод `predict_candidates(predicted_prices)` — перерахувати HF з новими цінами |
| `src/protocols/morpho/monitor.rs` | Додати метод `predict_candidates(predicted_prices)` |
| `Cargo.toml` | Додати `tokio-tungstenite`, опціонально `pyth-lazer-client` |

### Latency improvement

| Етап | Поточний | З Підходом 1 |
|------|----------|-------------|
| Дізнаємось про зміну ціни | Блок N (0ms) | **~1-3s ДО блоку N** (CEX WS latency ~50-100ms) |
| Перевіряємо HF | +100-150ms (multicall RPC) | **~0ms** (локальне обчислення з cached positions) |
| TX готовий | +50ms | **Вже готовий** (pre-built) |
| **Загальна перевага** | baseline | **~1-3s раніше** |

### Ризики

1. **Predicted price != actual oracle price.** Валідатори використовують weighted MEDIAN з 8 бірж, ми моніторимо 3 (ваги 7/12). Median може відрізнятись якщо Kraken/Kucoin/Gate/MEXC дають інші ціни. **Мітігація:** моніторити всі 8 бірж, або використовувати safety margin (predict liquidation тільки якщо predicted HF < 0.98).
2. **False positives = wasted gas.** Якщо ми відправимо TX але oracle ціна не оновиться як передбачено — TX revert. На HyperEVM gas ~$0.01, тому це дешево, але не безкоштовно. **Мітігація:** рахувати false positive rate, вимикати якщо > 20%.
3. **CEX WebSocket disconnects.** Binance/OKX можуть дропнути з'єднання. **Мітігація:** reconnect logic, multiple connections.
4. **Не знаємо КОЛИ валідатори pushing нову ціну.** Кожні 3 секунди, але exact timing невідомий. Може бути jitter.
5. **Round-trip для non-perp assets.** Для активів без перп (наприклад деякі stablecoins), ціни йдуть через keepers, не validators. Predictor не покриє ці випадки.

---

## Підхід 2: HyperCore Read Precompiles

### Дослідження

**Як працює:** HyperEVM має precompile контракти на адресах `0x0800+`, які читають дані безпосередньо з HyperCore state. Ціна з precompile **гарантовано відповідає** latest HyperCore state на момент побудови EVM блоку.

**Oracle Price Precompile:**
- Адреса: `0x0000000000000000000000000000000000000807`
- Функція: `oraclePx(uint32 index)` -> `uint64` price
- Індекси: BTC=0, ETH=1 (порядок з meta API universe масиву)
- Конверсія: `price / 10^(6 - szDecimals)` для перпів
- Gas: 2000 + 65 * (input_len + output_len) — дешево

**Інші корисні precompiles:**
- `0x0800` — perpAssetInfo (метадані перпу)
- `0x0807` — oraclePx (oracle ціна)
- Повний список: `L1Read.sol` в `hyperliquid-dev/hyper-evm-lib`

**Важливо:** precompile ціни оновлюються РАЗОМ з EVM блоком. Вони НЕ дають pre-block awareness. Ціна в precompile = ціна в тому ж блоці. Але precompile дає **пряму ціну валідаторів** без проходження через HyperLend oracle контракт.

**Ключове питання: чи precompile ціна оновлюється РАНІШЕ ніж HyperLend oracle контракт?**

Відповідь: **ТАК, потенційно.** HyperLend oracle контракт (AssetOracleProxy -> Aggregator -> SystemOracle) потребує EVM transaction для оновлення ціни. Є два сценарії:
- Якщо SystemOracle оновлюється через precompile call ВСЕРЕДИНІ EVM блоку — ціна в precompile і в HyperLend oracle однакова в тому ж блоці.
- Якщо HyperLend oracle контракт потребує ОКРЕМОЇ keeper TX для push нової ціни — precompile ціна може оновитись НА 1-2 БЛОКИ РАНІШЕ ніж HyperLend oracle.

**Поточне розуміння:** HyperLend oracle читає SystemOracle, який в свою чергу читає precompile. Якщо `latestRoundData()` реалізований як view function що викликає precompile — вони синхронні. Якщо ж keeper submit + EMA — є затримка.

### Технічна реалізація

```
Новий модуль: src/core/hypercore_reader.rs

1. HyperCoreReader struct:
   - Зберігає mapping: asset_symbol -> perp_index (з meta API)
   - Метод read_oracle_price(index: u32) -> Price
     - Виклик: provider.call(0x0807, encode(index))
     - Декодування uint64 -> f64

2. Використання для перевірки oracle drift:
   - На кожному блоці: читати precompile prices для всіх collateral/debt активів
   - Порівнювати з HyperLend oracle ціною (з last multicall)
   - Якщо є drift (precompile price змінилась, а HyperLend oracle ще ні):
     - Перерахувати HF з precompile ціною
     - Pre-build candidate якщо predicted HF < 1.0

3. Batch via multicall:
   - Один multicall може зробити N precompile calls + M getUserAccountData calls
   - Атомарно: всі ціни з того ж блоку
```

**Solidity (для читання precompile з Rust через eth_call):**
```solidity
// Виклик через low-level staticcall
(bool success, bytes memory data) = address(0x0807).staticcall(
    abi.encode(uint32(index))
);
uint64 price = abi.decode(data, (uint64));
```

**Rust (через alloy):**
```rust
use alloy::primitives::{Address, Bytes, U256};

let precompile = Address::from_word(
    "0x0000000000000000000000000000000000000807".parse().unwrap()
);
let input = U256::from(index).to_be_bytes::<32>();
let result = provider.call(
    &alloy::rpc::types::TransactionRequest::default()
        .to(precompile)
        .input(Bytes::from(input.to_vec()))
).await?;
let raw_price = u64::from_be_bytes(result[24..32].try_into().unwrap());
```

### Файли для зміни

| Файл | Зміна |
|------|-------|
| `src/core/hypercore_reader.rs` | **НОВИЙ** — precompile reader, asset index resolver, batch oracle read |
| `src/core/mod.rs` | Додати `pub mod hypercore_reader;` |
| `src/main.rs` | Ініціалізувати HyperCoreReader для HyperEVM chain |
| `src/protocols/aave_v3/monitor.rs` | Додати optional precompile price comparison в quick_scan (HyperEVM only) |
| `src/protocols/morpho/monitor.rs` | Те саме для Morpho |

### Latency improvement

| Сценарій | Покращення |
|----------|-----------|
| Precompile і HyperLend oracle синхронні | **0ms** — ніякого покращення (ціна та сама в тому ж блоці) |
| HyperLend oracle має затримку (keeper delay) | **1-2 блоки (1-2s)** — precompile показує нову ціну раніше |
| Порівняно з Підходом 1 | **Гірше** — precompile не дає pre-block awareness, тільки intra-block |

**Реалістична оцінка: малоймовірне значне покращення latency.** Головна цінність — додатковий сигнал для валідації predicted prices з Підходу 1, та backup oracle для перевірки.

### Ризики

1. **Немає pre-block awareness.** Precompile ціни доступні тільки ВСЕРЕДИНІ блоку. Якщо HyperLend oracle оновлюється в тому ж блоці — нуль покращення.
2. **Asset index mapping може змінитись.** Hyperliquid може додати нові перпи, змінивши індекси. Потрібен runtime resolve через meta API.
3. **Не всі HyperLend активи мають перп.** UBTC, kHYPE, wstHYPE, beHYPE — можуть не мати відповідного перп індексу. Precompile oraclePx працює тільки для активів з перп маркетом.
4. **Додатковий RPC overhead.** Кожний precompile call = додаткова gas/compute навантаження в multicall.

---

## Підхід 3: Node Replica Stream

### Дослідження

**Як працює:** Запустити Hyperliquid non-validator node на нашому AWS інстансі. Node пише L1 transactions (включаючи oracle updates від валідаторів) в `~/hl/data/replica_cmds/{start_time}/{date}/{height}`. Ми можемо watch цю директорію через inotify і парсити дані ДО того, як вони потраплять в EVM блок.

**Node data streams:**
- `--replica-cmds-style actions-and-responses` — пише ВСІ L1 транзакції в `~/hl/data/replica_cmds/`
- `--write-trades` — трейди в `~/hl/data/node_trades/hourly/{date}/{hour}`
- `--write-fills` — fills в `~/hl/data/node_fills/hourly/{date}/{hour}`
- `--write-hip3-oracle-updates` — HIP-3 deployer oracle updates в `~/hl/data/hip3_oracle_updates/hourly/{date}/{hour}`

**Формат:** JSON файли, один файл на height (block). Приклад trade:
```json
{
  "coin": "BTC",
  "side": "B",
  "time": 1714746101000,
  "px": "96036.0",
  "sz": "0.1",
  "hash": "0x...",
  "trade_dir_override": null,
  "side_info": [...]
}
```

**Oracle updates в replica_cmds:** L1 дані містять ВСІ транзакції, включаючи **oracle price submissions від валідаторів**. Формат oracle submission — не задокументований публічно, але можна reverse-engineer з `actions-and-responses` стилю.

**Проблема: ~/hl/data/ не існує на нашому AWS інстансі.** Ми НЕ запускаємо Hyperliquid node. Потрібно буде:
1. Встановити `hl-visor`
2. Запустити non-validator node з `--write-trades --replica-cmds-style actions-and-responses`
3. Дочекатись синхронізації (може зайняти години/дні)

**Hardware вимоги (non-validator):**
- CPU: 2+ cores
- RAM: 16GB+ recommended
- Storage: 100GB+ SSD (L1 data grows fast)
- Network: stable connection

**Наш AWS: t3.small (2 vCPU, 2GB RAM)** — НЕДОСТАТНЬО для node. Потрібен мінімум t3.xlarge (4 vCPU, 16GB RAM, $0.17/hr = $122/mo).

### Технічна реалізація

```
Фаза A: Розгортання node (DevOps)

1. Upgrade AWS instance до t3.xlarge або окремий instance для node
2. Встановити hl-visor:
   curl https://binaries.hyperliquid.xyz/Testnet/hl-visor > ~/hl-visor
   chmod +x ~/hl-visor
3. Запустити non-validator:
   ~/hl-visor run-non-validator \
     --replica-cmds-style actions-and-responses \
     --write-trades \
     --serve-eth-rpc
4. Дочекатись синхронізації з мережею

Фаза B: Oracle stream parser (Rust)

Новий модуль: src/core/replica_watcher.rs

1. ReplicaWatcher struct:
   - inotify watch на ~/hl/data/replica_cmds/{start_time}/{date}/
   - Parser для JSON файлів
   - Фільтр: шукаємо oracle price submission actions

2. При виявленні нової oracle ціни:
   - Порівняти з cached ціною
   - Якщо суттєва зміна: перерахувати HF для at-risk позицій
   - Pre-build candidates

3. Інтеграція з main loop:
   - mpsc channel: ReplicaWatcher -> scan loop
   - Scan loop перевіряє channel перед кожним блоком
```

### Файли для зміни

| Файл | Зміна |
|------|-------|
| `src/core/replica_watcher.rs` | **НОВИЙ** — inotify watcher, JSON parser, oracle extraction |
| `src/core/mod.rs` | Додати `pub mod replica_watcher;` |
| `src/main.rs` | Запуск ReplicaWatcher в окремому tokio task, mpsc channel |
| `Cargo.toml` | Додати `inotify` (Linux-specific), `notify` (cross-platform alternative) |

### Latency improvement

| Етап | Поточний | З replica stream |
|------|----------|-----------------|
| Oracle price available | Після EVM блоку (WSS newHeads) | **~0.5-2s ДО EVM блоку** (L1 oracle action йде раніше ніж EVM block construction) |
| HF computation | +100-150ms (RPC multicall) | **~0ms** (локальне обчислення) |
| **Загальна перевага** | baseline | **~0.5-2s раніше** |

**Теоретично:** L1 oracle submissions потрапляють в replica_cmds РАНІШЕ ніж в EVM блок, бо EVM блок будується ПІСЛЯ L1 consensus. Це дає вікно ~0.5-2s.

### Ризики

1. **Потребує значних інфраструктурних інвестицій.** Upgrade AWS ($122/mo), node setup, sync time. ROI непевний.
2. **Oracle action format не задокументований.** Потрібно reverse-engineer яку L1 action шукати в replica_cmds. Може виявитись що oracle submissions НЕ включені або мають інший формат.
3. **Node sync + reliability.** Якщо node відстане — дані застаріють. Потрібен моніторинг health.
4. **inotify latency.** Файлова система notification може мати ~10-50ms латентність. Не zero-cost.
5. **Storage growth.** `actions-and-responses` стиль генерує великий обсяг даних. Потрібна ротація.
6. **Single point of failure.** Якщо node падає — втрачаємо pre-block awareness. Потрібен fallback.
7. **Невідомо, чи є oracle price submission в replica_cmds.** `--write-hip3-oracle-updates` пише тільки HIP-3 deployer oracle updates, НЕ системні oracle ціни. Системні oracle submissions можуть бути internal L1 consensus mechanism, не exposed в replica_cmds.

---

## Порівняльна таблиця

| Критерій | Підхід 1: CEX Feed | Підхід 2: Precompile | Підхід 3: Replica Stream |
|----------|--------------------|--------------------|------------------------|
| Pre-block awareness | **ТАК (1-3s)** | НІ (same block) | МОЖЛИВО (0.5-2s) |
| Latency improvement | **1-3 секунди** | 0-2 блоки | 0.5-2 секунди |
| Implementation complexity | Середня | Низька | Висока |
| Infrastructure cost | ~$0 (CEX WS free) | $0 | ~$122/mo (AWS upgrade) |
| Reliability | Висока (multiple CEXes) | Дуже висока (on-chain) | Середня (node dependency) |
| False positive risk | Середній | Низький | Середній |
| Works for all assets | Тільки CEX-listed | Тільки перп-listed | Невідомо |
| Time to implement | 2-3 sessions | 1 session | 3-5 sessions |

---

## План реалізації

### Етап 1: Підхід 2 (Precompile) — LOW HANGING FRUIT

**Чому першим:** найпростіший, zero infrastructure cost, дасть нам baseline розуміння timing oracle updates.

1. Створити `src/core/hypercore_reader.rs`
2. Resolve asset indices через Hyperliquid meta API (one-time на старті)
3. В quick_scan для HyperEVM: додати precompile price read
4. Логувати drift між precompile price і HyperLend oracle price
5. **Метрика:** скільки блоків precompile ціна випереджає HyperLend oracle?

**Якщо drift > 0:** precompile дає перевагу, оптимізувати.
**Якщо drift = 0:** переходити до Підходу 1.

### Етап 2: Підхід 1 (CEX Feed) — MAIN BET

**Чому другим:** найбільший потенціал (1-3s pre-block), але складніший в імплементації.

1. Створити `src/core/price_predictor.rs`
2. Підключитись до Binance + OKX + Bybit WebSocket (покриває 7/12 weight)
3. Реалізувати weighted median compute
4. Інтегрувати з monitor: predict_candidates() на кожний price tick
5. Paper test: логувати predicted vs actual oracle prices для 24-48h
6. Якщо accuracy > 95%: увімкнути speculative TX sending

### Етап 3: Підхід 3 (Replica Stream) — ONLY IF NEEDED

**Чому останнім:** найбільш ризикований, найдорожчий, може не дати oracle data.

1. **Спочатку:** перевірити чи oracle submissions видимі в replica_cmds (запустити node на testnet)
2. Якщо так — upgrade AWS, deploy node, реалізувати watcher
3. Якщо ні — KILL цей підхід

### Залежності

```
Етап 1 (Precompile) — незалежний, можна починати одразу
    │
    └── Результат: drift metric
         │
         ├── drift > 0 → Оптимізувати precompile, І починати Етап 2
         └── drift = 0 → Починати Етап 2 (CEX Feed як основний підхід)

Етап 2 (CEX Feed) — незалежний від Етапу 1, але інформований його результатами
    │
    └── Результат: prediction accuracy + win rate improvement
         │
         ├── accuracy > 95%, win rate improved → DONE, skip Етап 3
         └── accuracy < 90% → Досліджувати Етап 3

Етап 3 (Replica) — тільки якщо Етап 1+2 недостатні
```

---

## Kill Criteria

### Підхід 1: CEX Feed
- **KILL якщо:** predicted price vs actual oracle price accuracy < 85% over 48h
- **KILL якщо:** false positive TX rate > 30% (too many reverts)
- **KILL якщо:** Binance/OKX/Bybit WS stability < 95% uptime
- **PIVOT якщо:** accuracy 85-95% — збільшити safety margin (predict HF < 0.95 замість < 1.0)

### Підхід 2: Precompile
- **KILL якщо:** precompile price drift = 0 blocks consistently (no advantage over HyperLend oracle)
- **KILL якщо:** більшість HyperLend assets не мають перп index (coverage < 50%)
- **KEEP as monitoring tool** навіть якщо не дає latency advantage — корисно для oracle validation

### Підхід 3: Replica Stream
- **KILL BEFORE IMPLEMENTATION якщо:** testnet node shows oracle submissions NOT in replica_cmds
- **KILL якщо:** AWS upgrade + node maintenance cost > expected monthly profit increase
- **KILL якщо:** node sync time > 48h або frequent desync
- **KILL якщо:** Підхід 1 дає > 90% prediction accuracy (no need for Підхід 3)

---

## Appendix: Asset Index Mapping (HyperCore)

Індекси отримуються з `meta` endpoint: `POST https://api.hyperliquid.xyz/info` з body `{"type": "meta"}`.
Порядок у `universe` масиві = perp index для precompile `oraclePx(index)`.

Відомі маппінги:
- BTC = index 0 (szDecimals=5)
- ETH = index 1 (szDecimals=4)
- HYPE = index ~40+ (потребує runtime resolve, szDecimals TBD)
- SOL, DOGE, тощо = визначити через API

**Скрипт для resolve:**
```bash
curl -s -X POST https://api.hyperliquid.xyz/info \
  -H 'Content-Type: application/json' \
  -d '{"type":"meta"}' | jq '.universe | to_entries[] | "\(.key): \(.value.name) (szDec=\(.value.szDecimals))"'
```

## Appendix: HyperLend Oracle Contract Chain

```
CEX Prices (Binance, OKX, ...)
    ↓ [validators compute weighted median every 3s]
HyperCore System Oracle (L1 consensus state)
    ↓ [precompile 0x0807 reads this]
EVM Precompile (0x0807)
    ↓ [AssetOracleProxy reads precompile OR keeper submits]
HyperLend Oracle (0xC9Fb4fbE842d57EAc1dF3e641a281827493A630e)
    ↓ [Aave V3 Pool calls getAssetPrice() during liquidation]
HyperLend Pool (0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b)
```

**Точки де ми можемо "підсмотрити" ціну раніше:**
1. **Підхід 1:** моніторити CEX prices (на рівні "CEX Prices")
2. **Підхід 2:** читати precompile 0x0807 (на рівні "EVM Precompile")
3. **Підхід 3:** парсити L1 state (на рівні "HyperCore System Oracle")
