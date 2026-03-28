#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys


def add_dictionary_flags(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--dictionary-experiment",
        choices=("off", "on"),
        default="off",
        help="Enable deterministic dictionary experiment comparator.",
    )
    parser.add_argument(
        "--dictionary-scope",
        choices=("per_dataset", "global"),
        default="per_dataset",
        help="Dictionary training cohort scope when experiment is enabled.",
    )
    parser.add_argument("--dictionary-max-samples", type=int, default=256)
    parser.add_argument("--dictionary-sample-bytes", type=int, default=16384)
    parser.add_argument("--dictionary-size-bytes", type=int, default=65536)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Canonical benchmark harness entrypoint.")
    sub = parser.add_subparsers(dest="command", required=True)

    datasets = sub.add_parser("datasets", help="Generate deterministic benchmark datasets.")
    datasets.add_argument("--output", default=".bench/datasets")
    datasets.add_argument("--clean", action="store_true")
    datasets.add_argument("--xattrs", choices=("off", "on"), default="off")

    run = sub.add_parser("run", help="Execute benchmark matrix using generated datasets.")
    run.add_argument("--datasets", default=".bench/datasets")
    run.add_argument("--output", default=".bench/results/benchmark_results.json")
    run.add_argument("--workdir", default=".bench/work")
    run.add_argument("--crushr-bin", default="target/release/crushr")
    add_dictionary_flags(run)

    full = sub.add_parser("full", help="Generate datasets then execute benchmark matrix.")
    full.add_argument("--datasets", default=".bench/datasets")
    full.add_argument("--output", default=".bench/results/benchmark_results.json")
    full.add_argument("--workdir", default=".bench/work")
    full.add_argument("--crushr-bin", default="target/release/crushr")
    full.add_argument("--clean", action="store_true")
    full.add_argument("--xattrs", choices=("off", "on"), default="off")
    add_dictionary_flags(full)

    return parser.parse_args()


def run_script(script_name: str, args: list[str]) -> None:
    script_path = pathlib.Path(__file__).resolve().parent / script_name
    cmd = [sys.executable, str(script_path), *args]
    subprocess.run(cmd, check=True)


def run_benchmark_script_args(args: argparse.Namespace) -> list[str]:
    return [
        "--datasets",
        args.datasets,
        "--output",
        args.output,
        "--workdir",
        args.workdir,
        "--crushr-bin",
        args.crushr_bin,
        "--dictionary-experiment",
        args.dictionary_experiment,
        "--dictionary-scope",
        args.dictionary_scope,
        "--dictionary-max-samples",
        str(args.dictionary_max_samples),
        "--dictionary-sample-bytes",
        str(args.dictionary_sample_bytes),
        "--dictionary-size-bytes",
        str(args.dictionary_size_bytes),
    ]


def main() -> None:
    args = parse_args()
    if args.command == "datasets":
        cmd_args = ["--output", args.output, "--xattrs", args.xattrs]
        if args.clean:
            cmd_args.append("--clean")
        run_script("generate_datasets.py", cmd_args)
        return

    if args.command == "run":
        run_script("run_benchmarks.py", run_benchmark_script_args(args))
        return

    if args.command == "full":
        dataset_args = ["--output", args.datasets, "--xattrs", args.xattrs]
        if args.clean:
            dataset_args.append("--clean")
        run_script("generate_datasets.py", dataset_args)
        run_script("run_benchmarks.py", run_benchmark_script_args(args))


if __name__ == "__main__":
    main()
