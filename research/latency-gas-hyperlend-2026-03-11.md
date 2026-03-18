# Latency & Gas Analysis — hyperlend

**Date:** 2026-03-11
**Events analyzed:** 500
**Unique TXs:** 500

## Per-Bot Gas & Position Analysis

| # | Bot | Count | Med Gas Used | Med Gas Price (gwei) | Med txIndex | Avg Gas Cost |
|---|-----|-------|-------------|---------------------|------------|-------------|
| 1 | `0x8715ed...1cd3` | 89 | 1,185,184 | 0.1200 | 1.0 | 765151.05 gwei |
| 2 | `0x7a5635...5e80` | 78 | 726,740 | 3.8778 | 4.0 | 16212314.39 gwei |
| 3 | `0x5d5ecb...4953` | 56 | 601,945 | 0.3426 | 0.5 | 503190.91 gwei |
| 4 | `0x1ea48b...9d26` | 50 | 961,038 | 0.5940 | 1.0 | 3297693.77 gwei |
| 5 | `0x873a1c...b056` | 48 | 788,591 | 6.3014 | 4.5 | 13961390.52 gwei |
| 6 | `0xdd8692...60fa` | 45 | 885,090 | 3145.7964 | 0.0 | 5468484287.28 gwei |
| 7 | `0xa7d048...3a0f` | 37 | 719,352 | 302.0342 | 0.0 | 242951174.88 gwei |
| 8 | `0xfded1a...cfa6` | 25 | 727,639 | 66.5947 | 1.0 | 144978524.98 gwei |
| 9 | `0x99c821...a5dc` | 18 | 718,651 | 0.2000 | 1.0 | 739473.87 gwei |
| 10 | `0x5fe6fa...f826` | 7 | 678,268 | 61.5507 | 1.0 | 38199436.44 gwei |
| 11 | `0x2f18fc...a0ab` | 6 | 721,502 | 0.4067 | 3.0 | 1958919.84 gwei |
| 12 | `0xa6b761...1ff9` | 6 | 811,917 | 11.1029 | 4.0 | 66464730.85 gwei |
| 13 | `0xcc1d35...9184` | 5 | 868,256 | 33.6006 | 4.0 | 82864296.95 gwei |
| 14 | `0xab8410...251d` | 4 | 638,776 | 0.1039 | 1.5 | 80724.84 gwei |
| 15 | `0x7a87f7...2722` | 4 | 956,849 | 6.6295 | 1.5 | 62079646.87 gwei |
| 16 | `0xac88b5...bcc9` | 3 | 1,156,474 | 57.6382 | 7.0 | 57941050.01 gwei |
| 17 | `0x1f77ea...9865` | 3 | 823,412 | 45688.3348 | 0.0 | 36588280799.79 gwei |
| 18 | `0xb30fff...6c23` | 3 | 562,229 | 27.0226 | 3.0 | 88545674.40 gwei |
| 19 | `0xc459b1...f4dc` | 3 | 705,609 | 963.4104 | 0.0 | 671206883.09 gwei |
| 20 | `0x2d1762...945b` | 2 | 713,784 | 52.4032 | 10.0 | 36621530.59 gwei |

## Same-Block Competition

Blocks where multiple bots attempted liquidation:

No competitive blocks found in this dataset.

## Transaction Ordering: FCFS vs Gas Priority

If gas price correlates with lower txIndex → gas priority auction.
If no correlation → FCFS (first-come-first-served).

**Pearson correlation (gasPrice vs txIndex):** -0.1222
- If negative (< -0.3): higher gas → earlier in block = **gas priority**
- If near 0: **FCFS** or random
- Mean gas price: 840.6595 gwei
- Mean txIndex: 2.4

## Gas Price Distribution

| Metric | Value (gwei) |
|--------|-------------|
| Min | 0.1000 |
| P25 | 0.2000 |
| Median | 1.9522 |
| P75 | 65.4231 |
| Max | 69224.4064 |
