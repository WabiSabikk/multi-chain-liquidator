# Handoff

**Сесія:** 2026-03-06 (session 6 — audit + fixes)
**Фокус:** Full end-to-end liquidation audit → verified bug fixes → deploy v0.5.2

## Зроблено цю сесію

### Аудит (верифіковано кожну проблему окремо)
1. **#4 Morpho seized_assets = 100% collateral** — CRITICAL, confirmed. Morpho reverts через arithmetic underflow коли seized > debt equivalent. Фікс: `compute_max_seized()` рахує seized з borrowed_assets + LIF + 95% safety margin.
2. **#2 V2 close factor 100% на Lendle** — HIGH, confirmed. Aave V2 ЗАВЖДИ 50% (verified from source). Lendle тихо обрізає, але Odos calldata для 2x більше collateral → revert. Фікс: `is_v2` check в обох `build_candidate_*` функціях.
3. **#8 Lendle USDT/USDC fee tier = 0** — MEDIUM, confirmed on-chain. FusionX не має pool USDT/USDC@3000 (default). Фікс: `setFeeTier(USDT, USDC, 100)` TX sent, verified on-chain.
4. **#3 parse_revert_reason false positives** — LOW, confirmed. `contains("35")` матчить gas amounts, hex data. Фікс: match тільки `"reverted: XX"` формат.
5. **#1 LendleCrossPoolLiquidator params.length dispatch** — confirmed, Odos routing тихо зламаний (fallback працює). Потребує redeploy для повного фіксу.

### Спростовані проблеми
- **#5 Nonce drift** — НЕ баг. На L2 з centralized sequencers TX accepted = TX mined.
- **#9 Zero candidates** — НЕ баг. min_profit фільтр (0.0 < 1.0) ловить їх.

### Deploy
- **v0.5.2** deployed to AWS, pm2 restarted, scans working

## Поточний стан
- **Bot:** v0.5.3, pm2 `multi-chain-liq`, LIVE, stable
- **Wallet:** `0xaa979a7FC2C112448638aB88518231cF82Ec3F3b`

### Контракти
| Chain | Contract | Address |
|-------|----------|---------|
| Mantle | MantleFlashLoanLiquidator (Aave V3) | `0xF2E6e7F255c46CC2353a8fD1D502f0c1920E1D43` |
| Mantle | LendleCrossPoolLiquidator v3 | `0xf4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91` |
| Ink | InkFlashLoanLiquidator v2 (Slipstream) | `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` |
| HyperEVM | HyperEVMFlashLoanLiquidator | `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` |
| HyperEVM | MorphoLiquidator | `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` |

### On-chain changes (this session)
- `setFeeTier(USDT, USDC, 100)` on v2 — TX `5c343ec1...`, block 92340660
- **LendleCrossPoolLiquidator v3** deployed — `0xf4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91`
  - Fix: `uint8 mode` flag instead of `params.length > 320` dispatch
  - USDT/USDC fee=100 in constructor
  - Odos router approved
- **Deprecated:** v2 `0x08cc9e00...` (params.length dispatch bug)

## Наступні кроки
1. Чекати crash/depeg — бот ready, всі баги пофіксені
2. Чекати crash/depeg — бот ready, Lendle = єдиний активний протокол
3. HyperEVM ініціалізація (повільна, скидається при рестарті)

## Контекст
- Lendle USDT (`0x201EBa5C`) ≠ Aave V3 USDT0 (`0x779Ded0c`) — різні токени
- Lendle close factor: ЗАВЖДИ 50% (V2 fork). V3 dynamic close factor НЕ застосовується.
- Morpho seized_assets: обчислюється з borrowed_assets * LIF, НЕ з collateral (underflow revert)
- Odos API: тільки Mantle (5000). `api.odos.xyz`, без ключа
- Velodrome Slipstream: `int24 tickSpacing`, NOT `uint24 fee`
- Бінарники v0.4.0-v0.5.1 збережені на AWS
