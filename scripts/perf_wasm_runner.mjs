#!/usr/bin/env node
// SPDX-License-Identifier: MIT OR Apache-2.0

import fs from "node:fs";
import path from "node:path";

function parseArgs() {
  const args = process.argv.slice(2);
  const out = {};
  for (let i = 0; i < args.length; i += 2) {
    out[args[i].replace(/^--/, "")] = args[i + 1];
  }
  return out;
}

function stats(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  const median = sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
  const mean = sorted.reduce((a, b) => a + b, 0) / sorted.length;
  return {
    samples_ms: samples.map((v) => Number(v.toFixed(3))),
    mean_ms: Number(mean.toFixed(3)),
    median_ms: Number(median.toFixed(3)),
    min_ms: Number(Math.min(...samples).toFixed(3)),
    max_ms: Number(Math.max(...samples).toFixed(3)),
  };
}

async function main() {
  const args = parseArgs();
  const specs = JSON.parse(fs.readFileSync(args.specs, "utf-8"));
  const runs = Number(args.runs || "5");

  const modulePath = path.resolve("demos/wasm-readonly-demo/dist/pkg/crushr_wasm_readonly_demo.js");
  const wasm = await import(modulePath);
  const wasmBytes = fs.readFileSync(path.resolve("demos/wasm-readonly-demo/dist/pkg/crushr_wasm_readonly_demo_bg.wasm"));
  await wasm.default(wasmBytes);
  wasm.init();

  const archives = [];
  for (const spec of specs) {
    const fileBytes = fs.readFileSync(spec.archive_path);

    const op = async (fn) => {
      const samples = [];
      for (let i = 0; i < runs; i += 1) {
        const t0 = process.hrtime.bigint();
        await fn();
        const elapsedMs = Number(process.hrtime.bigint() - t0) / 1_000_000;
        samples.push(elapsedMs);
      }
      return stats(samples);
    };

    const summaryStageSamples = {
      index_decode_parse: [],
      block_verification_scan: [],
      total: [],
    };

    const archiveSummary = await op(() => wasm.archive_summary(path.basename(spec.archive_path), fileBytes));
    for (let i = 0; i < runs; i += 1) {
      const result = await wasm.archive_summary_stage_breakdown(fileBytes);
      summaryStageSamples.index_decode_parse.push(result.index_decode_parse_ms);
      summaryStageSamples.block_verification_scan.push(result.block_verification_scan_ms);
      summaryStageSamples.total.push(result.index_decode_parse_ms + result.block_verification_scan_ms);
    }

    const operations = {
      archive_summary: archiveSummary,
      archive_summary_stage_breakdown: {
        index_decode_parse: stats(summaryStageSamples.index_decode_parse),
        block_verification_scan: stats(summaryStageSamples.block_verification_scan),
        total: stats(summaryStageSamples.total),
      },
      find: await op(() => wasm.find(fileBytes, spec.query)),
      entry: await op(() => wasm.entry(fileBytes, spec.entry_path)),
      propagation: await op(() => wasm.propagation(fileBytes)),
    };

    archives.push({ ...spec, operations });
  }

  fs.writeFileSync(args.out, JSON.stringify({ runs, mode: "node_wasm", archives }, null, 2) + "\n");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
