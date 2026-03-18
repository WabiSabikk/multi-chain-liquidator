#!/usr/bin/env node
/**
 * Historical liquidation backfill — Phase 7.5.1
 * Scans all LiquidationCall (Aave V2/V3) and Liquidate (Morpho) events.
 *
 * Usage:
 *   node scripts/backfill-liquidations.mjs --all
 *   node scripts/backfill-liquidations.mjs --protocol=lendle
 *   node scripts/backfill-liquidations.mjs --protocol=hyperlend,morpho
 *   node scripts/backfill-liquidations.mjs --protocol=hyperlend --rpc=http://localhost:3001/evm
 */

import { readFileSync, writeFileSync, appendFileSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const DATA_DIR = join(ROOT, 'data');
const PROGRESS_DIR = join(DATA_DIR, 'backfill-progress');

[DATA_DIR, PROGRESS_DIR].forEach(d => { if (!existsSync(d)) mkdirSync(d, { recursive: true }); });

// --- Load .env ---
function loadEnv() {
  const p = join(ROOT, '.env');
  if (!existsSync(p)) return {};
  const env = {};
  readFileSync(p, 'utf8').split('\n').forEach(line => {
    const eq = line.indexOf('=');
    if (eq > 0 && !line.startsWith('#')) env[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
  });
  return env;
}
const dotenv = loadEnv();

const RPCS = {
  mantle: dotenv.MANTLE_RPC_URL || 'https://rpc.mantle.xyz',
  ink: dotenv.INK_RPC_URL || 'https://rpc-gel.inkonchain.com',
  hyperevm: dotenv.HYPEREVM_RPC_URL || 'https://rpc.hyperliquid.xyz/evm',
};

// --- Event topics ---
const TOPIC_LIQUIDATION_CALL = '0xe413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286';
const TOPIC_MORPHO_LIQUIDATE = '0xa4946ede45d0c6f06a0f5ce92c9ad3b4751452d2fe0e25010783bcab57a67e41';

// --- Protocol configs ---
const PROTOCOLS = {
  lendle: {
    chain: 'mantle', address: '0xCFa5aE7c2CE8Fadc6426C1ff872cA45378Fb7cF3',
    topic: TOPIC_LIQUIDATION_CALL, startBlock: 62_000_000,
    type: 'aave', blockStep: 10_000, rateLimit: 10,
  },
  'mantle-aave-v3': {
    chain: 'mantle', address: '0x458F293454fE0d67EC0655f3672301301DD51422',
    topic: TOPIC_LIQUIDATION_CALL, startBlock: 91_500_000,
    type: 'aave', blockStep: 10_000, rateLimit: 10,
  },
  tydro: {
    chain: 'ink', address: '0x2816cf15f6d2a220e789aa011d5ee4eb6c47feba',
    topic: TOPIC_LIQUIDATION_CALL, startBlock: 1_000_000,
    type: 'aave', blockStep: 10_000, rateLimit: 10,
  },
  hyperlend: {
    chain: 'hyperevm', address: '0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b',
    topic: TOPIC_LIQUIDATION_CALL, startBlock: 20_000_000,
    type: 'aave', blockStep: 1_000, rateLimit: 8,
  },
  hypurrfi: {
    chain: 'hyperevm', address: '0xcecce0eb9dd2ef7996e01e25dd70e461f918a14b',
    topic: TOPIC_LIQUIDATION_CALL, startBlock: 1_000_000,
    type: 'aave', blockStep: 1_000, rateLimit: 8,
  },
  morpho: {
    chain: 'hyperevm', address: '0x68e37dE8d93d3496ae143F2E900490f6280C57cD',
    topic: TOPIC_MORPHO_LIQUIDATE, startBlock: 4_000_000,
    type: 'morpho', blockStep: 1_000, rateLimit: 8,
  },
};

// --- RPC ---
let rpcId = 1;

async function rpcCall(url, method, params, retries = 4) {
  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 30000);
      const res = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ jsonrpc: '2.0', id: rpcId++, method, params }),
        signal: controller.signal,
      });
      clearTimeout(timeout);
      const json = await res.json();
      if (json.error) {
        const msg = json.error.message || '';
        // "too many results" — halve block step
        if (msg.includes('too many') || msg.includes('query returned more than') || msg.includes('exceed') || msg.includes('ranges over')) {
          return { _tooMany: true };
        }
        // Rate limited — backoff
        if (msg.includes('rate') || json.error.code === 429 || json.error.code === -32005 || json.error.code === -32016) {
          await sleep(2000 * (attempt + 1));
          continue;
        }
        throw new Error(`RPC: ${msg} (${json.error.code})`);
      }
      return json.result;
    } catch (err) {
      if (err.name === 'AbortError') err.message = 'timeout';
      if (attempt < retries - 1) {
        await sleep(1500 * (attempt + 1));
        continue;
      }
      throw err;
    }
  }
}

