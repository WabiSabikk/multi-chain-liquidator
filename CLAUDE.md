# Multi-Chain Liquidator

Use Ukrainian language for communication.

## Overview

Multi-chain, multi-protocol liquidation bot на Rust. Flash loan funded ($0 capital risk). Один бінарник для всіх chains і протоколів.

**Chains:** Mantle (5000), Ink (57073), HyperEVM (999)
**Protocols:**
- Aave V3: Mantle (Aave V3 + Lendle), Ink (Tydro), HyperEVM (HyperLend + HypurrFi Pool)
- Morpho Blue: HyperEVM
- Felix CDP (Liquity V2): HyperEVM (future)
**Stack:** Rust + Alloy (Ethereum SDK) + tokio async + Foundry (Solidity contracts)

## Architecture

```
multi-chain-liquidator/
├── src/
│   ├── core/              # shared: types, rpc client, wallet, tx signing, alerts, config
│   ├── chains/
│   │   ├── mantle/        # chain 5000: MNT gas, Alchemy RPC
│   │   ├── ink/           # chain 57073: ETH gas, Gelato sequencer, OP Stack
│   │   └── hyperevm/      # chain 999: HYPE gas, dual-block (1s fast + 60s slow)
│   ├── protocols/
│   │   ├── aave_v3/       # shared Aave V3 logic (Mantle + Ink + HyperLend)
│   │   │   ├── monitor    # scan borrowers via Multicall3, calculate HF
│   │   │   ├── executor   # flash loan -> liquidate -> swap -> repay
│   │   │   └── types      # Position, Candidate, ReserveConfig
│   │   └── morpho/        # Morpho Blue (HyperEVM only)
│   │       ├── monitor    # scan via RPC (Morpho GraphQL optional)
│   │       ├── executor   # callback liquidation (no flash loans needed)
│   │       └── types
│   ├── dex/
│   │   ├── velodrome/     # Ink: Velodrome Slipstream CL
│   │   ├── fusionx/       # Mantle: FusionX V3 + Agni
│   │   ├── hyperswap/     # HyperEVM: HyperSwap V3
│   │   └── liquidswap/    # HyperEVM: LiquidSwap aggregator (for Morpho)
│   └── main.rs            # tokio runtime, spawn per-chain tasks
├── contracts/             # Solidity (Foundry)
│   └── src/
│       └── FlashLoanLiquidator.sol
├── docs/
├── research/              # research findings
├── data/                  # runtime data (borrower cache, logs)
├── Cargo.toml
└── .env                   # secrets (never commit)
```

## Working Principles

Same as parent project (~/projects/Crypto trading/CLAUDE.md):
- Test before deploy. Build locally, DRY_RUN first.
- Deploy and restart only with user permission.
- Every claim = on-chain proof or marked "HYPOTHESIS".
- No time estimates. Task lists and immediate action.

## Chain Configs

### Mantle (Chain 5000) — EXISTING DEPLOYMENT

#### Aave V3 (deployed 11 Feb 2026, 0 liquidations as of 6 Mar)

