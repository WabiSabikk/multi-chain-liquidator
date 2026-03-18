#!/usr/bin/env node
/**
 * Competitor Census Analysis — Phase 7.5.2
 * Analyzes historical liquidation data from backfill to produce:
 * - Top liquidators per protocol
 * - Market share distribution
 * - Collateral/debt pair frequencies
 * - Activity patterns (crash clustering)
 * - Profit estimation (rough, based on seized amounts)
 *
 * Usage:
 *   node scripts/analyze-census.mjs
 *   node scripts/analyze-census.mjs --protocol=hyperlend
 */

import { readFileSync, writeFileSync, existsSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, '..', 'data');
const RESEARCH_DIR = join(__dirname, '..', 'research');

// Our known addresses (lowercase)
const OUR_ADDRESSES = new Set([
  '0xaa979a7fc2c112448638ab88518231cf82ec3f3b', // wallet
  '0xf2e6e7f255c46cc2353a8fd1d502f0c1920e1d43', // mantle aave v3
  '0xf4c17331c8dc453e8b5bab98559fd7f1aa1cad91', // lendle v3
  '0xd47223b7a191643ecddc4715c948d88d5a13bdd',  // ink v2 + morpho
  '0x17b7b1b73ffba773e6a92bcbc3b27538a427977c', // hyperevm aave
]);

// Token names (lowercase address → symbol)
const TOKENS = {
  // Mantle
  '0xdeaddead...1111': 'WETH', // shortened
  '0xdeaddeaddeaddeaddeaddeaddeaddeaddead1111': 'WETH',
  '0x78c1b0c915c4faa5fffa6cabf0219da63d7f4cb8': 'WMNT',
  '0x09bc4e0d864854c6afb6eb9a9cdf58ac190d0df9': 'USDC',
  '0x201eba5cc46d216ce6dc03f6a759e8e766e956ae': 'USDT(L)',
  '0x779ded0c9e1022225f8e0630b35a9b54be713736': 'USDT0',
  '0x5d3a1ff2b6bab83b63cd9ad0787074081a52ef34': 'USDe',
  '0x211cc4dd073734da055fbf44a2b4667d5e5fe5d2': 'sUSDe',
  '0xc96de26018a54d51c097160568752c4e3bd6c364': 'FBTC',
  '0xcda86a272531e8640cd7f1a92c01839911b90bb0': 'mETH',
  '0xe6829d9a7ee3040e1276fa75293bde931859e8fa': 'cmETH',
  '0x00000000efe302beaa2b3e6e1b18d08d69a9012a': 'AUSD',
  '0xcabae6f6ea1ecab08ad02fe02ce9a44f09aebfa2': 'WBTC',
  // HyperEVM
  '0x5555555555555555555555555555555555555555': 'wHYPE',
  '0xfd739d4e423301ce9385c1fb8850539d657c296d': 'kHYPE',
  '0x94e8396e0869c9f2200760af0621afd240e1cf38': 'wstHYPE',
  '0xd8fc8f0b03eba61f64d08b0bef69d80916e5dda9': 'beHYPE',
  '0x9fdbda0a5e284c32744d2f17ee5c74b284993463': 'UBTC',
  '0xbe6727b535545c67d5caa73dea54865b92cf7907': 'UETH',
  '0x068f321fa8fb9f0d135f290ef6a3e2813e1c8a29': 'USOL',
  '0xb88339cb7199b77e23db6e890353e22632ba630f': 'USDC',
  '0xb8ce59fc3717ada4c02eadf9682a9e934f625ebb': 'USDT0',
  '0xb50a96253abdf803d85efcdce07ad8becbc52bd5': 'USDHL',
  '0x111111a1a0667d36bd57c0a9f569b98057111111': 'USDH',
  '0x0ad339d66bf4aed5ce31c64bc37b3244b6394a77': 'USR',
  // Ink
  '0x4200000000000000000000000000000000000006': 'WETH',
  '0x2d270e6886d130d724215a266106e6832161eaed': 'USDC',
  '0x0200c29006150606b650577bbe7b6248f58470c1': 'USDT0',
  '0xa3d68b74bf0528fdd07263c60d6488749044914b': 'weETH',
  '0x9f0a74a92287e323eb95c1cd9ecdbeb0e397cae4': 'wrsETH',
  '0x2416092f143378750bb29b79ed961ab195cceea5': 'ezETH',
  '0xfc421ad3c883bf9e7c4f42de845c4e4405799e73': 'GHO',
  '0xe343167631d89b6ffc58b88d6b7fb0228795491d': 'USDG',
  '0x73e0c0d45e048d25fc26fa3159b0aa04bfa4db98': 'kBTC',
  '0xae4efbc7736f963982aacb17efa37fcbab924cb3': 'SolvBTC',
};

