#!/usr/bin/env python3
"""
Generate deterministic cross-format comparison tables for crushr Phase 2.

Inputs:
  PHASE2_RESEARCH/results/normalized_results.json
  PHASE2_RESEARCH/results/normalization_summary.json

Outputs:
  PHASE2_RESEARCH/summaries/comparison_tables.json
  PHASE2_RESEARCH/summaries/format_rankings.json
  PHASE2_RESEARCH/summaries/comparison_summary.md
"""

from __future__ import annotations

import json
import math
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Tuple


FORMAT_ORDER = ["crushr", "zip", "tar+zstd", "tar+gz", "tar+xz"]
RESULT_CLASS_ORDER = ["SUCCESS", "PARTIAL", "REFUSED", "STRUCTURAL_FAIL", "TOOL_ERROR"]
FAILURE_STAGE_ORDER = ["NONE", "PRE_EXTRACT", "EXTRACTION", "UNKNOWN"]
DIAGNOSTIC_ORDER = ["NONE", "GENERIC", "STRUCTURAL", "PRECISE"]
BLAST_RADIUS_ORDER = ["NONE", "LOCALIZED", "PARTIAL_SET", "WIDESPREAD", "TOTAL"]


@dataclass(frozen=True)
class Paths:
    root: Path
    normalized_results: Path
    normalization_summary: Path
    out_dir: Path
    comparison_tables: Path
    format_rankings: Path
    comparison_summary_md: Path


def project_paths(repo_root: Path) -> Paths:
    root = repo_root / "PHASE2_RESEARCH"
    results = root / "results"
    summaries = root / "summaries"
    return Paths(
        root=root,
        normalized_results=results / "normalized_results.json",
        normalization_summary=results / "normalization_summary.json",
        out_dir=summaries,
        comparison_tables=summaries / "comparison_tables.json",
        format_rankings=summaries / "format_rankings.json",
        comparison_summary_md=summaries / "comparison_summary.md",
    )


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def mean(values: Iterable[float]) -> float:
    vals = list(values)
    return sum(vals) / len(vals) if vals else 0.0


def pct(part: int, total: int) -> float:
    return (part / total) if total else 0.0


def ordered_counter(counter: Counter, order: List[str]) -> Dict[str, int]:
    keys = list(order)
    for key in sorted(counter.keys()):
        if key not in keys:
            keys.append(key)
    return {key: int(counter.get(key, 0)) for key in keys}


def ordered_nested_counts(
    nested: Dict[str, Counter], outer_order: List[str], inner_order: List[str]
) -> Dict[str, Dict[str, int]]:
    outer_keys = list(outer_order)
    for key in sorted(nested.keys()):
        if key not in outer_keys:
            outer_keys.append(key)

    return {
        outer: ordered_counter(nested.get(outer, Counter()), inner_order)
        for outer in outer_keys
    }


def safe_round(value: float, digits: int = 6) -> float:
    if math.isnan(value) or math.isinf(value):
        return 0.0
    return round(value, digits)


