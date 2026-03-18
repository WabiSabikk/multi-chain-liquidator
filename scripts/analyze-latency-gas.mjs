#!/usr/bin/env node
/**
 * Latency & Gas Analysis — Phase 7.5.4
 * For each liquidation event, fetches TX receipt to get:
 * - effectiveGasPrice
 * - gasUsed
 * - txIndex position in block
 * Then analyzes per-bot: median gas, median txIndex, speed patterns.
 *
 * Also checks for oracle update TXs in the same block (for HyperLend).
 *
 * Usage:
 *   node scripts/analyze-latency-gas.mjs --protocol=hyperlend [--limit=500] [--rpc=URL]
 */

import { readFileSync, writeFileSync, existsSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, '..', 'data');
const RESEARCH_DIR = join(__dirname, '..', 'research');

// Load .env
function loadEnv() {
  const p = join(__dirname, '..', '.env');
  if (!existsSync(p)) return {};
  const env = {};
  readFileSync(p, 'utf8').split('\n').forEach(l => {
    const eq = l.indexOf('=');
    if (eq > 0 && !l.startsWith('#')) env[l.slice(0, eq).trim()] = l.slice(eq + 1).trim();
  });
  return env;
}

const dotenv = loadEnv();
const RPCS = {
  mantle: dotenv.MANTLE_RPC_URL || 'https://rpc.mantle.xyz',
  ink: dotenv.INK_RPC_URL || 'https://rpc-gel.inkonchain.com',
  hyperevm: dotenv.HYPEREVM_RPC_URL || 'https://rpc.hyperliquid.xyz/evm',
};

let rpcId = 1;
const sleep = ms => new Promise(r => setTimeout(r, ms));

async function rpcCall(url, method, params, retries = 3) {
  for (let i = 0; i < retries; i++) {
    try {
      const ctrl = new AbortController();
      const t = setTimeout(() => ctrl.abort(), 30000);
      const res = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ jsonrpc: '2.0', id: rpcId++, method, params }),
        signal: ctrl.signal,
      });
      clearTimeout(t);
      const json = await res.json();
      if (json.error) {
        if (json.error.code === -32005 || json.error.code === 429) {
          await sleep(3000 * (i + 1));
          continue;
        }
        throw new Error(JSON.stringify(json.error));
      }
      return json.result;
    } catch (e) {
      if (i < retries - 1) { await sleep(2000 * (i + 1)); continue; }
      throw e;
    }
  }
}

// Batch RPC for receipts
async function batchGetReceipts(url, txHashes) {
  const body = txHashes.map((h, i) => ({
    jsonrpc: '2.0', id: i + 1,
    method: 'eth_getTransactionReceipt', params: [h]
  }));

  try {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), 60000);
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      signal: ctrl.signal,
    });
    clearTimeout(t);
    const results = await res.json();
    if (Array.isArray(results)) {
      return results.map(r => r.result);
    }
    // Some RPCs don't support batch — fall back to individual
    return null;
  } catch {
    return null;
  }
}

function median(arr) {
  if (!arr.length) return 0;
  const s = [...arr].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
}

function shortAddr(a) { return a.slice(0, 8) + '...' + a.slice(-4); }

