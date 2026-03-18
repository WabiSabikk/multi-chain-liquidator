# All Lending/Borrowing Protocols on Target Chains

**Date:** 2026-03-05
**Purpose:** Find ALL protocols where liquidations can happen on Mantle, Ink, HyperEVM

---

## MANTLE (Chain 5000) — 16 lending/borrowing protocols found

### 1. Aave V3 (ALREADY MONITORED)
- **TVL:** ~$1B total market size (supply + borrow), launched 2026-02-11
- **Pool:** `0x458F293454fE0d67EC0655f3672301301DD51422`
- **Status:** Our bot already monitors this
- **Liquidation type:** Standard Aave V3 flash loan liquidation

### 2. Lendle (Aave V2 fork) — HIGH PRIORITY
- **TVL:** ~$6.4M (down 22% recently), 12 pools, avg APY 3.8%
- **LendingPool:** `0xCFa5aE7c2CE8Fadc6426C1ff872cA45378Fb7cF3`
- **LendingPoolAddressesProvider:** `0xAb94Bedd21ae3411eB2698945dfCab1D5C19C3d4`
- **AaveOracle:** `0x870c9692Ab04944C86ec6FEeF63F261226506EfC`
- **AaveProtocolDataProvider:** `0x552b9e4bae485C4B7F540777d7D25614CdB84773`
- **LendingPoolConfigurator:** `0x30D990834539E1CE8Be816631b73a534e5044856`
- **WETHGateway:** `0xEc831f8710C6286a91a348928600157f07aC55c2`
- **LEND token:** `0x25356aeca4210eF7553140edb9b8026089E49396`
- **Liquidation type:** Aave V2 style (liquidationCall on LendingPool)
- **Flash loans:** YES (Aave V2 built-in)
- **Note:** First lending protocol on Mantle (since 2023). $6.4M is small but it's the OLDEST protocol — likely HAS had liquidations. Aave V2 interface = our existing code works with minimal changes.

### 3. INIT Capital (Hook-based lending) — MEDIUM PRIORITY
- **TVL:** ~$5.3M on Mantle, $1.6M borrowed
- **InitCore:** `0x972BcB0284cca0152527c4f70f8F689852bCAFc5`
- **InitOracle:** `0x4E195A32b2f6eBa9c4565bA49bef34F23c2C0350`
- **Config:** `0x007F91636E0f986068Ef27c950FA18734BA553Ac`
- **PosManager:** `0x0e7401707CD08c03CDb53DAEF3295DDFb68BBa92`
- **RiskManager:** `0x0c03cd3e8b669680Bf306Fc72F1dc2cAC592f951`
- **LiqIncentiveCalculator:** `0x66BDbf2Eefc84f83b476dB238574ca5Cb00550aD`
- **InitLens:** `0x7d2b278b8ef87bEb83AeC01243ff2Fed57456042`
- **MoneyMarketHook:** `0xf82CBcAB75C1138a8F1F20179613e7C0C8337346`
- **Lending Pools:**
  - WETH: `0x51AB74f8B03F0305d8dcE936B473AB587911AEC4`
  - WBTC: `0x9c9F28672C4A8Ad5fb2c9Aca6d8D68B02EAfd552`
  - WMNT: `0x44949636f778fAD2b139E665aee11a2dc84A2976`
  - USDC: `0x00A55649E597d463fD212fBE48a3B40f0E227d06`
  - USDT: `0xadA66a8722B5cdfe3bC504007A5d793e7100ad09`
  - METH: `0x5071c003bB45e49110a905c1915EbdD2383A89dF`
  - USDY: `0xf084813F1be067d980a0171F067f084f27B3F63A`
  - USDe: `0x3282437C436eE6AA9861a6A46ab0822d82581b1c`
  - fBTC: `0x233493e9dc68e548ac27e4933a600a3a4682c0c3`
- **Liquidation type:** Custom (INIT-specific, hook-based). Need to study InitCore.liquidate()
- **Note:** "Liquidity Hook" architecture is unique. Lower TVL but 9 lending pools. Requires custom liquidation logic.

### 4. Compound V3 (Comet) — MEDIUM PRIORITY
- **TVL:** Unknown (recently deployed on Mantle)
- **Market:** USDe market with WETH, mETH, FBTC as collateral
- **Contract:** Not found in search — need to check mantlescan.xyz or Compound Github deployments folder
- **Liquidation type:** Compound V3 absorb() function
- **Note:** Updated oracle to SVR (Feb 2026). Active governance. Different liquidation mechanism than Aave (absorb vs liquidationCall).

