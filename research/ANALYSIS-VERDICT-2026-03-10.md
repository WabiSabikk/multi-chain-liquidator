# Вердикт: Аналіз ліквідаційної діяльності (2026-03-10)

**Мета:** Зрозуміти реальну картину перед тим як писати код.
**Метод:** On-chain getLogs, логи бота, TX аналіз конкурентів, DEX liquidity mapping.

---

## 1. Скільки грошей ми РЕАЛЬНО втрачаємо?

### On-chain факти (30 днів, HyperEVM):
| Протокол | Ліквідацій | Наших | Estimated profit pool |
|----------|-----------|-------|----------------------|
| HyperLend | 231 | 0 | ~$3,118 |
| Morpho (meaningful) | 9 | 0 | ~$50-100 |
| HypurrFi | 0 | 0 | $0 |
| **TOTAL** | **240** | **0** | **~$3,200** |

### Втрачено за 36 годин бота (з логів):
| Причина | Сума |
|---------|------|
| Speculative TX gas drain | ~$25 (2,470 ревертнутих TX) |
| Missed HyperLend liqs (est 6-7 events) | ~$50-200 |
| Mantle downtime (30+ годин) | ~$5-20 |
| **TOTAL LOST** | **~$80-245** |

### Конкурентний ландшафт:
- **15+ ботів** борються за **~$104/день** на HyperLend
- Топ-3 боти забирають **66%** всіх ліквідацій
- Середній profit: **$13/ліквідацію** (після gas)

---

## 2. ГОЛОВНА ПРИЧИНА наших 0 ліквідацій

### Причина #1: НЕПРАВИЛЬНИЙ DEX FACTORY (CRITICAL)

Наш контракт маршрутує через **HyperSwap factory** (`0xB1c0fa...`).
Конкуренти використовують **Alt factory** (`0xFf7B3e8C...`).

| Пара | HyperSwap | Alt Factory | Різниця |
|------|-----------|-------------|---------|
| wHYPE/USDC | **НЕ ІСНУЄ** | 1.66e19 liq | ∞ |
| wHYPE/USDT0 | **НЕ ІСНУЄ** | 1.58e18 liq | ∞ |
| kHYPE/wHYPE | 3.86e24 | **7.61e25** | 19.7x |
| wHYPE/wstHYPE | 1.62e20 | **3.56e23** | 2,196x |

**29% всіх ліквідацій** (67 events) мають пари wHYPE→USDC або wHYPE→USDT0 — для нас НЕМОЖЛИВІ бо немає пулу на нашій фабриці.

### Причина #2: НЕПРАВИЛЬНІ FEE TIERS

| Пара | Наш контракт | Правильний | Наслідок |
|------|-------------|-----------|----------|
| kHYPE/wHYPE | fee=3000 (fallback) | fee=100 | 0 liquidity at 3000 |
| wstHYPE/wHYPE | fee=3000 | fee=100 | 0 liquidity at 3000 |
| beHYPE/wHYPE | fee=3000 | fee=100 | 0 liquidity at 3000 |
| wHYPE/USDC | fee=3000 | fee=500 (alt) | NO POOL AT ALL |

### Причина #3: AAVE FLASH LOAN vs V3 FLASH SWAP

| | Наш бот | Конкурент |
|--|---------|----------|
| Flash loan source | HyperLend (4 bps premium) | Uniswap V3 pool swap (0 premium) |
| Cost on 25 HYPE | $0.30 | $0.08 |

### Причина #4: SPECULATIVE MODE BURNED GAS
- 33,086 speculative fires → 2,470 TX sent → 0 success
- 65% failed due to gas exhaustion
- 26% failed due to nonce desync
- 5.3% reverted (HF > 1.0 — expected)

---

## 3. Чи варто продовжувати?

### Аргументи ЗА:
- Ринок існує: 231 ліквідацій/міс = 7.7/день
- Profit pool: ~$3,200/міс (HyperLend)
- Наш контракт ВЖЕ підтримує custom routing через `swapData` параметр
- Фікси прості: approve alt router + побудувати calldata в Rust
- Gas дешевий: ~$0.01/tx на HyperEVM
- Ми маємо node в Tokyo з localhost RPC = ~0ms latency

### Аргументи ПРОТИ:
- 15+ конкурентів за $104/день = ~$7/день на бота якщо рівні шанси
- Топ-3 боти забирають 66% — важко конкурувати без переваги
- Morpho майже мертвий (74% dust)
- Mantle/Ink мають окремі проблеми (RPC, bad debt)
- ROI: навіть якщо захопимо 20% ринку = ~$620/міс, при $490/міс за сервер в Tokyo

### Вердикт: **ПРОДОВЖУВАТИ, але з МІНІМАЛЬНИМИ змінами**

ROI позитивний ТІЛЬКИ якщо:
1. Фіксимо routing (alt factory + fee tiers) — це дає нам доступ до 100% ліквідацій замість ~30%
2. НЕ витрачаємо час на нові features (oracle prediction, pre-signed TX)
3. Мінімізуємо gas drain (вимкнути speculative mode)

---

## 4. Що САМЕ треба фіксити (конкретний план)

