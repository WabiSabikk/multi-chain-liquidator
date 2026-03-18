# CORRECTED: HyperEVM Liquidation Activity Research

**Date:** 2026-03-06
**Status:** CRITICAL CORRECTIONS to 2026-03-05 findings
**Previous report:** WHY-ZERO-LIQUIDATIONS-2026-03-05.md -- had WRONG data for Morpho and Felix

## TL;DR

**Previous conclusion was WRONG.** Morpho Blue and Felix CDP DO have significant liquidation activity on HyperEVM. The errors were:

1. **Morpho Blue:** Wrong contract address. Used canonical `0xBBBBBbb...` which has NO CODE on HyperEVM. Actual address: `0x68e37dE8d93d3496ae143F2E900490f6280C57cD`
2. **Morpho Blue:** Wrong event topic. Used 9-param signature (with `seizedShares`) but actual event has 8 params (no `seizedShares`)
3. **ALL keccak256 hashes:** Used `hashlib.sha3_256` which is **NIST SHA-3, NOT Ethereum's keccak-256**. These are different algorithms! Every topic hash computed with `hashlib.sha3_256` was wrong.
4. **Felix CDP:** Liquidations exist but are encoded as `urgentRedemption` (operation type 6) inside `TroveOperation` events, not as separate `TroveLiquidated` events.

| Protocol | Previous Finding | CORRECTED Finding |
|----------|-----------------|-------------------|
| **Morpho Blue** | "ZERO liquidations" | **2,000-3,000+ liquidations** (May 2025 - Feb 2026) |
| **Felix CDP** | "ZERO liquidations" | **83+ urgentRedemptions** on wstHYPE TroveManager alone (Aug 2025) |
| **HypurrFi Euler V2** | "ZERO liquidations" | Still unclear - could not find vault contract addresses |

---

## 1. Morpho Blue on HyperEVM -- MASSIVE Liquidation Activity

### Critical Error: Wrong Contract Address

The CLAUDE.md listed Morpho Core as `0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb`. This is the **canonical CREATE2 address** used on Ethereum mainnet, Base, etc. But on HyperEVM, **this address has NO deployed code**.

**Actual Morpho Blue address on HyperEVM:** `0x68e37dE8d93d3496ae143F2E900490f6280C57cD`

Full HyperEVM Morpho deployment (from docs.morpho.org/addresses):

| Contract | Address |
|----------|---------|
| **Morpho Core** | `0x68e37dE8d93d3496ae143F2E900490f6280C57cD` |
| Adaptive Curve Irm | `0xD4a426F010986dCad727e8dd6eed44cA4A9b7483` |
| Morpho Chainlink Oracle V2 Factory | `0xeb476f124FaD625178759d13557A72394A6f9aF5` |
| MetaMorpho Factory V1.1 | `0xec051b19d654C48c357dC974376DeB6272f24e53` |
| Public Allocator | `0x517505be22D9068687334e69ae7a02fC77edf4Fc` |
| PreLiquidation Factory | `0x1b6782Ac7A859503cE953FBf4736311CC335B8f0` |
| VaultV2Factory | `0xD7217E5687FF1071356C780b5fe4803D9D967da7` |
| MorphoVaultV1 AdapterFactory | `0xdf5202e29654e02011611A086f15477880580CAc` |
| MorphoMarketV1 AdapterV2Factory | `0xaEff6Ef4B7bbfbAadB18b634A8F11392CBeB72Be` |
| MorphoRegistry | `0x857B55cEb57dA0C2A83EE08a8dB529B931089aee` |

### Critical Error: Wrong Event Signature

The task description said Morpho's Liquidate event has 9 parameters including `seizedShares`. **Wrong.** The actual event from `EventsLib.sol`:

```solidity
event Liquidate(
    Id indexed id,          // bytes32 (market id)
    address indexed caller,  // liquidator
    address indexed borrower,
    uint256 repaidAssets,
    uint256 repaidShares,
    uint256 seizedAssets,    // NO seizedShares!
    uint256 badDebtAssets,
    uint256 badDebtShares
);
```

**8 parameters, not 9.** The ABI signature is:
`Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)`

**Correct keccak-256 topic:**
`0xa4946ede45d0c6f06a0f5ce92c9ad3b4751452d2fe0e25010783bcab57a67e41`

### Critical Error: SHA-3 vs Keccak-256

Python's `hashlib.sha3_256` implements **NIST SHA-3 (FIPS 202)**, which is different from Ethereum's **keccak-256** (pre-NIST version). They produce completely different hashes for the same input:

```python
# WRONG (NIST SHA-3):
hashlib.sha3_256(b"Upgraded(address)").hexdigest()
# = 0x0e51260f13833a...  (WRONG!)

# CORRECT (keccak-256 via pycryptodome):
from Crypto.Hash import keccak
keccak.new(digest_bits=256, data=b"Upgraded(address)").hexdigest()
# = 0xbc7cd75a20ee27...  (CORRECT!)
```