const sleep = ms => new Promise(r => setTimeout(r, ms));
const toHex = n => '0x' + n.toString(16);
const parseAddr = h => '0x' + h.slice(-40).toLowerCase();
const parseU256 = h => BigInt('0x' + h).toString();

// --- Decoders ---
function decodeAave(log, chain, protocol) {
  const d = log.data.slice(2);
  return {
    chain, protocol,
    block: parseInt(log.blockNumber, 16),
    txHash: log.transactionHash,
    txIndex: parseInt(log.transactionIndex, 16),
    logIndex: parseInt(log.logIndex, 16),
    liquidator: parseAddr(d.slice(128, 192)),
    borrower: parseAddr(log.topics[3]),
    debtAsset: parseAddr(log.topics[2]),
    debtCovered: parseU256(d.slice(0, 64)),
    collateralAsset: parseAddr(log.topics[1]),
    collateralSeized: parseU256(d.slice(64, 128)),
  };
}

function decodeMorpho(log, chain, protocol) {
  const d = log.data.slice(2);
  return {
    chain, protocol,
    block: parseInt(log.blockNumber, 16),
    txHash: log.transactionHash,
    txIndex: parseInt(log.transactionIndex, 16),
    logIndex: parseInt(log.logIndex, 16),
    marketId: log.topics[1],
    liquidator: parseAddr(d.slice(0, 64)),
    borrower: parseAddr(d.slice(64, 128)),
    debtCovered: parseU256(d.slice(128, 192)),
    collateralSeized: parseU256(d.slice(256, 320)),
    badDebtAssets: parseU256(d.slice(320, 384)),
  };
}

// --- Progress ---
function loadProgress(proto) {
  const f = join(PROGRESS_DIR, `${proto}.json`);
  return existsSync(f) ? JSON.parse(readFileSync(f, 'utf8')) : { lastBlock: 0, totalEvents: 0 };
}
function saveProgress(proto, lastBlock, totalEvents) {
  writeFileSync(join(PROGRESS_DIR, `${proto}.json`),
    JSON.stringify({ lastBlock, totalEvents, ts: new Date().toISOString() }));
}