function tokenName(addr) {
  return TOKENS[addr.toLowerCase()] || addr.slice(0, 8);
}

function shortAddr(addr) {
  return addr.slice(0, 8) + '...' + addr.slice(-4);
}

// --- Load data ---
function loadEvents(protocol) {
  const files = readdirSync(DATA_DIR).filter(f =>
    f.startsWith('historical-liqs-') && f.endsWith('.jsonl') &&
    (!protocol || f.includes(protocol))
  );

  const events = [];
  for (const file of files) {
    const lines = readFileSync(join(DATA_DIR, file), 'utf8').trim().split('\n').filter(Boolean);
    for (const line of lines) {
      try {
        events.push(JSON.parse(line));
      } catch (e) {}
    }
  }
  return events;
}

// --- Analysis ---
function analyzeProtocol(events, protoName) {
  if (!events.length) return null;

  // Sort by block
  events.sort((a, b) => a.block - b.block);

  // Top liquidators
  const byLiquidator = {};
  for (const ev of events) {
    const liq = ev.liquidator.toLowerCase();
    if (!byLiquidator[liq]) byLiquidator[liq] = { count: 0, blocks: [], pairs: {} };
    byLiquidator[liq].count++;
    byLiquidator[liq].blocks.push(ev.block);
    const pair = `${tokenName(ev.collateralAsset)}→${tokenName(ev.debtAsset || 'unknown')}`;
    byLiquidator[liq].pairs[pair] = (byLiquidator[liq].pairs[pair] || 0) + 1;
  }

  const topLiquidators = Object.entries(byLiquidator)
    .sort(([, a], [, b]) => b.count - a.count)
    .slice(0, 20);

  // Pairs frequency
  const pairCounts = {};
  for (const ev of events) {
    const pair = `${tokenName(ev.collateralAsset)}→${tokenName(ev.debtAsset || 'unknown')}`;
    pairCounts[pair] = (pairCounts[pair] || 0) + 1;
  }
  const topPairs = Object.entries(pairCounts).sort(([, a], [, b]) => b - a).slice(0, 15);

  // Unique borrowers
  const borrowers = new Set(events.map(e => e.borrower.toLowerCase()));

  // Block clustering (find crash periods)
  const blockBuckets = {};
  for (const ev of events) {
    // Bucket by ~1000 blocks (~17min on HyperEVM, ~33min Mantle)
    const bucket = Math.floor(ev.block / 1000) * 1000;
    blockBuckets[bucket] = (blockBuckets[bucket] || 0) + 1;
  }
  const hotPeriods = Object.entries(blockBuckets)
    .filter(([, c]) => c >= 5)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10);

  // Our bot stats
  const ourEvents = events.filter(e => OUR_ADDRESSES.has(e.liquidator.toLowerCase()));

  return {
    total: events.length,
    uniqueLiquidators: Object.keys(byLiquidator).length,
    uniqueBorrowers: borrowers.size,
    blockRange: [events[0].block, events[events.length - 1].block],
    topLiquidators,
    topPairs,
    hotPeriods,
    ourEvents: ourEvents.length,
  };
}