### 5. Fluid (Instadapp) — MEDIUM PRIORITY
- **TVL:** Unknown on Mantle specifically (total $3B+ across all chains)
- **Contracts:** Not found — check docs.fluid.instadapp.io/contracts/contract-addresses.html
- **Liquidation type:** Fluid-specific (Liquidity layer + fToken vaults)
- **Note:** Major protocol ($3B TVL total). If significant TVL on Mantle, worth pursuing.

### 6. Kinza Finance — LOW PRIORITY
- **TVL:** Unknown on Mantle
- **Chains:** BNB, opBNB, Ethereum, Mantle, Monad Testnet
- **Contracts:** Not found — check docs.kinza.finance
- **Liquidation type:** Aave V3 fork (KZA-lending Github)
- **Note:** If Aave V3 fork, our existing code works. Need to verify Mantle TVL.

### 7. Aurelius Finance (CDP + Lending) — LOW PRIORITY
- **TVL:** ~$2.1M (as of May 2024, may be different now)
- **aUSD token:** `0x00000000efe302beaa2b3e6e1b18d08d69a9012a` (on Mantlescan)
- **Liquidation type:** CDP liquidation (Liquity/Ethos Reserve style) + Granary-style lending market
- **Note:** Small TVL. CDP = different liquidation mechanism than pool-based lending.

### 8. Gravita Protocol (CDP) — LOW PRIORITY
- **TVL:** ~$11.7M total (across all chains, Mantle share unknown)
- **Contracts:** Not found — check github.com/Gravita-Protocol/Gravita-SmartContracts
- **Liquidation type:** CDP liquidation (GRAI stablecoin)
- **Note:** Multi-chain protocol. Need to check Mantle-specific TVL.

### 9. Clearpool — SKIP (no standard liquidation)
- **Type:** Unsecured institutional lending (no collateral = no liquidation)
- **Note:** Not relevant for liquidation bot.

### 10. Dolomite — SKIP (tiny TVL)
- **TVL on Mantle:** $86K (negligible)
- **Main chains:** Ethereum ($227M), Arbitrum ($32M)
- **Note:** Not worth the integration effort on Mantle.

### 11. MethLab — SKIP (no liquidation by design)
- **Type:** Liquidation-free, oracle-less lending
- **Note:** Fixed-rate/fixed-term. No liquidation mechanism = no opportunity.

### 12. Minterest — SKIP (wind-down)
- **Status:** Completed orderly wind-down. No new deposits.
- **Note:** Dead protocol. Only existing users closing positions.

### 13. MYSO Finance — SKIP (no standard liquidation)
- **Type:** Zero-liquidation lending (fixed-term, no oracle-based liquidation)

### 14. Timeswap — LOW PRIORITY
- **Type:** Oracle-less AMM-based lending
- **Note:** Different mechanism (no HF-based liquidation). May not have traditional liquidation opportunities.

### 15. Teller — SKIP (non-custodial, NFT lending)
- **Type:** P2P lending, NFT collateral
- **Note:** Different market, not pool-based.

### 16. Demex — SKIP (derivatives + money market)
- **Type:** Primarily derivatives with small money market
- **Note:** Unknown TVL, likely small.

### MANTLE SUMMARY
| Protocol | TVL | Priority | Liquidation Type | Code Reuse |
|----------|-----|----------|-----------------|------------|
| **Aave V3** | ~$1B | DONE | Flash loan | Already built |
| **Lendle** | ~$6.4M | HIGH | Aave V2 liquidationCall | 95% reuse |
| **INIT Capital** | ~$5.3M | MEDIUM | Custom hook-based | New code needed |
| **Compound V3** | Unknown | MEDIUM | absorb() | New code needed |
| **Fluid** | Unknown | MEDIUM | Fluid-specific | New code needed |
| **Kinza** | Unknown | LOW | Aave V3 fork | 95% reuse |
| **Aurelius** | ~$2M | LOW | CDP (Liquity) | New code needed |
| **Gravita** | Unknown | LOW | CDP (Liquity) | New code needed |

---

## INK (Chain 57073) — 3 lending/borrowing protocols found