def build_metrics(records: List[Dict[str, Any]]) -> Dict[str, Any]:
    by_format: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    by_format_result: Dict[str, Counter] = defaultdict(Counter)
    by_format_diag: Dict[str, Counter] = defaultdict(Counter)
    by_format_stage: Dict[str, Counter] = defaultdict(Counter)
    by_format_blast: Dict[str, Counter] = defaultdict(Counter)

    for rec in records:
        fmt = rec["format"]
        by_format[fmt].append(rec)
        by_format_result[fmt][rec["result_class"]] += 1
        by_format_diag[fmt][rec["diagnostic_specificity"]] += 1
        by_format_stage[fmt][rec["failure_stage"]] += 1
        by_format_blast[fmt][rec["blast_radius_class"]] += 1

    per_format_metrics: Dict[str, Dict[str, Any]] = {}

    for fmt in FORMAT_ORDER:
        rows = by_format.get(fmt, [])
        total = len(rows)

        success_count = sum(1 for r in rows if r["result_class"] == "SUCCESS")
        partial_or_better_count = sum(
            1 for r in rows if r["result_class"] in {"SUCCESS", "PARTIAL"}
        )
        refusal_count = sum(1 for r in rows if r["result_class"] == "REFUSED")
        structural_fail_count = sum(
            1 for r in rows if r["result_class"] == "STRUCTURAL_FAIL"
        )
        tool_error_count = sum(1 for r in rows if r["result_class"] == "TOOL_ERROR")

        detected_pre_extract_count = sum(
            1 for r in rows if bool(r.get("detected_pre_extract"))
        )
        precise_diag_count = sum(
            1 for r in rows if r["diagnostic_specificity"] == "PRECISE"
        )
        structural_or_precise_diag_count = sum(
            1 for r in rows if r["diagnostic_specificity"] in {"STRUCTURAL", "PRECISE"}
        )
        localized_or_none_count = sum(
            1 for r in rows if r["blast_radius_class"] in {"NONE", "LOCALIZED"}
        )

        file_ratios = [float(r.get("recovery_ratio_files", 0.0) or 0.0) for r in rows]
        byte_ratios = [float(r.get("recovery_ratio_bytes", 0.0) or 0.0) for r in rows]

        per_format_metrics[fmt] = {
            "scenario_count": total,
            "success_rate": safe_round(pct(success_count, total)),
            "partial_or_better_rate": safe_round(pct(partial_or_better_count, total)),
            "refusal_rate": safe_round(pct(refusal_count, total)),
            "structural_fail_rate": safe_round(pct(structural_fail_count, total)),
            "tool_error_rate": safe_round(pct(tool_error_count, total)),
            "detected_pre_extract_rate": safe_round(
                pct(detected_pre_extract_count, total)
            ),
            "precise_diagnostic_rate": safe_round(pct(precise_diag_count, total)),
            "structural_or_precise_diagnostic_rate": safe_round(
                pct(structural_or_precise_diag_count, total)
            ),
            "localized_or_none_blast_rate": safe_round(
                pct(localized_or_none_count, total)
            ),
            "mean_recovery_ratio_files": safe_round(mean(file_ratios)),
            "mean_recovery_ratio_bytes": safe_round(mean(byte_ratios)),
            "result_class_counts": ordered_counter(
                by_format_result[fmt], RESULT_CLASS_ORDER
            ),
            "failure_stage_counts": ordered_counter(
                by_format_stage[fmt], FAILURE_STAGE_ORDER
            ),
            "diagnostic_specificity_counts": ordered_counter(
                by_format_diag[fmt], DIAGNOSTIC_ORDER
            ),
            "blast_radius_counts": ordered_counter(
                by_format_blast[fmt], BLAST_RADIUS_ORDER
            ),
        }

    return {
        "per_format_metrics": per_format_metrics,
        "per_format_result_class_counts": ordered_nested_counts(
            by_format_result, FORMAT_ORDER, RESULT_CLASS_ORDER
        ),
        "per_format_failure_stage_counts": ordered_nested_counts(
            by_format_stage, FORMAT_ORDER, FAILURE_STAGE_ORDER
        ),
        "per_format_diagnostic_specificity_counts": ordered_nested_counts(
            by_format_diag, FORMAT_ORDER, DIAGNOSTIC_ORDER
        ),
        "per_format_blast_radius_counts": ordered_nested_counts(
            by_format_blast, FORMAT_ORDER, BLAST_RADIUS_ORDER
        ),
    }


def build_rankings(per_format_metrics: Dict[str, Dict[str, Any]]) -> Dict[str, Any]:
    def rank(metric: str, reverse: bool = True) -> List[Dict[str, Any]]:
        items: List[Tuple[str, float]] = []
        for fmt in FORMAT_ORDER:
            value = float(per_format_metrics[fmt][metric])
            items.append((fmt, value))
        items.sort(key=lambda x: (-x[1], x[0]) if reverse else (x[1], x[0]))
        return [
            {"rank": i + 1, "format": fmt, "value": safe_round(val)}
            for i, (fmt, val) in enumerate(items)
        ]

    return {
        "rankings": {
            "survivability_by_file_ratio": rank(
                "mean_recovery_ratio_files", reverse=True
            ),
            "survivability_by_byte_ratio": rank(
                "mean_recovery_ratio_bytes", reverse=True
            ),
            "success_rate": rank("success_rate", reverse=True),
            "partial_or_better_rate": rank("partial_or_better_rate", reverse=True),
            "diagnostic_quality_structural_or_precise": rank(
                "structural_or_precise_diagnostic_rate", reverse=True
            ),
            "pre_extract_detection_rate": rank(
                "detected_pre_extract_rate", reverse=True
            ),
            "blast_radius_localized_or_none": rank(
                "localized_or_none_blast_rate", reverse=True
            ),
            "tool_error_rate_lowest_is_best": rank("tool_error_rate", reverse=False),
        }
    }