| Contract | Address |
|----------|---------|
| Aave V3 Pool | `0x458F293454fE0d67EC0655f3672301301DD51422` |
| PoolDataProvider | `0x487c5c669D9eee6057C44973207101276cf73b68` |
| AaveOracle | `0x47a063CfDa980532267970d478EC340C0F80E8df` |
| FlashLoanLiquidator | `0xF2E6e7F255c46CC2353a8fD1D502f0c1920E1D43` |
| FusionX V3 Router | `0x5989FB161568b9F133eDf5Cf6787f5597762797F` |
| Agni Router | `0x319B69888b0d11cEC22caA5034e25FfFBDc88421` |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` |

**Why 0 liquidations (verified 2026-03-06 via RPC eth_getLogs):** Protocol is ~50 days old. TVL dominated by stablecoin loops (sUSDe->USDT0, eMode 97% LTV). <50 whale wallets = 85% TVL. Closest: HF=1.0022 ($4.1M). Needs sUSDe depeg or market crash.
**WARNING:** Routescan API returns 0 events for this pool (incorrect). Blockscout `topic_0_include` filter is BROKEN (returns all events). Only RPC `eth_getLogs` is reliable.
| Odos Router (approved) | `0xD9F4e85489aDCD0bAF0Cd63b4231c6af58c26745` |

#### Lendle (Aave V2 fork, since 2023, 108 liqs/30d, 14 bots)

| Contract | Address |
|----------|---------|
| LendingPool | `0xCFa5aE7c2CE8Fadc6426C1ff872cA45378Fb7cF3` |
| AaveProtocolDataProvider | `0x552b9e4bae485C4B7F540777d7D25614CdB84773` |
| AaveOracle | `0x870c9692Ab04944C86ec6FEeF63F261226506EfC` |
| AddressesProvider | `0xAb94Bedd21ae3411eB2698945dfCab1D5C19C3d4` |

| LendleCrossPoolLiquidator v3 | `0xf4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91` |

**Dominant bot:** `0x13418a...` (38%, 41/108). Code reuse: 95% from aave_v3 module (V2 interface similar).
**Common pairs:** mETH→WETH (41%), USDC→WMNT (16%), USDT→USDT (16%).
**USDT NOTE:** Lendle USDT (`0x201EBa5CC46D216Ce6DC03F6a759e8E766e956aE`) ≠ Aave V3 USDT0 (`0x779Ded0c`). Cross-token mode: flash loan USDC → swap → USDT → liquidate → swap back.
**Liquidation activity:** ~4019 all-time, 11/7d (ONLY active protocol as of 2026-03-06).
**Reserves (11):** USDC, USDT, WBTC, WETH, WMNT, mETH, USDe, FBTC, cmETH, AUSD, sUSDe

**Gas:** MNT (~$0.00001/tx)
**Wallet:** `0xaa979a7FC2C112448638aB88518231cF82Ec3F3b` (same key for all chains)
**RPC:** dRPC (MANTLE_RPC_URL in .env), 10k block getLogs limit on free tier
**TVL:** Aave V3 ~$1B, Lendle ~$1.8M
**Status:** LIVE, pm2 `multi-chain-liq`

**Tokens:**
- WETH: `0xdEAddEaDdeadDEadDEADDEAddEADDEAddead1111` (18 dec)
- WMNT: `0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8` (18 dec)
- USDT0: `0x779Ded0c9e1022225f8E0630b35a9b54bE713736` (6 dec)
- USDC: `0x09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9` (6 dec)
- USDe: `0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34` (18 dec)
- sUSDe: `0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2` (18 dec)
- FBTC: `0xC96dE26018A54D51c097160568752c4E3BD6C364` (8 dec)

### Ink (Chain 57073) — NEW DEPLOYMENT

| Contract | Address |
|----------|---------|
| Aave V3 Pool (Tydro) | `0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba` |
| PoolAddressesProvider | `0x4172E6aAEC070ACB31aaCE343A58c93E4C70f44D` |
| PoolDataProvider | `0x96086C25d13943C80Ff9a19791a40Df6aFc08328` |
| AaveOracle (Chaos Labs) | `0x4758213271BFdC72224A7a8742dC865fC97756e1` |
| PoolConfigurator | `0x4f221e5c0b7103f7e3291e10097de6d9e3bfc02d` |
| Velodrome CLFactory | `0x04625B046C69577EfC40e6c0Bb83CDBAfab5a55F` |
| Velodrome SwapRouter | `0x63951637d667f23D5251DEdc0f9123D22d8595be` |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` |
| FlashLoanLiquidator v2 (Slipstream) | `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` |

**IMPORTANT:** Velodrome Slipstream uses `int24 tickSpacing` NOT `uint24 fee`. Selector `0xa026383e` (not Uniswap V3 `0x414bf389`). Old contract `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` is DEPRECATED (wrong interface).
**Tick spacings (verified):** WETH/USDC=100, WETH/weETH=1, WETH/wrsETH=1, WETH/ezETH=1, USDC/USDT0=1, USDC/GHO=1

**Gas:** ETH (~$0.00-$0.19/tx, 0.01 gwei)
**Wallet:** `0xaa979a7FC2C112448638aB88518231cF82Ec3F3b` (same key as Mantle)
**RPC (free):**
- `https://rpc-gel.inkonchain.com` (Gelato, primary + WSS)
- `https://rpc-qnd.inkonchain.com` (QuickNode + WSS)
- `https://ink.drpc.org` (dRPC + WSS)
**Block time:** 1s (OP Stack)
**Sequencer:** Gelato (centralized, FCFS)
**Oracle:** Chaos Labs Edge (NOT Chainlink -> NO SVR impact)
**TVL:** ~$446-527M
**DEX volume:** $5.15M/day (thin!)
**Competition:** 7 bots, dominant `0xf0570e...` takes 79% of all liquidations
**Liquidations (30d):** 127 on Tydro, clustered during crashes (Nov 12-14: 516 in 3 days)
**Explorer:** https://explorer.inkonchain.com (Blockscout), etherscan-style API works