### 1. Tydro/Aave V3 (ALREADY PLANNED)
- **TVL:** ~$380-534M (growing rapidly with Kraken backing)
- **Pool:** `0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba`
- **Status:** Already in our codebase for deployment
- **Liquidation type:** Standard Aave V3 flash loan liquidation

### 2. Morpho Blue — LOW PRIORITY (infrastructure-only)
- **TVL:** ~$1.41 (effectively zero, infrastructure-only deployment)
- **Core:** `0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb` (same address on all chains)
- **Status:** Deployed but not active. No supported markets or MORPHO rewards yet.
- **Note:** May become active later if DAO approves. Monitor.

### 3. No other lending protocols found on Ink
- Ink is 15 months old, Kraken-backed
- DeFi ecosystem still maturing
- Tydro is the dominant (only meaningful) lending protocol
- DEX: Velodrome (primary), $5.15M/day volume

### INK SUMMARY
| Protocol | TVL | Priority | Status |
|----------|-----|----------|--------|
| **Tydro/Aave V3** | ~$534M | HIGH | Ready for deployment |
| **Morpho** | ~$0 | SKIP | Infrastructure-only |

**Key insight for Ink:** Tydro is THE protocol. All liquidation opportunities on Ink come from Tydro. Focus efforts here.

---

## HYPEREVM (Chain 999) — 10+ lending/borrowing protocols found

### 1. HyperLend (Aave V3 fork) — ALREADY PLANNED
- **TVL:** ~$380M supply, $172M borrowed (DeFiLlama)
- **Pool:** `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b`
- **Status:** Already in our codebase
- **Liquidation type:** Standard Aave V3 flash loan liquidation
- **Flash loans:** YES (4 bps = 0.04%)

### 2. HypurrFi (Euler V2 fork) — **HIGH PRIORITY — BIGGEST OPPORTUNITY MISSED**
- **TVL:** ~$350M (supply), massive protocol
- **Type:** Euler V2-based with pooled + isolated markets
- **Stablecoin:** USDXL (synthetic dollar)
- **Built on:** Euler Vault Kit (EVK) + Ethereum Vault Connector (EVC)
- **Contracts:** Not found directly — need to check docs.hypurr.fi or hypurrscan.io
- **Liquidation type:** Euler V2 style (checkLiquidation + liquidate via EVC)
- **Flash loans:** Via EVC (Euler Vault Connector enables atomic operations)
- **Note:** THIS IS HUGE. $350M TVL protocol we weren't tracking. Euler V2 liquidation mechanism. Need to research ASAP.