To compute correct Ethereum event topics, use:
- `web3.py`: `Web3.keccak(text="...")`
- `pycryptodome`: `from Crypto.Hash import keccak`
- `pysha3`: `import sha3; sha3.keccak_256()`
- **NEVER** `hashlib.sha3_256`

### Actual Liquidation Data

**Period:** May 14, 2025 -- ~February 2, 2026 (8.5 months)
**Total:** 2,000 - 3,000+ liquidation events (conservative estimate based on paginated API)
**Status as of March 6, 2026:** No liquidations in the last ~32 days, but protocol is ACTIVE (72 events in 500 blocks including supply, withdraw, borrow, repay, flash loans)

**Key metrics (from first 1000 events, May-Sep 2025):**
- 55 unique liquidator addresses
- 34+ unique borrowers liquidated
- First liquidation: block 3,787,882 (May 14, 2025)

**Top liquidators:**
| Address | Count (first 1000) |
|---------|-------------------|
| `0xbcccf9e4b0048a9dc5af93036288502bde1a71b1` | 192 |
| `0x9369fa6f4d3d32225e1629b04ef308e0eb568fb0` | 183 |
| `0x2ed17846e0d1c4cc28236d95b6eb3b12dcc86909` | 87 |
| `0xcdf6af1093d29478aff5f1ccd93cc67f8aadfddc` | 49 |
| `0xabf1321cabec8486dca5a9a96fb5202184106e54` | 34 |

**Why no recent liquidations (last 32 days):**
- HYPE price stabilized above $30 (currently $30.58)
- After Jan/Feb 2026 liquidation wave, remaining positions have healthy LTV
- TVL dropped from $4.7B to $4.2B ecosystem-wide (risk-off)
- Borrowers became more cautious after previous waves

### Impact on Our Bot

Morpho Blue on HyperEVM is a **proven, active liquidation market** with established competition (55 bots). Key considerations:
- Callback mechanism (no flash loans needed) = lower barrier
- 5 active competing bots dominate (top 5 = 545 out of 1000 liquidations)
- $119M borrowed = significant opportunity when next volatility hits
- Need to monitor HYPE price drops below $25-28 for liquidation cascade triggers

---

## 2. Felix CDP -- urgentRedemption Activity Found

### How Felix Liquidations Work

Felix is a Liquity V2 fork. It does NOT emit separate `TroveLiquidated` events. Instead, all trove operations (open, close, adjust, liquidate, redeem) are encoded in a single event:

```solidity
event TroveOperation(
    uint256 indexed _troveId,
    uint8 _operation,      // 0=open, 1=close, 2=adjust, 3=applyPending, 4=liquidate, 5=redeem, 6=urgentRedemption
    uint256 _annualInterestRate,
    uint256 _debtDelta,
    uint256 _collDelta,
    int256 _debtChange,
    uint256 _stake,
    int256 _collChange
);
```

**Topic:** `0x962110f281c1213763cd97a546b337b3cbfd25a31ea9723e9d8b7376ba45da1a`

### Felix Contract Addresses (from usefelix.xyz)

| Collateral | TroveManager | StabilityPool |
|-----------|-------------|---------------|
| HYPE (`0x5555...`) | `0x3100f4e7bda2ed2452d9a57eb30260ab071bbe62` | `0x576c9c501473e01ae23748de28415a74425efd6b` |
| feUBTC (`0xefbd...`) | `0xbbe5f227275f24b64bd290a91f55723a00214885` | `0xabf0369530205ae56dd4c9629474c65d1168924` |
| kHYPE (`0xfd73...`) | `0x7c07bb77b1cf9a5b40d92f805c10d90c90957e4a` | `0x56a346e0730cb209a93964c41cd36098030779ab` |
| wstHYPE (`0x94e8...`) | `0x58446c58caa8a6f6cc8be343f812ebf0b997c001` | `0xadfba621a75beced7dd1727b2067047b7eeedc8b` |

### Actual Liquidation Activity by TroveManager

| TroveManager | Total Ops | openTrove | adjustTrove | urgentRedemption | liquidate |
|-------------|-----------|-----------|-------------|------------------|-----------|
| HYPE | 100+ | 23 | 67 | 0 | 0 |
| feUBTC | 100+ | 18 | 55 | 0 | 0 |
| kHYPE | 100+ | 36 | 41 | 1 | 0 |
| **wstHYPE** | **2000+** | ~23 | ~160+ | **83+ (page 1) + 87+ (page 2) + more** | 0 |

**wstHYPE TroveManager has MASSIVE urgentRedemption activity:**
- First urgentRedemption: August 27, 2025 16:09 UTC
- Concentrated burst: Aug 27-28, 2025 (83 in page 1)
- Continued through September 2025 (87+ more in page 2)
- Only 2 unique troves affected in initial burst
- No urgentRedemptions after ~November 2025