// --- Main backfill ---
async function backfill(protoName, rpcOverride) {
  const cfg = PROTOCOLS[protoName];
  if (!cfg) throw new Error(`Unknown: ${protoName}`);

  const rpcUrl = rpcOverride || RPCS[cfg.chain];
  const outFile = join(DATA_DIR, `historical-liqs-${cfg.chain}-${protoName}.jsonl`);
  const decode = cfg.type === 'morpho' ? decodeMorpho : decodeAave;

  console.log(`\n${'='.repeat(60)}`);
  console.log(`BACKFILL: ${protoName} (${cfg.chain})`);
  console.log(`Pool: ${cfg.address}`);

  const currentBlock = parseInt(await rpcCall(rpcUrl, 'eth_blockNumber', []), 16);
  console.log(`Current block: ${currentBlock.toLocaleString()}`);

  const progress = loadProgress(protoName);
  let startBlock = Math.max(cfg.startBlock, progress.lastBlock + 1);
  let totalEvents = progress.totalEvents;
  let blockStep = cfg.blockStep;

  if (startBlock >= currentBlock) {
    console.log(`Up to date. ${totalEvents} events.`);
    return totalEvents;
  }

  const totalBlocks = currentBlock - startBlock;
  console.log(`Scanning ${startBlock.toLocaleString()} → ${currentBlock.toLocaleString()} (${totalBlocks.toLocaleString()} blocks)`);

  const minInterval = 1000 / cfg.rateLimit;
  let lastReqTime = 0;
  let chunksDone = 0;
  let errors = 0;
  const t0 = Date.now();
  let from = startBlock;

  while (from <= currentBlock) {
    const to = Math.min(from + blockStep - 1, currentBlock);

    // Rate limit
    const now = Date.now();
    if (now - lastReqTime < minInterval) await sleep(minInterval - (now - lastReqTime));
    lastReqTime = Date.now();

    try {
      const logs = await rpcCall(rpcUrl, 'eth_getLogs', [{
        address: cfg.address,
        topics: [cfg.topic],
        fromBlock: toHex(from),
        toBlock: toHex(to),
      }]);

      // Handle "too many results" — halve step
      if (logs && logs._tooMany) {
        blockStep = Math.max(100, Math.floor(blockStep / 2));
        console.log(`  Too many results, reducing step to ${blockStep}`);
        continue; // retry same `from`
      }

      if (logs && logs.length > 0) {
        for (const log of logs) {
          try {
            const ev = decode(log, cfg.chain, protoName);
            appendFileSync(outFile, JSON.stringify(ev) + '\n');
            totalEvents++;
          } catch (e) {
            console.error(`  Decode err block ${parseInt(log.blockNumber, 16)}: ${e.message}`);
          }
        }
      }

      errors = Math.max(0, errors - 1);
      // Gradually restore step after "too many" reduction
      if (blockStep < cfg.blockStep) blockStep = Math.min(cfg.blockStep, blockStep + 100);
    } catch (err) {
      errors++;
      console.error(`  Error ${from}-${to}: ${err.message}`);
      if (errors > 10) {
        cfg.rateLimit = Math.max(1, cfg.rateLimit * 0.5);
        console.log(`  Throttle → ${cfg.rateLimit.toFixed(1)} req/s`);
        errors = 0;
      }
      await sleep(3000);
      continue; // retry same `from`
    }

    from = to + 1;
    chunksDone++;

    // Progress every 200 chunks or at end
    if (chunksDone % 200 === 0 || from > currentBlock) {
      const pct = ((to - startBlock) / totalBlocks * 100).toFixed(1);
      const elapsed = (Date.now() - t0) / 1000;
      const rate = chunksDone / elapsed;
      const eta = rate > 0 ? (currentBlock - to) / blockStep / rate : 0;
      console.log(`  ${pct}% | block ${to.toLocaleString()} | ${totalEvents} events | ${rate.toFixed(1)} ch/s | ETA ${fmtTime(eta)}`);
      saveProgress(protoName, to, totalEvents);
    }
  }

  saveProgress(protoName, currentBlock, totalEvents);
  console.log(`\n✅ ${protoName}: ${totalEvents} events → ${outFile}`);
  return totalEvents;
}

function fmtTime(s) {
  if (s < 60) return `${s.toFixed(0)}s`;
  if (s < 3600) return `${(s / 60).toFixed(1)}m`;
  return `${(s / 3600).toFixed(1)}h`;
}

// --- CLI ---
async function main() {
  const args = process.argv.slice(2);
  let protocols = [];
  let rpcOverride = null;

  for (const a of args) {
    if (a === '--all') protocols = Object.keys(PROTOCOLS);
    else if (a.startsWith('--protocol=')) protocols = a.slice(11).split(',').map(s => s.trim());
    else if (a.startsWith('--rpc=')) rpcOverride = a.slice(6);
  }

  if (!protocols.length) {
    console.log('Usage: node backfill-liquidations.mjs --all | --protocol=name[,name2]');
    console.log('Protocols:', Object.keys(PROTOCOLS).join(', '));
    console.log('Options: --rpc=URL (override RPC for all)');
    process.exit(1);
  }

  console.log(`Backfill: ${protocols.join(', ')}`);
  console.log(`Time: ${new Date().toISOString()}\n`);

  const results = {};
  for (const p of protocols) {
    try {
      results[p] = await backfill(p, rpcOverride);
    } catch (err) {
      console.error(`❌ ${p}: ${err.message}`);
      results[p] = -1;
    }
  }

  console.log('\n' + '='.repeat(60));
  console.log('SUMMARY');
  for (const [p, n] of Object.entries(results)) {
    console.log(`  ${p}: ${n >= 0 ? n + ' events' : 'FAILED'}`);
  }
}

main().catch(e => { console.error('Fatal:', e); process.exit(1); });