def markdown_table(headers: List[str], rows: List[List[str]]) -> str:
    out = []
    out.append("| " + " | ".join(headers) + " |")
    out.append("|" + "|".join(["---"] * len(headers)) + "|")
    for row in rows:
        out.append("| " + " | ".join(row) + " |")
    return "\n".join(out)


def build_summary_md(
    normalization_summary: Dict[str, Any],
    per_format_metrics: Dict[str, Dict[str, Any]],
    rankings: Dict[str, Any],
) -> str:
    total_runs = normalization_summary["total_normalized_runs"]

    overview_rows = []
    for fmt in FORMAT_ORDER:
        m = per_format_metrics[fmt]
        overview_rows.append(
            [
                fmt,
                str(m["scenario_count"]),
                f'{m["mean_recovery_ratio_files"]:.6f}',
                f'{m["mean_recovery_ratio_bytes"]:.6f}',
                f'{m["success_rate"]:.6f}',
                f'{m["partial_or_better_rate"]:.6f}',
                f'{m["tool_error_rate"]:.6f}',
            ]
        )

    blast_rows = []
    for fmt in FORMAT_ORDER:
        counts = per_format_metrics[fmt]["blast_radius_counts"]
        blast_rows.append([fmt] + [str(counts[k]) for k in BLAST_RADIUS_ORDER])

    ranking_rows = []
    for entry in rankings["rankings"]["survivability_by_file_ratio"]:
        ranking_rows.append(
            [str(entry["rank"]), entry["format"], f'{entry["value"]:.6f}']
        )

    parts = [
        "# Phase 2 Cross-Format Comparison Summary",
        "",
        f"Total normalized runs: **{total_runs}**",
        "",
        "## Survivability overview",
        "",
        markdown_table(
            [
                "Format",
                "Runs",
                "Mean file recovery",
                "Mean byte recovery",
                "Success rate",
                "Partial-or-better rate",
                "Tool error rate",
            ],
            overview_rows,
        ),
        "",
        "## Blast radius distribution",
        "",
        markdown_table(
            ["Format"] + BLAST_RADIUS_ORDER,
            blast_rows,
        ),
        "",
        "## Survivability ranking by mean file recovery",
        "",
        markdown_table(
            ["Rank", "Format", "Mean file recovery"],
            ranking_rows,
        ),
        "",
        "## Notes",
        "",
        "- Rates are proportions in the closed interval [0, 1].",
        "- Recovery metrics are derived from extracted file presence and byte counts.",
        "- This summary does not reinterpret or modify normalized inputs.",
    ]
    return "\n".join(parts) + "\n"


def main() -> None:
    repo_root = Path.cwd()
    paths = project_paths(repo_root)

    normalized_results = load_json(paths.normalized_results)
    normalization_summary = load_json(paths.normalization_summary)

    if not isinstance(normalized_results, list):
        raise SystemExit("normalized_results.json must contain a top-level list")
    if len(normalized_results) != int(normalization_summary["total_normalized_runs"]):
        raise SystemExit(
            "normalized_results.json record count does not match normalization_summary.json"
        )

    metrics = build_metrics(normalized_results)
    per_format_metrics = metrics["per_format_metrics"]
    rankings = build_rankings(per_format_metrics)

    comparison_tables = {
        "source_artifacts": {
            "normalized_results": str(paths.normalized_results.relative_to(repo_root)),
            "normalization_summary": str(
                paths.normalization_summary.relative_to(repo_root)
            ),
        },
        "total_runs": len(normalized_results),
        **metrics,
    }

    summary_md = build_summary_md(normalization_summary, per_format_metrics, rankings)

    paths.out_dir.mkdir(parents=True, exist_ok=True)
    paths.comparison_tables.write_text(
        json.dumps(comparison_tables, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    paths.format_rankings.write_text(
        json.dumps(rankings, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    paths.comparison_summary_md.write_text(summary_md, encoding="utf-8")

    print(f"Wrote {paths.comparison_tables}")
    print(f"Wrote {paths.format_rankings}")
    print(f"Wrote {paths.comparison_summary_md}")


if __name__ == "__main__":
    main()