// --- Report generation ---
function generateReport(analysisByProto) {
  const date = new Date().toISOString().split('T')[0];
  let md = `# Competitor Census — ${date}\n\n`;
  md += `Generated from historical backfill data (Phase 7.5.2)\n\n`;

  for (const [proto, data] of Object.entries(analysisByProto)) {
    if (!data) {
      md += `## ${proto}\n\nNo data available.\n\n`;
      continue;
    }

    md += `## ${proto}\n\n`;
    md += `- **Total liquidations:** ${data.total.toLocaleString()}\n`;
    md += `- **Unique liquidators (bots):** ${data.uniqueLiquidators}\n`;
    md += `- **Unique borrowers liquidated:** ${data.uniqueBorrowers}\n`;
    md += `- **Block range:** ${data.blockRange[0].toLocaleString()} → ${data.blockRange[1].toLocaleString()}\n`;
    md += `- **Our bot liquidations:** ${data.ourEvents}\n\n`;

    // Top liquidators table
    md += `### Top Liquidators\n\n`;
    md += `| # | Address | Count | Share | Top Pair | Is Us? |\n`;
    md += `|---|---------|-------|-------|----------|--------|\n`;
    for (let i = 0; i < data.topLiquidators.length; i++) {
      const [addr, info] = data.topLiquidators[i];
      const share = (info.count / data.total * 100).toFixed(1);
      const topPair = Object.entries(info.pairs).sort(([, a], [, b]) => b - a)[0];
      const isOurs = OUR_ADDRESSES.has(addr) ? 'YES' : '';
      md += `| ${i + 1} | \`${shortAddr(addr)}\` | ${info.count} | ${share}% | ${topPair[0]} (${topPair[1]}) | ${isOurs} |\n`;
    }

    // Top pairs
    md += `\n### Top Liquidation Pairs\n\n`;
    md += `| Pair (Collateral→Debt) | Count | Share |\n`;
    md += `|------------------------|-------|-------|\n`;
    for (const [pair, count] of data.topPairs) {
      md += `| ${pair} | ${count} | ${(count / data.total * 100).toFixed(1)}% |\n`;
    }

    // Hot periods
    if (data.hotPeriods.length > 0) {
      md += `\n### Crash Clusters (${'>'}=5 liqs per 1K-block window)\n\n`;
      md += `| Block Range | Liquidations |\n`;
      md += `|-------------|-------------|\n`;
      for (const [block, count] of data.hotPeriods) {
        md += `| ${parseInt(block).toLocaleString()} - ${(parseInt(block) + 999).toLocaleString()} | ${count} |\n`;
      }
    }

    md += '\n---\n\n';
  }

  // Summary: competition density
  md += `## Competition Density Summary\n\n`;
  md += `| Protocol | Total Liqs | Bots | Liqs/Bot | Our Share |\n`;
  md += `|----------|-----------|------|----------|-----------|\n`;
  for (const [proto, data] of Object.entries(analysisByProto)) {
    if (!data) continue;
    const liqPerBot = (data.total / data.uniqueLiquidators).toFixed(1);
    const ourShare = data.ourEvents > 0 ? (data.ourEvents / data.total * 100).toFixed(1) + '%' : '0%';
    md += `| ${proto} | ${data.total} | ${data.uniqueLiquidators} | ${liqPerBot} | ${ourShare} |\n`;
  }

  return md;
}

// --- Main ---
function main() {
  const args = process.argv.slice(2);
  let filterProto = null;
  for (const a of args) {
    if (a.startsWith('--protocol=')) filterProto = a.slice(11);
  }

  // Find all backfill files
  const files = readdirSync(DATA_DIR).filter(f =>
    f.startsWith('historical-liqs-') && f.endsWith('.jsonl')
  );

  if (!files.length) {
    console.log('No backfill data found in data/. Run backfill-liquidations.mjs first.');
    process.exit(1);
  }

  console.log('Found backfill files:', files.join(', '));

  // Group events by protocol
  const byProto = {};
  for (const file of files) {
    // Extract protocol from filename: historical-liqs-{chain}-{protocol}.jsonl
    const match = file.match(/historical-liqs-(.+)-(.+)\.jsonl/);
    if (!match) continue;
    const proto = match[2];
    if (filterProto && proto !== filterProto) continue;

    const lines = readFileSync(join(DATA_DIR, file), 'utf8').trim().split('\n').filter(Boolean);
    const events = lines.map(l => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);
    console.log(`  ${file}: ${events.length} events`);
    byProto[proto] = events;
  }

  // Analyze each protocol
  const analysis = {};
  for (const [proto, events] of Object.entries(byProto)) {
    console.log(`\nAnalyzing ${proto}...`);
    analysis[proto] = analyzeProtocol(events, proto);
    if (analysis[proto]) {
      console.log(`  ${analysis[proto].total} liqs, ${analysis[proto].uniqueLiquidators} bots`);
    }
  }

  // Generate report
  const report = generateReport(analysis);
  const outFile = join(RESEARCH_DIR, `competitor-census-${new Date().toISOString().split('T')[0]}.md`);
  writeFileSync(outFile, report);
  console.log(`\nReport: ${outFile}`);

  // Also print summary
  console.log('\n=== SUMMARY ===');
  for (const [proto, data] of Object.entries(analysis)) {
    if (!data) continue;
    console.log(`${proto}: ${data.total} liqs, ${data.uniqueLiquidators} bots, our: ${data.ourEvents}`);
    const top3 = data.topLiquidators.slice(0, 3).map(([a, i]) => `${shortAddr(a)}(${i.count})`).join(', ');
    console.log(`  Top 3: ${top3}`);
  }
}

main();