**What is urgentRedemption (operation type 6)?**
In Liquity V2, urgentRedemption allows anyone to redeem feUSD against troves that fall below the minimum collateral ratio (MCR). It's the closest equivalent to a standard liquidation -- the undercollateralized trove's collateral is taken and feUSD debt is destroyed. This is different from normal redemption (type 5) which operates on any trove in order of interest rate.

### Why wstHYPE Had Liquidations But Not Others

wstHYPE (wrapped staked HYPE) has additional price risk:
- HYPE price drops = wstHYPE drops
- staking derivatives can depeg from underlying (liquid staking risk)
- wstHYPE/HYPE ratio changes add volatility on top of HYPE price

HYPE and kHYPE (another liquid staking derivative) had minimal activity because:
- HYPE is the native gas token with deeper liquidity and less "double risk"
- kHYPE may have more conservative LTV settings
- feUBTC uses Bitcoin as collateral (different risk profile)

### Impact on Our Bot

Felix urgentRedemptions are a liquidation opportunity, but:
- Volume is low and concentrated in bursts (needs price crash)
- Stability Pool handles most liquidations automatically
- urgentRedemption is essentially the "overflow" when SP is depleted
- Only profitable during extreme volatility events
- Current Felix TVL: $36M (relatively small)

---

## 3. HypurrFi Euler V2 Isolated -- Still Unclear

### Correct Euler V2 Liquidate Topic

```solidity
event Liquidate(
    address indexed liquidator,
    address indexed violator,
    address collateral,
    uint256 repayAssets,
    uint256 yieldBalance
);
```

**Correct keccak-256 topic:** `0x8246cc71ab01533b5bebc672a636df812f10637ad720797319d5741d5ebb3962`

### What We Know

- HypurrFi POOLED markets (Aave V3 Pool at `0xcecce0...`) have 4,000+ liquidations (confirmed in previous research)
- HypurrFi ISOLATED markets use Euler V2 Vault Kit architecture
- Could not find specific Euler V2 vault addresses for HypurrFi isolated markets
- The Routescan chain-wide topic search returned events from other chains (not chain-specific)
- TVL is only $1.6M (very small)

### Why Possibly Zero Liquidations

- $1.6M TVL is tiny -- may have very few borrowers
- Euler V2 isolated vaults have per-vault risk parameters
- Each vault is independent -- need to find specific vault addresses to check
- HypurrFi may have conservative LTV settings on isolated vaults

---

## 4. Correct Event Topic Hashes (keccak-256, verified)

| Protocol | Event | Topic Hash |
|----------|-------|-----------|
| **Morpho Blue** | `Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)` | `0xa4946ede45d0c6f06a0f5ce92c9ad3b4751452d2fe0e25010783bcab57a67e41` |
| **Morpho PreLiq** | `PreLiquidate(bytes32,address,address,uint256,uint256,uint256,uint256)` | `0xed58daccd6eb019eb5c858581c9d558f7be7168eba4fb217c562fdb8bfcc0942` |
| **Aave V3** | `LiquidationCall(address,address,address,uint256,uint256,address,bool)` | `0xe413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286` |
| **Euler V2** | `Liquidate(address,address,address,uint256,uint256)` | `0x8246cc71ab01533b5bebc672a636df812f10637ad720797319d5741d5ebb3962` |
| **Felix/Liquity V2** | `TroveOperation(uint256,uint8,uint256,uint256,uint256,int256,uint256,int256)` | `0x962110f281c1213763cd97a546b337b3cbfd25a31ea9723e9d8b7376ba45da1a` |

---

## 5. Lessons Learned

1. **NEVER use `hashlib.sha3_256` for Ethereum topics.** Use `pycryptodome` keccak or `web3.py`.
2. **Always verify contract addresses on-chain** with `eth_getCode`. Canonical CREATE2 addresses may differ per chain.
3. **Liquity V2 forks encode liquidations inside TroveOperation events**, not as separate events. Must decode the `_operation` parameter.
4. **Morpho Blue event signature has 8 params, not 9.** Always check the actual source code in `EventsLib.sol`.
5. **Routescan API without address filter may return cross-chain results** -- always filter by contract address.

---

## 6. Updated Bot Strategy Implications

### Morpho Blue on HyperEVM (HIGH PRIORITY)
- **2,000-3,000+ historical liquidations** with 55 competing bots
- Correct contract: `0x68e37dE8d93d3496ae143F2E900490f6280C57cD`
- Callback mechanism = $0 capital needed
- 5% LIF bonus on some markets
- Last liquidation ~Feb 2, 2026 -- needs next HYPE price drop to activate
- **ACTION:** Update CLAUDE.md with correct Morpho address

### Felix CDP (LOW PRIORITY)
- urgentRedemptions are rare and concentrated in crashes
- Stability Pool absorbs most liquidations automatically
- Small TVL ($36M)
- Only profitable during extreme events

### HypurrFi Euler V2 Isolated (VERY LOW PRIORITY)
- $1.6M TVL = too small
- Cannot find vault addresses
- Pooled side (Aave V3) already has competition (4000+ liqS)