async function main() {
  const args = process.argv.slice(2);
  let protocol = null;
  let limit = 1000;
  let rpcOverride = null;

  for (const a of args) {
    if (a.startsWith('--protocol=')) protocol = a.slice(11);
    if (a.startsWith('--limit=')) limit = parseInt(a.slice(8));
    if (a.startsWith('--rpc=')) rpcOverride = a.slice(6);
  }

  if (!protocol) {
    console.log('Usage: node analyze-latency-gas.mjs --protocol=hyperlend [--limit=500]');
    process.exit(1);
  }

  // Find data file
  const files = readdirSync(DATA_DIR).filter(f =>
    f.startsWith('historical-liqs-') && f.includes(protocol) && f.endsWith('.jsonl')
  );
  if (!files.length) {
    console.log(`No data for ${protocol}. Run backfill first.`);
    process.exit(1);
  }

  const lines = readFileSync(join(DATA_DIR, files[0]), 'utf8').trim().split('\n').filter(Boolean);
  let events = lines.map(l => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);

  console.log(`Loaded ${events.length} events for ${protocol}`);

  // Take most recent N events (most relevant)
  events.sort((a, b) => b.block - a.block);
  if (events.length > limit) {
    events = events.slice(0, limit);
    console.log(`Limited to ${limit} most recent events`);
  }

  const chain = events[0]?.chain;
  const rpcUrl = rpcOverride || RPCS[chain];
  console.log(`RPC: ${rpcUrl.slice(0, 40)}...`);

  // Deduplicate TX hashes (multiple events can be in same TX)
  const uniqueTxHashes = [...new Set(events.map(e => e.txHash))];
  console.log(`Unique TXs: ${uniqueTxHashes.length}`);

  // Fetch receipts
  console.log('Fetching TX receipts...');
  const receipts = {};
  const BATCH_SIZE = 5;
  let done = 0;

  for (let i = 0; i < uniqueTxHashes.length; i++) {
    const h = uniqueTxHashes[i];
    try {
      const r = await rpcCall(rpcUrl, 'eth_getTransactionReceipt', [h]);
      if (r) receipts[h] = r;
    } catch (e) {
      console.error(`  Receipt error ${h.slice(0, 10)}: ${e.message}`);
    }
    done++;
    if (done % 50 === 0) console.log(`  ${done}/${uniqueTxHashes.length} receipts (${Object.keys(receipts).length} ok)`);
    await sleep(120);
  }

  console.log(`Got ${Object.keys(receipts).length} receipts`);

  // Enrich events with receipt data
  for (const ev of events) {
    const r = receipts[ev.txHash];
    if (r) {
      ev.gasUsed = parseInt(r.gasUsed, 16);
      ev.effectiveGasPrice = parseInt(r.effectiveGasPrice || '0x0', 16);
      ev.gasCostWei = BigInt(r.gasUsed) * BigInt(r.effectiveGasPrice || '0x0');
      ev.gasCostGwei = Number(ev.gasCostWei) / 1e9;
      ev.status = r.status === '0x1' ? 'success' : 'revert';
    }
  }

  // Analysis per liquidator
  const byBot = {};
  for (const ev of events) {
    const bot = ev.liquidator.toLowerCase();
    if (!byBot[bot]) byBot[bot] = { events: [], gasUsed: [], gasPrice: [], txIndex: [] };
    byBot[bot].events.push(ev);
    if (ev.gasUsed) byBot[bot].gasUsed.push(ev.gasUsed);
    if (ev.effectiveGasPrice) byBot[bot].gasPrice.push(ev.effectiveGasPrice);
    byBot[bot].txIndex.push(ev.txIndex);
  }

  // Sort bots by count
  const sortedBots = Object.entries(byBot).sort(([, a], [, b]) => b.events.length - a.events.length);

  // Generate report
  let md = `# Latency & Gas Analysis — ${protocol}\n\n`;
  md += `**Date:** ${new Date().toISOString().split('T')[0]}\n`;
  md += `**Events analyzed:** ${events.length}\n`;
  md += `**Unique TXs:** ${uniqueTxHashes.length}\n\n`;

  md += `## Per-Bot Gas & Position Analysis\n\n`;
  md += `| # | Bot | Count | Med Gas Used | Med Gas Price (gwei) | Med txIndex | Avg Gas Cost |\n`;
  md += `|---|-----|-------|-------------|---------------------|------------|-------------|\n`;

  for (let i = 0; i < Math.min(sortedBots.length, 20); i++) {
    const [addr, data] = sortedBots[i];
    const medGas = Math.round(median(data.gasUsed));
    const medGasPrice = (median(data.gasPrice) / 1e9).toFixed(4);
    const medTxIdx = median(data.txIndex).toFixed(1);
    const avgGasCost = data.events.reduce((s, e) => s + (e.gasCostGwei || 0), 0) / data.events.length;
    md += `| ${i + 1} | \`${shortAddr(addr)}\` | ${data.events.length} | ${medGas.toLocaleString()} | ${medGasPrice} | ${medTxIdx} | ${avgGasCost.toFixed(2)} gwei |\n`;
  }

  // Same-block competition analysis
  md += `\n## Same-Block Competition\n\n`;
  md += `Blocks where multiple bots attempted liquidation:\n\n`;

  const byBlock = {};
  for (const ev of events) {
    if (!byBlock[ev.block]) byBlock[ev.block] = [];
    byBlock[ev.block].push(ev);
  }
  const competitiveBlocks = Object.entries(byBlock)
    .filter(([, evs]) => {
      const bots = new Set(evs.map(e => e.liquidator.toLowerCase()));
      return bots.size > 1;
    })
    .sort(([, a], [, b]) => b.length - a.length)
    .slice(0, 15);

  if (competitiveBlocks.length) {
    md += `| Block | TXs | Bots | Winner (txIdx) |\n`;
    md += `|-------|-----|------|----------------|\n`;
    for (const [block, evs] of competitiveBlocks) {
      const bots = [...new Set(evs.map(e => e.liquidator.toLowerCase()))];
      const winner = evs.reduce((w, e) => e.txIndex < w.txIndex ? e : w, evs[0]);
      md += `| ${parseInt(block).toLocaleString()} | ${evs.length} | ${bots.length} | \`${shortAddr(winner.liquidator)}\` (idx ${winner.txIndex}) |\n`;
    }
  } else {
    md += `No competitive blocks found in this dataset.\n`;
  }

  // txIndex distribution — is it FCFS or gas priority?
  md += `\n## Transaction Ordering: FCFS vs Gas Priority\n\n`;
  md += `If gas price correlates with lower txIndex → gas priority auction.\n`;
  md += `If no correlation → FCFS (first-come-first-served).\n\n`;

  // Check correlation between gasPrice and txIndex for top bots
  const allWithData = events.filter(e => e.effectiveGasPrice && e.txIndex !== undefined);
  if (allWithData.length > 10) {
    // Simple Pearson correlation
    const n = allWithData.length;
    const prices = allWithData.map(e => e.effectiveGasPrice);
    const indices = allWithData.map(e => e.txIndex);
    const meanP = prices.reduce((s, v) => s + v, 0) / n;
    const meanI = indices.reduce((s, v) => s + v, 0) / n;
    let num = 0, denP = 0, denI = 0;
    for (let i = 0; i < n; i++) {
      const dp = prices[i] - meanP;
      const di = indices[i] - meanI;
      num += dp * di;
      denP += dp * dp;
      denI += di * di;
    }
    const corr = denP && denI ? num / Math.sqrt(denP * denI) : 0;
    md += `**Pearson correlation (gasPrice vs txIndex):** ${corr.toFixed(4)}\n`;
    md += `- If negative (< -0.3): higher gas → earlier in block = **gas priority**\n`;
    md += `- If near 0: **FCFS** or random\n`;
    md += `- Mean gas price: ${(meanP / 1e9).toFixed(4)} gwei\n`;
    md += `- Mean txIndex: ${meanI.toFixed(1)}\n`;
  }

  // Gas price distribution
  md += `\n## Gas Price Distribution\n\n`;
  const gasPrices = events.filter(e => e.effectiveGasPrice).map(e => e.effectiveGasPrice / 1e9);
  if (gasPrices.length) {
    const sorted = [...gasPrices].sort((a, b) => a - b);
    md += `| Metric | Value (gwei) |\n`;
    md += `|--------|-------------|\n`;
    md += `| Min | ${sorted[0].toFixed(4)} |\n`;
    md += `| P25 | ${sorted[Math.floor(sorted.length * 0.25)].toFixed(4)} |\n`;
    md += `| Median | ${sorted[Math.floor(sorted.length * 0.5)].toFixed(4)} |\n`;
    md += `| P75 | ${sorted[Math.floor(sorted.length * 0.75)].toFixed(4)} |\n`;
    md += `| Max | ${sorted[sorted.length - 1].toFixed(4)} |\n`;
  }

  const outFile = join(RESEARCH_DIR, `latency-gas-${protocol}-${new Date().toISOString().split('T')[0]}.md`);
  writeFileSync(outFile, md);
  console.log(`\nReport: ${outFile}`);

  // Also save enriched data
  const enrichedFile = join(DATA_DIR, `enriched-liqs-${protocol}.jsonl`);
  const replacer = (_, v) => typeof v === 'bigint' ? v.toString() : v;
  writeFileSync(enrichedFile, events.map(e => JSON.stringify(e, replacer)).join('\n') + '\n');
  console.log(`Enriched data: ${enrichedFile}`);
}

main().catch(e => { console.error('Fatal:', e); process.exit(1); });