**Tokens (12 reserves):**
- WETH: `0x4200000000000000000000000000000000000006` (18 dec)
- kBTC: `0x73e0c0d45e048d25fc26fa3159b0aa04bfa4db98`
- USDT0: `0x0200c29006150606b650577bbe7b6248f58470c1` (6 dec)
- USDG: `0xe343167631d89b6ffc58b88d6b7fb0228795491d`
- GHO: `0xfc421ad3c883bf9e7c4f42de845c4e4405799e73` (18 dec)
- USDC: `0x2d270e6886d130d724215a266106e6832161eaed` (6 dec)
- weETH: `0xa3d68b74bf0528fdd07263c60d6488749044914b` (18 dec)
- wrsETH: `0x9f0a74a92287e323eb95c1cd9ecdbeb0e397cae4` (18 dec)
- ezETH: `0x2416092f143378750bb29b79ed961ab195cceea5` (18 dec)
- sUSDe: `0x211cc4dd073734da055fbf44a2b4667d5e5fe5d2` (18 dec)
- USDe: `0x5d3a1ff2b6bab83b63cd9ad0787074081a52ef34` (18 dec)
- SolvBTC: `0xae4efbc7736f963982aacb17efa37fcbab924cb3`

**Flash Loans:** YES (Aave V3 built-in, 5 bps = 0.05% premium)

### HyperEVM (Chain 999) — NEW DEPLOYMENT

**Four protocols on same chain:**

#### HyperLend (Aave V3 fork, 3,091+ liqs all-time, 132/30d, ~10 bots)

