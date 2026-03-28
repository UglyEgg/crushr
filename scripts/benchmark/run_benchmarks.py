#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass

from contract import COMPARATORS, DATASET_NAMES, DEFAULT_LEVEL, MANIFEST_VERSION, SCHEMA_VERSION


@dataclass
class CommandMeasurement:
    wall_ms: int
    peak_rss_kb: int | None
    user_ms: int | None
    sys_ms: int | None


def require_tool(name: str) -> None:
    if shutil.which(name) is None:
        raise SystemExit(f"required tool not found in PATH: {name}")


def run_with_measurement(cmd: list[str], cwd: pathlib.Path) -> CommandMeasurement:
    time_bin = shutil.which("time")
    if time_bin is None:
        start = time.perf_counter()
        subprocess.run(cmd, cwd=cwd, check=True)
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        return CommandMeasurement(wall_ms=elapsed_ms, peak_rss_kb=None, user_ms=None, sys_ms=None)

    metrics_file = cwd / ".bench_time_metrics.txt"
    timed_cmd = [
        time_bin,
        "-f",
        "real_sec=%e\nuser_sec=%U\nsys_sec=%S\nmax_rss_kb=%M",
        "-o",
        str(metrics_file),
        *cmd,
    ]
    print("$ " + " ".join(cmd))
    start = time.perf_counter()
    subprocess.run(timed_cmd, cwd=cwd, check=True)
    elapsed_ms = int((time.perf_counter() - start) * 1000)

    metrics: dict[str, str] = {}
    for line in metrics_file.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        metrics[key.strip()] = value.strip()
    metrics_file.unlink(missing_ok=True)

    def parse_seconds_ms(key: str) -> int | None:
        raw = metrics.get(key)
        if raw is None:
            return None
        try:
            return int(float(raw) * 1000)
        except ValueError:
            return None

    peak_rss = None
    try:
        peak_rss = int(metrics["max_rss_kb"])
    except (KeyError, ValueError):
        pass

    return CommandMeasurement(
        wall_ms=parse_seconds_ms("real_sec") or elapsed_ms,
        peak_rss_kb=peak_rss,
        user_ms=parse_seconds_ms("user_sec"),
        sys_ms=parse_seconds_ms("sys_sec"),
    )


def archive_size(path: pathlib.Path) -> int:
    return path.stat().st_size


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run deterministic crushr benchmark suite.")
    parser.add_argument("--datasets", default=".bench/datasets", help="Dataset root directory.")
    parser.add_argument(
        "--output",
        default=".bench/results/benchmark_results.json",
        help="Output JSON path for benchmark records.",
    )
    parser.add_argument(
        "--workdir",
        default=".bench/work",
        help="Scratch directory for archives and extraction outputs.",
    )
    parser.add_argument(
        "--crushr-bin",
        default="target/release/crushr",
        help="Path to crushr binary used for benchmark runs.",
    )
    return parser.parse_args()


def collect_environment() -> dict[str, str]:
    return {
        "platform": platform.platform(),
        "python": platform.python_version(),
        "uname": " ".join(platform.uname()),
        "cwd": str(pathlib.Path.cwd()),
    }


