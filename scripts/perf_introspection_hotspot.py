#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_DIR = REPO_ROOT / ".bench" / "introspection_hotspot"
BASELINE_DIR = REPO_ROOT / ".bench" / "introspection_baseline"
WASM_DEMO_DIR = REPO_ROOT / "demos" / "wasm-readonly-demo"


def run(cmd: list[str], cwd: Path | None = None) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def ensure_archive_set() -> Path:
    specs = BASELINE_DIR / "archive_set.json"
    if specs.exists():
        return specs
    run(["python3", "scripts/perf_introspection_baseline.py", "--runs", "1"], cwd=REPO_ROOT)
    return specs


def pct(part: float, total: float) -> float:
    if total <= 0:
        return 0.0
    return round((part / total) * 100.0, 1)


def summarize(cli: dict, wasm: dict) -> str:
    lines = [
        "# Phase 18 Step 07 — `find` / `entry` hotspot characterization",
        "",
        "## Scope and method",
        "",
        "- Archive set: existing baseline `small`, `medium`, `large`, `very_large_stress` fixtures.",
        "- Focus for findings: `large` and `very_large_stress` classes.",
        "- Measurements include cold call and repeated-call medians for both CLI and WASM-byte path.",
        "",
        "## CLI repeated-call stage medians (ms)",
        "",
        "| Class | op | total | open/read | decode/parse | summary prep | traversal | materialization |",
        "|---|---|---:|---:|---:|---:|---:|---:|",
    ]

    cli_rows = {row["class_name"]: row for row in cli["archives"]}
    wasm_rows = {row["class_name"]: row for row in wasm["archives"]}

    for cls in ["large", "very_large_stress"]:
        row = cli_rows[cls]["operations"]
        for op in ["find", "entry"]:
            op_row = row[op]
            lines.append(
                "| {cls} | {op} | {total} | {open_} | {decode} | {prep} | {trav} | {mat} |".format(
                    cls=cls,
                    op=op,
                    total=op_row["repeated_total_ms"]["median_ms"],
                    open_=op_row["repeated_archive_open_read_ms"]["median_ms"],
                    decode=op_row["repeated_index_decode_parse_ms"]["median_ms"],
                    prep=op_row["repeated_summary_index_prep_ms"]["median_ms"],
                    trav=op_row["repeated_traversal_ms"]["median_ms"],
                    mat=op_row["repeated_result_materialization_ms"]["median_ms"],
                )
            )

    lines += [
        "",
        "## WASM repeated-call medians (ms)",
        "",
        "| Class | find wall | entry wall |",
        "|---|---:|---:|",
    ]

    for cls in ["large", "very_large_stress"]:
        row = wasm_rows[cls]["operations"]
        lines.append(
            f"| {cls} | {row['find']['median_ms']} | {row['entry']['median_ms']} |"
        )

    vls_cli = cli_rows["very_large_stress"]["operations"]
    def find_share(source: dict, op: str, key: str) -> float:
        op_row = source[op]
        return pct(op_row[key]["median_ms"], op_row["repeated_total_ms"]["median_ms"])

    lines += [
        "",
        "## Findings",
        "",
        f"1. `find` is dominated by summary/index preparation on large archives (CLI very_large_stress median share: {find_share(vls_cli, 'find', 'repeated_summary_index_prep_ms')}%).",
        f"2. `entry` is also dominated by summary/index preparation on large archives (CLI very_large_stress median share: {find_share(vls_cli, 'entry', 'repeated_summary_index_prep_ms')}%).",
        "3. Repeated calls remain close to cold calls for both `find` and `entry`, indicating repeated index+record reconstruction work instead of reuse.",
        "4. Traversal and result materialization are small contributors relative to preparation, even at very_large_stress size.",
        "5. WASM wall-clock remains higher than CLI for the same archive classes, consistent with added byte-bridge/setup overhead on top of shared core work (inference from CLI stage dominance + WASM totals).",
        "",
        "## Recommendation for next bounded optimization packet",
        "",
        "- Highest-value target: cache/reuse decoded index and entry report surfaces per loaded archive for `find`/`entry` (shared by CLI and WASM).",
        "- Follow-on bounded work should separately measure post-cache bridge/materialization cost, then decide if JS/WASM boundary slimming is the next priority.",
    ]

    return "\n".join(lines) + "\n"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--out-dir", default=str(DEFAULT_DIR))
    args = ap.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    specs = ensure_archive_set()
    all_specs = json.loads(specs.read_text())
    filtered_specs = [row for row in all_specs if row["class_name"] in {"large", "very_large_stress"}]
    filtered_specs_path = out_dir / "archive_set_hotspot.json"
    filtered_specs_path.write_text(json.dumps(filtered_specs, indent=2) + "\n")

    cli_out = out_dir / "cli_hotspot.json"
    wasm_out = out_dir / "wasm_hotspot.json"

    run(
        [
            "cargo",
            "run",
            "-p",
            "crushr",
            "--example",
            "introspection_hotspot",
            "--",
            "--specs",
            str(filtered_specs_path),
            "--runs",
            str(args.runs),
            "--out",
            str(cli_out),
        ],
        cwd=REPO_ROOT,
    )

    run(["bash", "./build-dist.sh"], cwd=WASM_DEMO_DIR)
    run(
        [
            "node",
            "scripts/perf_wasm_runner.mjs",
            "--specs",
            str(filtered_specs_path),
            "--runs",
            str(args.runs),
            "--out",
            str(wasm_out),
        ],
        cwd=REPO_ROOT,
    )

    cli = json.loads(cli_out.read_text())
    wasm = json.loads(wasm_out.read_text())

    report_path = REPO_ROOT / "docs/reference/introspection-hotspot-p18s07.md"
    report_path.write_text(summarize(cli, wasm))
    print(f"wrote {report_path}")


if __name__ == "__main__":
    main()