| Contract | Address |
|----------|---------|
| Pool | `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` |
| PoolAddressesProvider | `0x72c98246a98bFe64022a3190e7710E157497170C` |
| Oracle | `0xC9Fb4fbE842d57EAc1dF3e641a281827493A630e` |
| ProtocolDataProvider | `0x5481bf8d3946E6A3168640c1D7523eB59F055a29` |
| UiPoolDataProvider | `0x3Bb92CF81E38484183cc96a4Fb8fBd2d73535807` |
| ACLManager | `0x10914Ee2C2dd3F3dEF9EFFB75906CA067700a04A` |
| HyperSwap V3 Router | `0x4e2960a8cd19b467b82d26d83facb0fae26b094d` |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` |
| FlashLoanLiquidator | `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` |

**Flash Loans:** YES (HyperLend, 4 bps = 0.04% premium)
**TVL:** $650M+ supply, $172M borrowed

**Tokens:**
| Token | Address | Dec |
|-------|---------|-----|
| wHYPE | `0x5555555555555555555555555555555555555555` | 18 |
| kHYPE | `0xfd739d4e423301ce9385c1fb8850539d657c296d` | 18 |
| wstHYPE | `0x94e8396e0869c9F2200760aF0621aFd240E1CF38` | 18 |
| beHYPE | `0xd8FC8F0b03eBA61F64D08B0bef69d80916E5DdA9` | 18 |
| UBTC | `0x9FDBdA0A5e284c32744D2f17Ee5c74B284993463` | 8 |
| UETH | `0xBe6727B535545C67d5cAa73dEa54865B92CF7907` | 18 |
| USOL | `0x068f321Fa8Fb9f0D135f290Ef6a3e2813e1c8A29` | 9 |
| USDe | `0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34` | 18 |
| sUSDe | `0x211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2` | 18 |
| USDC | `0xb88339CB7199b77E23DB6E890353E22632Ba630f` | 6 |
| USDT0 | `0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb` | 6 |
| USDHL | `0xb50A96253aBDF803D85efcDce07Ad8becBc52BD5` | 6 |
| USDH | `0x111111a1a0667d36bd57c0a9f569b98057111111` | 6 |
| USR | `0x0ad339d66bf4aed5ce31c64bc37b3244b6394a77` | 18 |, beHYPE

**Oracles:**
- Pyth HYPE/USD: `0xdE8d22d022261c9Fb4b5338DA8ceFb029175D0F5`
- Chainlink HYPE/USD: `0xf1CeE6FD8464a059B6d2F3e8A0754cD530e78c17`

#### HypurrFi Pool (Aave V3 fork, 4,000+ liqs all-time, dormant since Jan 20)

| Contract | Address |
|----------|---------|
| Pool (proxy) | `0xcecce0eb9dd2ef7996e01e25dd70e461f918a14b` |
| PoolAddressesProvider | `0xA73ff12D177D8F1Ec938c3ba0e87D33524dD5594` |
| ProtocolDataProvider | `0x895C799a5bbdCb63B80bEE5BD94E7b9138D977d6` |
| HyFiOracle | `0x9BE2ac1ff80950DCeb816842834930887249d9A8` |
| UiPoolDataProvider | `0x7b883191011AEAe40581d3Fa1B112413808C9c00` |
| ACLManager | `0x79CBF4832439554885E4bec9457C1427DFB9D0d3` |

**Creator:** `0xa73ff12d177d8f1ec938c3ba0e87d33524dd5594`
**Dominant bot:** `0x097bfc...` (151 liqs). Our aave_v3 module works with address swap.
**Peak days:** Dec 17-18 (555 liqs in 2 days), Sep 24-25 (246), Nov 21-22 (201)
**Status:** No liquidations since Jan 20, 2026 — will reactivate on next crash.
**Flash Loans:** Needs research — check if HypurrFi Pool supports flash loans or use HyperLend flash loans.
**NOTE:** HypurrFi also has Euler V2 isolated vaults ($1.6M TVL, 0 liquidations) — different mechanism, low priority.

#### Morpho Blue (2,000-3,000+ liqs all-time, 55 bots)

| Contract | Address |
|----------|---------|
| **Morpho Core** | `0x68e37dE8d93d3496ae143F2E900490f6280C57cD` |
| Adaptive Curve Irm | `0xD4a426F010986dCad727e8dd6eed44cA4A9b7483` |
| Chainlink Oracle V2 Factory | `0xeb476f124FaD625178759d13557A72394A6f9aF5` |
| MetaMorpho Factory V1.1 | `0xec051b19d654C48c357dC974376DeB6272f24e53` |
| Public Allocator | `0x517505be22D9068687334e69ae7a02fC77edf4Fc` |
| PreLiquidation Factory | `0x1b6782Ac7A859503cE953FBf4736311CC335B8f0` |
| VaultV2Factory | `0xD7217E5687FF1071356C780b5fe4803D9D967da7` |
| MorphoRegistry | `0x857B55cEb57dA0C2A83EE08a8dB529B931089aee` |
| Uniswap V3 Factory | `0xB1c0fa0B789320044A6F623cFe5eBda9562602E3` |

**NOTE:** Canonical `0xBBBBBbb...` has NO CODE on HyperEVM. Actual address is different!

**Whitelisted Vaults:**
- Felix USDC: `0x8A862fD6c12f9ad34C9c2ff45AB2b6712e8CEa27`
- Felix USDT: `0xFc5126377F0efc0041C0969Ef9BA903Ce67d151e`
- Felix HYPE: `0x2900ABd73631b2f60747e687095537B673c06A76`

**Flash Loans:** NOT needed (Morpho callback mechanism)
**TVL:** $431M supply, $119M borrowed
**LIF:** ~5% bonus (varies by LLTV)
**Liquidation history:** 2,000-3,000+ events (May 2025 - Feb 2026), 55 competing bots
**Liquidate event topic:** `0xa4946ede45d0c6f06a0f5ce92c9ad3b4751452d2fe0e25010783bcab57a67e41`

#### Felix CDP (Liquity V2 fork, 83+ liqs, low priority)

**Mechanism:** urgentRedemption (TroveOperation op=6), NOT TroveLiquidated event.
**TVL:** $36M. Stability Pool absorbs most liquidations automatically.
**Status:** Future — requires separate Liquity V2 module, not Aave V3 compatible.

**HyperEVM shared info:**
- Gas: HYPE (~$0.01/tx, 0.1 gwei)
- wHYPE: `0x5555555555555555555555555555555555555555`
- Block: dual (1s fast/2M gas + 60s slow/30M gas)
- RPC: `https://rpc.hyperliquid.xyz/evm`
- getLogs max: 1000 blocks (pagination required)
- EVM: Cancun (no blobs)
- Wallet: `0xaa979a7FC2C112448638aB88518231cF82Ec3F3b` (same key as Mantle)
- Explorer: https://hyperevmscan.io
- **Chainlink SVR: LIVE** (may capture 65% OEV on HyperLend)
- **Read precompiles:** `0x...0800+` (read HyperCore orderbook prices)