def command_fingerprint() -> str:
    data = {
        "comparators": [
            {"tool": comparator.tool, "profile": comparator.profile, "level": DEFAULT_LEVEL}
            for comparator in COMPARATORS
        ],
        "datasets": DATASET_NAMES,
    }
    encoded = json.dumps(data, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.blake2b(encoded, digest_size=16).hexdigest()


def read_dataset_manifest(datasets_root: pathlib.Path) -> dict[str, object]:
    manifest_path = datasets_root / "dataset_manifest.json"
    if not manifest_path.exists():
        raise SystemExit(f"dataset manifest missing: {manifest_path}")
    raw = json.loads(manifest_path.read_text(encoding="utf-8"))
    if raw.get("manifest_version") != MANIFEST_VERSION:
        raise SystemExit(
            f"unsupported manifest version: {raw.get('manifest_version')}; expected {MANIFEST_VERSION}"
        )
    names = tuple(dataset["name"] for dataset in raw.get("datasets", []))
    if names != DATASET_NAMES:
        raise SystemExit(f"dataset manifest names mismatch: expected {DATASET_NAMES}, got {names}")
    return raw


def main() -> None:
    args = parse_args()
    datasets_root = pathlib.Path(args.datasets).resolve()
    output_path = pathlib.Path(args.output).resolve()
    work_root = pathlib.Path(args.workdir).resolve()
    crushr_bin = pathlib.Path(args.crushr_bin).resolve()
    dataset_manifest = read_dataset_manifest(datasets_root)

    for tool in ("tar", "zstd", "xz"):
        require_tool(tool)
    if not crushr_bin.exists():
        raise SystemExit(f"crushr binary not found: {crushr_bin}")

    for name in DATASET_NAMES:
        if not (datasets_root / name).is_dir():
            raise SystemExit(f"dataset missing: {datasets_root / name}")

    if work_root.exists():
        shutil.rmtree(work_root)
    work_root.mkdir(parents=True, exist_ok=True)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    run_records: list[dict[str, object]] = []
    benchmark_started_at = int(time.time())

    for dataset in DATASET_NAMES:
        input_path = datasets_root / dataset
        input_path_rel = os.path.relpath(input_path, start=datasets_root.parent)
        for comparator in COMPARATORS:
            tool_name = comparator.tool
            profile = comparator.profile
            variant_id = f"{tool_name}_{profile or 'na'}"
            archive_dir = work_root / "archives" / dataset
            extract_dir = work_root / "extracted" / dataset / variant_id
            archive_dir.mkdir(parents=True, exist_ok=True)
            extract_dir.mkdir(parents=True, exist_ok=True)

            if tool_name == "tar_zstd":
                archive_path = archive_dir / "archive.tar.zst"
                pack_cmd = [
                    "tar",
                    "--sort=name",
                    "--mtime=@0",
                    "--owner=0",
                    "--group=0",
                    "--numeric-owner",
                    "--pax-option=delete=atime,delete=ctime",
                    "-I",
                    f"zstd -{DEFAULT_LEVEL}",
                    "-cf",
                    str(archive_path),
                    input_path_rel,
                ]
                extract_cmd = ["tar", "-xf", str(archive_path), "-C", str(extract_dir)]
            elif tool_name == "tar_xz":
                archive_path = archive_dir / "archive.tar.xz"
                pack_cmd = [
                    "tar",
                    "--sort=name",
                    "--mtime=@0",
                    "--owner=0",
                    "--group=0",
                    "--numeric-owner",
                    "--pax-option=delete=atime,delete=ctime",
                    "-I",
                    f"xz -{DEFAULT_LEVEL}",
                    "-cf",
                    str(archive_path),
                    input_path_rel,
                ]
                extract_cmd = ["tar", "-xf", str(archive_path), "-C", str(extract_dir)]
            else:
                archive_path = archive_dir / f"archive_{profile}.crs"
                pack_cmd = [
                    str(crushr_bin),
                    "pack",
                    str(input_path),
                    "-o",
                    str(archive_path),
                    "--level",
                    str(DEFAULT_LEVEL),
                    "--preservation",
                    str(profile),
                    "--silent",
                ]
                extract_cmd = [
                    str(crushr_bin),
                    "extract",
                    str(archive_path),
                    "-o",
                    str(extract_dir),
                    "--all",
                    "--overwrite",
                    "--silent",
                ]

            pack_metrics = run_with_measurement(pack_cmd, cwd=datasets_root.parent)
            extract_metrics = run_with_measurement(extract_cmd, cwd=datasets_root.parent)

            run_records.append(
                {
                    "dataset": dataset,
                    "tool": tool_name,
                    "profile": profile,
                    "pack_command": " ".join(pack_cmd),
                    "extract_command": " ".join(extract_cmd),
                    "archive_path": str(archive_path),
                    "archive_size_bytes": archive_size(archive_path),
                    "pack_time_ms": pack_metrics.wall_ms,
                    "extract_time_ms": extract_metrics.wall_ms,
                    "pack_peak_rss_kb": pack_metrics.peak_rss_kb,
                    "extract_peak_rss_kb": extract_metrics.peak_rss_kb,
                    "pack_user_time_ms": pack_metrics.user_ms,
                    "pack_sys_time_ms": pack_metrics.sys_ms,
                    "extract_user_time_ms": extract_metrics.user_ms,
                    "extract_sys_time_ms": extract_metrics.sys_ms,
                }
            )

    report = {
        "schema_version": SCHEMA_VERSION,
        "benchmark_started_unix": benchmark_started_at,
        "environment": collect_environment(),
        "dataset_manifest": dataset_manifest,
        "assumptions": {
            "level": DEFAULT_LEVEL,
            "command_set_id": command_fingerprint(),
            "comparators": [
                {"tool": comparator.tool, "profile": comparator.profile}
                for comparator in COMPARATORS
            ],
        },
        "runs": run_records,
    }
    output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote benchmark results: {output_path}")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        print(f"command failed with exit code {exc.returncode}", file=sys.stderr)
        sys.exit(exc.returncode)