### FIX #1: Approve alt factory router (5 хв, on-chain TX)
```
cast send 0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c \
  "setApprovedRouter(address,bool)" \
  0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B true \
  --rpc-url https://rpc.hyperliquid.xyz/evm \
  --private-key $PRIVATE_KEY
```

### FIX #2: Fix fee tiers (5 хв, on-chain TXs)
```
# kHYPE/wHYPE: 3000 → 100
cast send 0x17B7... "setFeeTier(address,address,uint24)" \
  0xfd739d4e423301ce9385c1fb8850539d657c296d \
  0x5555555555555555555555555555555555555555 100

# wstHYPE/wHYPE: 3000 → 100
# beHYPE/wHYPE: 3000 → 100
# wHYPE/USDC: 3000 → 500 (alt factory)
# wHYPE/USDT0: 3000 → 500 (alt factory)
```

### FIX #3: Multi-hop calldata в Rust (1-2 години)
В `src/protocols/aave_v3/executor.rs`:
1. Побудувати Uniswap V3 path encoding: `tokenIn(20 bytes) + fee(3 bytes) + tokenOut(20 bytes)`
2. Для multi-hop: `tokenIn + fee1 + intermediate + fee2 + tokenOut`
3. Encode `exactInput(ExactInputParams)` calldata
4. Передати через `executeLiquidation_1(swapData, swapRouter)` з alt router

### FIX #4: Routing table в Rust
Маппінг на основі DEX аналізу:
```rust
// Direct routes (alt factory, 0x1EBDfC75...)
(kHYPE, wHYPE) => fee=100, direct
(wstHYPE, wHYPE) => fee=100, direct
(beHYPE, wHYPE) => fee=100, direct
(UETH, wHYPE) => fee=3000, direct
(wHYPE, USDC) => fee=500, direct (alt)
(wHYPE, USDT0) => fee=500, direct (alt)
(USDC, USDT0) => fee=100, direct (alt)
(USDe, USDC) => fee=100, direct (alt)

// Multi-hop routes (alt factory)
(kHYPE, USDC) => kHYPE→(100)→wHYPE→(500)→USDC
(kHYPE, USDT0) => kHYPE→(100)→wHYPE→(500)→USDT0
(kHYPE, UBTC) => kHYPE→(100)→wHYPE→(3000)→UBTC
(UETH, USDC) => UETH→(3000)→wHYPE→(500)→USDC
(wstHYPE, USDC) => wstHYPE→(100)→wHYPE→(500)→USDC
(beHYPE, USDC) => beHYPE→(100)→wHYPE→(500)→USDC
```

### FIX #5: Вимкнути speculative mode
Threshold: HF <= 1.0005 замість 1.005 (або вимкнути повністю).

### FIX #6: Поповнити газ
Wallet потребує мінімум 1-2 HYPE для нормальної роботи.

### FIX #7: Mantle RPC
Замінити Alchemy на dRPC для Mantle (Alchemy rate limited).

---

## 5. ROI оцінка

### Вартість фіксів:
- On-chain TXs (fix #1, #2): ~0.01 HYPE ($0.30)
- Rust routing code (fix #3, #4): ~2 години роботи
- Gas top-up: 1-2 HYPE ($30-60)
- Server: вже працює

### Очікувана дохідність:
- При 10% market share (реалістично для нового бота): $320/міс
- При 20% market share (оптимістично): $640/міс
- Server cost: $490/міс (Tokyo i3.xlarge)

### Breakeven:
- Потрібно **~15-16% market share** (~37 liqs/міс) щоб окупити сервер
- Або **перенести на дешевший instance** ($100-200/міс) для позитивного ROI

### CRITICAL: Tokyo server також хостить HyperEVM ноду
- Нода потрібна для ~0ms latency до HyperEVM
- Без ноди — latency 200-300ms через public RPC = програш конкурентам
- Нода + ліквідатор на одному сервері = спільний кост

---

## Alt factory Router та пули — Appendix

**Alt Factory:** `0xFf7B3e8C00e57ea31477c32A5B52a58Eea47b072`
**Alt Router:** `0x1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B`

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
| USDC/USDT0 | 100 | `0x94291BEA4c3aC9dBE81615083BB9A028722eeBeC` | 7.65e14 |
| USDC/USDe | 100 | `0xD2c34b86F9c2A1beB3D07eaFbA85a9D92Dc6A248` | 4.41e18 |
| USDe/USDT0 | 100 | `0x57a39276dA55040800eff10E4Bcaaa0DAFBb9a06` | 1.56e19 |

## Competitor Contracts

| Address | Role | Method |
|---------|------|--------|
| `0xdd8692Bc25972DBa5906201960e2dbe783D460Fa` | Liquidator contract | V3 flash swap, alt factory |
| `0xac5b4731229265627Cd3Fce62BB826493B38a66f` | EOA operator | |
| `0x5c5c3a1b17f911932720821ee4d07680e8de7d93` | Flash proxy (43KB) | Complex multi-route |
| `0xa7d0485a177236b19096a690210278271e133a0f` | Multi-protocol bot | exactInputSingle on alt router |