## Rust Stack

- **alloy** — Ethereum types, providers, contracts, signers (successor to ethers-rs)
- **tokio** — async runtime
- **foundry** — Solidity compilation and deployment
- **tracing** — structured logging
- **serde** — config/state serialization
- **reqwest** — HTTP client (for DEX aggregator APIs)

## Key Design Decisions

1. **One binary, multi-chain.** tokio::spawn per chain+protocol. Shared wallet manager and alerting.
2. **Aave V3 as shared module.** Mantle (Aave V3 + Lendle), Ink (Tydro), HyperEVM (HyperLend + HypurrFi Pool) share 95% of code. Chain-specific: addresses, DEX routing, gas token. Lendle = Aave V2 fork (same LiquidationCall event, slightly different interface).
3. **Morpho as separate module.** Different liquidation mechanism (callback vs flash loan), different position detection. Correct Liquidate event has 8 params (not Aave's 7).
4. **DEX abstraction.** Trait `SwapRouter` with implementations per DEX. Allows easy addition of new DEXes.
5. **Poll-based, not event-driven.** These chains don't need sub-10ms latency (unlike Arbitrum F1). Poll every 1-5s, check HF, execute.
6. **Flash loans for Aave V3, callbacks for Morpho.** $0 capital for all.

## Existing Deployed Contracts

| Chain | Contract | Address |
|-------|----------|---------|
| Mantle | MantleFlashLoanLiquidator (Aave V3) | `0xF2E6e7F255c46CC2353a8fD1D502f0c1920E1D43` |
| Mantle | LendleCrossPoolLiquidator v3 | `0xf4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91` |
| Ink | InkFlashLoanLiquidator v2 (Slipstream) | `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` |
| HyperEVM | HyperEVMFlashLoanLiquidator | `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` |
| HyperEVM | MorphoLiquidator | `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` |

**Odos Router** (approved on Mantle contracts): `0xD9F4e85489aDCD0bAF0Cd63b4231c6af58c26745`
**Deprecated:** Ink old contract `0x17B7b1B73FFbA773E6A92Bcbc3b27538A427977c` (wrong Velodrome interface)
**Deprecated:** Lendle v1 contract `0xD47223b7a191643ecdddC4715C948D88D5a13Bdd` (no cross-token support)

## AWS Deployment

Same EC2 as other bots: `ssh -i ~/.ssh/liquidator-bot.pem ec2-user@3.17.130.7`
pm2 process name: `multi-chain-liq` (single process for all chains)

## Related Projects

- **F1 (Arbitrum):** `~/projects/Crypto trading/liquidator/f1/` — NOT included here (needs local Nitro node, sequencer feed, sub-50ms latency)
- **Mantle monitor (TS):** `~/projects/Crypto trading/liquidator/mantle/` — will be replaced by this bot
- **Research:** `~/projects/crypto-inefficiencies/research/liquidation-new-chains-2026-03-05.md`

## Competitive Intelligence

**Liquidation pattern:** 80% of liquidations happen in 2-3 day clusters during market crashes. Bot MUST be always-on.

**Useful APIs for scanning:**
- Routescan (HyperEVM): `https://api.routescan.io/v2/network/mainnet/evm/999/etherscan/api`
- Routescan (Mantle): `https://api.routescan.io/v2/network/mainnet/evm/5000/etherscan/api`
- Ink Blockscout: `https://explorer.inkonchain.com/api?module=logs&action=getLogs&...`
- Mantle Blockscout: `https://explorer.mantle.xyz/api/v2`

**Event topics:**
- LiquidationCall (Aave V2/V3): `0xe413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286`
- Morpho Liquidate: `0xa4946ede45d0c6f06a0f5ce92c9ad3b4751452d2fe0e25010783bcab57a67e41`

**IMPORTANT:** Python `hashlib.sha3_256` != Ethereum keccak-256. Use `pycryptodome` or `web3.py` for topic hashes.
