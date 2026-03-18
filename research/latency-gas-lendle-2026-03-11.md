# Latency & Gas Analysis — lendle

**Date:** 2026-03-11
**Events analyzed:** 300
**Unique TXs:** 273

## Per-Bot Gas & Position Analysis

| # | Bot | Count | Med Gas Used | Med Gas Price (gwei) | Med txIndex | Avg Gas Cost |
|---|-----|-------|-------------|---------------------|------------|-------------|
| 1 | `0x13418a...8759` | 108 | 12,643,320,800 | 0.0301 | 5.0 | 507215255.58 gwei |
| 2 | `0x4b8626...79d2` | 76 | 5,806,401,709 | 0.0220 | 2.0 | 131187953.50 gwei |
| 3 | `0xdf89e6...33eb` | 33 | 10,062,956,884 | 0.0301 | 1.0 | 297012042.16 gwei |
| 4 | `0x54b39b...da60` | 27 | 6,057,353,739 | 0.0201 | 3.0 | 116330533.03 gwei |
| 5 | `0x8df56c...4fb1` | 9 | 5,282,532,024 | 0.0201 | 1.0 | 107750452.38 gwei |
| 6 | `0xc5035f...fbc8` | 9 | 7,286,771,979 | 0.0250 | 1.0 | 195660686.69 gwei |
| 7 | `0x22a282...d1b4` | 5 | 8,807,401,197 | 0.0203 | 4.0 | 181632558.08 gwei |
| 8 | `0x3be30a...bae9` | 4 | 7,010,912,208 | 0.0301 | 1.0 | 206672144.79 gwei |
| 9 | `0xa92593...f8f0` | 3 | 5,943,439,628 | 0.1809 | 1.0 | 1092350855.89 gwei |
| 10 | `0x7d23a3...35d9` | 3 | 6,983,097,220 | 0.0241 | 12.0 | 201377501.62 gwei |
| 11 | `0xb52049...3df0` | 3 | 8,288,166,642 | 0.0241 | 1.0 | 210200002.28 gwei |
| 12 | `0x083cfa...9998` | 3 | 10,643,942,333 | 0.1809 | 4.0 | 1940356789.41 gwei |
| 13 | `0x67034b...f8ae` | 3 | 5,131,891,619 | 0.0203 | 1.0 | 127703757.45 gwei |
| 14 | `0xfca11b...e506` | 3 | 7,250,330,186 | 0.0201 | 1.0 | 129623685.65 gwei |
| 15 | `0x807bce...54a6` | 2 | 4,642,595,358 | 0.0201 | 1.5 | 93316166.69 gwei |
| 16 | `0x6af964...87f9` | 2 | 6,805,482,283 | 0.0241 | 10.5 | 164148232.65 gwei |
| 17 | `0xc3e3c1...0908` | 2 | 6,627,571,740 | 0.0600 | 1.5 | 410746523.12 gwei |
| 18 | `0x6c3594...f83d` | 2 | 9,407,398,006 | 0.0301 | 1.5 | 283633049.87 gwei |
| 19 | `0x6c4abf...e8dd` | 1 | 6,330,003,891 | 0.0203 | 1.0 | 128182578.79 gwei |
| 20 | `0xc89c32...ca33` | 1 | 8,564,002,672 | 0.2500 | 2.0 | 2141000668.00 gwei |

## Same-Block Competition

Blocks where multiple bots attempted liquidation:

| Block | TXs | Bots | Winner (txIdx) |
|-------|-----|------|----------------|
| 90,874,635 | 2 | 2 | `0x13418a...8759` (idx 22) |
| 90,877,358 | 2 | 2 | `0xc89c32...ca33` (idx 2) |
| 91,103,271 | 2 | 2 | `0x4b8626...79d2` (idx 13) |

## Transaction Ordering: FCFS vs Gas Priority

If gas price correlates with lower txIndex → gas priority auction.
If no correlation → FCFS (first-come-first-served).

**Pearson correlation (gasPrice vs txIndex):** -0.0490
- If negative (< -0.3): higher gas → earlier in block = **gas priority**
- If near 0: **FCFS** or random
- Mean gas price: 0.0301 gwei
- Mean txIndex: 4.9

## Gas Price Distribution

| Metric | Value (gwei) |
|--------|-------------|
| Min | 0.0200 |
| P25 | 0.0220 |
| Median | 0.0301 |
| P75 | 0.0301 |
| Max | 0.2500 |