### 3. Morpho Blue — ALREADY PLANNED
- **TVL:** ~$431M supply, $119M borrowed
- **Core:** `0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb`
- **Status:** Already in our codebase
- **Note:** Felix operates Morpho vaults (see #4)

### 4. Felix Protocol (Liquity V2 CDP + Morpho Vanilla) — HIGH PRIORITY
- **TVL:** ~$265M total ($330M feUSD at peak)
- **Two products:**
  - **Felix CDP:** Liquity V2-style, mints feUSD stablecoin. Collateral: HYPE, kHYPE, BTC
  - **Felix Vanilla:** Morpho-powered lending markets (USDC, HYPE, UBTC)
- **Felix USDC Vault (Morpho):** `0x8A862fD6c12f9ad34C9c2ff45AB2b6712e8CEa27`
- **Felix USDT Vault (Morpho):** `0xFc5126377F0efc0041C0969Ef9BA903Ce67d151e`
- **Felix HYPE Vault (Morpho):** `0x2900ABd73631b2f60747e687095537B673c06A76`
- **Liquidation type:** Two types:
  - CDP: Liquity V2 redemption + liquidation mechanism
  - Vanilla: Morpho liquidation (already covered by Morpho module)
- **Note:** The Morpho vaults are already handled by our Morpho module. The CDP liquidations require separate Liquity V2-style code.

### 5. Keiko Finance (CDP) — LOW PRIORITY
- **TVL:** ~$650K (tiny)
- **Type:** Liquity-style CDP, mints KEI stablecoin
- **Collateral:** HYPE, wstHYPE, LHYPE, PURR, UBTC
- **Note:** Too small TVL to justify integration effort.

### 6. Laminar — SKIP (not lending)
- **Type:** Liquidity aggregator/DEX router, NOT a lending protocol
- **Note:** Confused in some sources as "lending" but actually a swap aggregator.

### 7. Kinetiq — SKIP (liquid staking, not lending)
- **Type:** Liquid staking protocol (kHYPE)
- **Note:** LST, not lending/borrowing. No liquidations.

### 8. Harmonix — LOW PRIORITY
- **Type:** Delta-neutral yield + lending
- **TVL:** Unknown
- **Note:** Mentioned but scarce details. Need more research.

### 9. Liminal — LOW PRIORITY
- **Type:** Leveraged lending marketplace for clean leverage loops
- **TVL:** Unknown
- **Note:** Niche protocol. Need more research.

### 10. Perps Hub — SKIP (derivatives, not lending)
- **Type:** Permissionless perpetual futures

### HYPEREVM SUMMARY
| Protocol | TVL | Priority | Liquidation Type | Code Reuse |
|----------|-----|----------|-----------------|------------|
| **HyperLend** | ~$380M | DONE | Aave V3 flash loan | Already built |
| **HypurrFi** | ~$350M | **HIGHEST** | Euler V2 (EVC) | New module needed |
| **Morpho Blue** | ~$431M | DONE | Morpho callback | Already built |
| **Felix CDP** | ~$265M | HIGH | Liquity V2 | New module needed |
| **Felix Vanilla** | (in Morpho) | DONE | Morpho callback | Covered by Morpho |
| **Keiko** | ~$650K | SKIP | CDP | Too small |
| **Harmonix** | Unknown | LOW | Unknown | Need research |
| **Liminal** | Unknown | LOW | Unknown | Need research |

---

## KEY FINDINGS

### 1. HypurrFi is the biggest missed opportunity
- $350M TVL on HyperEVM that we WEREN'T tracking
- Euler V2-based = well-documented liquidation mechanism
- GitHub: github.com/euler-xyz/liquidation-bot-v2 (open-source reference bot)
- Euler Vault Connector enables atomic liquidations without flash loans
- **Action:** Add HypurrFi/Euler V2 module to multi-chain-liquidator

### 2. Lendle on Mantle is the easiest win
- Aave V2 interface = 95% code reuse from our existing Aave V3 monitor
- $6.4M TVL, been operating since 2023
- Must have had liquidations historically
- **Action:** Add Lendle (Aave V2) support with minimal changes

### 3. Felix CDP on HyperEVM is large but requires new code
- $265M TVL in CDP system
- Liquity V2-style liquidation = need new module
- The Morpho Vanilla part is already covered

### 4. Compound V3 on Mantle needs investigation
- Deployed, active governance, oracle updates
- Need to find contract addresses
- Different liquidation mechanism (absorb)

### 5. Ink is truly Tydro-only for now
- No other meaningful lending protocol on Ink
- Morpho is deployed but empty ($1.41 TVL)
- All liquidation opportunity on Ink = Tydro/Aave V3

---

## PRIORITY ACTION PLAN

### Immediate (this session)
1. **HypurrFi research:** Find contract addresses, understand Euler V2 liquidation flow, check open-source bot
2. **Lendle integration:** Add Aave V2 support to our Aave module (minimal diff from V3)

### Next session
3. **Compound V3 on Mantle:** Find comet address, study absorb() mechanism
4. **Felix CDP on HyperEVM:** Study Liquity V2 liquidation flow
5. **Fluid on Mantle:** Find contract addresses, check TVL

### Later
6. **INIT Capital on Mantle:** Study custom hook liquidation
7. **Monitor Morpho on Ink:** Check if markets become active

---

## Sources
- DeFiLlama: defillama.com/chain/Mantle, /chain/Ink, /chain/hyperliquid-l1
- Lendle docs: docs.lendle.xyz/contracts-and-security/mantle-contracts
- INIT Capital docs: dev.init.capital/contract-addresses/mantle
- Morpho docs: docs.morpho.org/get-started/resources/addresses/
- Fluid docs: docs.fluid.instadapp.io/contracts/contract-addresses.html
- Mantle dApps: mantle.xyz/dapp
- HypurrFi docs: docs.hypurr.fi
- Felix: alearesearch.substack.com/p/felix-protocol-native-money-markets
- Compound Mantle: comp.xyz/t/deploy-compound-iii-on-mantle-network/5774
- Aurelius: aurelius.finance
- Gravita: gravitaprotocol.com
