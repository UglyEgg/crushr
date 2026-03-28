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
from dataclasses import asdict, dataclass

from contract import (
    DATASET_NAMES,
    DEFAULT_LEVEL,
    MANIFEST_VERSION,
    SCHEMA_VERSION,
    DictionaryExperimentModel,
    DictionaryTrainingRule,
    ZstdExperimentModel,
    assumptions_fingerprint,
    comparator_set,
    dictionary_model,
    zstd_experiment_model,
)


@dataclass
class CommandMeasurement:
    wall_ms: int
    peak_rss_kb: int | None
    user_ms: int | None
    sys_ms: int | None


@dataclass(frozen=True)
class TrainingSample:
    relpath: str
    size_bytes: int
    content_hash: str


@dataclass(frozen=True)
class DictionaryArtifact:
    enabled: bool
    scope: str | None
    cohort_label: str | None
    dictionary_path: str | None
    dictionary_content_hash: str | None
    dictionary_id: str | None
    dependency_kind: str | None
    training_provenance: dict[str, object]


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


def zstd_supports_strategy_flag() -> bool:
    probe = subprocess.run(
        ["zstd", "--strategy=fast", "-q", "-c"],
        input=b"strategy-probe",
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return probe.returncode == 0


def validate_zstd_strategy_capability(
    *,
    dictionary_experiment: DictionaryExperimentModel,
    zstd_experiment: ZstdExperimentModel,
) -> None:
    requires_strategy_flag = any(
        comparator.tool.startswith("tar_zstd") and (comparator.zstd_strategy or "default") != "default"
        for comparator in comparator_set(dictionary_experiment, zstd_experiment)
    )
    if not requires_strategy_flag:
        return
    if zstd_supports_strategy_flag():
        return

    requested = sorted(
        {
            comparator.zstd_strategy or "default"
            for comparator in comparator_set(dictionary_experiment, zstd_experiment)
            if comparator.tool.startswith("tar_zstd") and (comparator.zstd_strategy or "default") != "default"
        }
    )
    raise SystemExit(
        "host zstd CLI does not support --strategy=<name>; "
        f"cannot run requested non-default strategy experiment(s): {', '.join(requested)}. "
        "Use --zstd-strategies default or run on a zstd build with strategy flag support."
    )


def build_zstd_cli_args(
    *,
    level: int,
    strategy: str,
    dictionary_path: str | None = None,
) -> str:
    args = [f"zstd -{level}"]
    if strategy != "default":
        args.append(f"--strategy={strategy}")
    if dictionary_path is not None:
        args.append(f"-D {dictionary_path}")
    return " ".join(args)


def build_tar_zstd_commands(
    *,
    archive_path: pathlib.Path,
    input_path_rel: str,
    extract_dir: pathlib.Path,
    zstd_level: int,
    zstd_strategy: str,
    dictionary_path: str | None = None,
) -> tuple[list[str], list[str]]:
    pack_cmd = [
        "tar",
        "--sort=name",
        "--mtime=@0",
        "--owner=0",
        "--group=0",
        "--numeric-owner",
        "--pax-option=delete=atime,delete=ctime",
        "-I",
        build_zstd_cli_args(level=zstd_level, strategy=zstd_strategy, dictionary_path=dictionary_path),
        "-cf",
        str(archive_path),
        input_path_rel,
    ]
    if dictionary_path is None:
        extract_cmd = ["tar", "-xf", str(archive_path), "-C", str(extract_dir)]
    else:
        extract_cmd = [
            "tar",
            "-I",
            f"zstd -d -D {dictionary_path}",
            "-xf",
            str(archive_path),
            "-C",
            str(extract_dir),
        ]
    return pack_cmd, extract_cmd


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
    parser.add_argument(
        "--dictionary-max-samples",
        type=int,
        default=256,
        help="Maximum number of training files considered per dictionary cohort.",
    )
    parser.add_argument(
        "--dictionary-sample-bytes",
        type=int,
        default=16384,
        help="Bytes read per training sample file (from file start).",
    )
    parser.add_argument(
        "--dictionary-size-bytes",
        type=int,
        default=65536,
        help="Target dictionary size bytes passed to zstd --train.",
    )
    parser.add_argument(
        "--zstd-levels",
        default=str(DEFAULT_LEVEL),
        help="Comma-separated zstd levels for tar+zstd experiment matrix (e.g. 1,3,6).",
    )
    parser.add_argument(
        "--zstd-strategies",
        default="default",
        help="Comma-separated zstd strategies for tar+zstd experiment matrix.",
    )
    return parser.parse_args()


def parse_csv_levels(raw: str) -> tuple[int, ...]:
    values = tuple(part.strip() for part in raw.split(",") if part.strip())
    if not values:
        raise SystemExit("zstd level experiment matrix must not be empty")
    try:
        return tuple(int(value) for value in values)
    except ValueError as exc:
        raise SystemExit(f"invalid zstd level list: {raw}") from exc


def parse_csv_strings(raw: str) -> tuple[str, ...]:
    values = tuple(part.strip() for part in raw.split(",") if part.strip())
    if not values:
        raise SystemExit("zstd strategy experiment matrix must not be empty")
    return values


def collect_environment() -> dict[str, str]:
    return {
        "platform": platform.platform(),
        "python": platform.python_version(),
        "uname": " ".join(platform.uname()),
        "cwd": str(pathlib.Path.cwd()),
    }


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


def read_training_samples(
    datasets_root: pathlib.Path,
    datasets: tuple[str, ...],
    training_rule: DictionaryTrainingRule,
) -> tuple[list[TrainingSample], list[pathlib.Path]]:
    all_files: list[pathlib.Path] = []
    for dataset in datasets:
        dataset_root = datasets_root / dataset
        for path in sorted(dataset_root.rglob("*")):
            if path.is_file() and not path.is_symlink():
                all_files.append(path)

    selected_paths = all_files[: training_rule.max_samples]
    samples: list[TrainingSample] = []
    for path in selected_paths:
        relpath = path.relative_to(datasets_root).as_posix()
        data = path.read_bytes()[: training_rule.sample_bytes]
        samples.append(
            TrainingSample(
                relpath=relpath,
                size_bytes=len(data),
                content_hash=hashlib.blake2b(data, digest_size=16).hexdigest(),
            )
        )
    return samples, selected_paths


def compute_training_manifest_id(
    *,
    cohort_label: str,
    scope: str,
    training_rule: DictionaryTrainingRule,
    samples: list[TrainingSample],
) -> str:
    data = {
        "cohort_label": cohort_label,
        "scope": scope,
        "training_rule": asdict(training_rule),
        "samples": [asdict(sample) for sample in samples],
    }
    encoded = json.dumps(data, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.blake2b(encoded, digest_size=16).hexdigest()


def train_dictionary_artifact(
    *,
    datasets_root: pathlib.Path,
    model: DictionaryExperimentModel,
    cohort_datasets: tuple[str, ...],
    cohort_label: str,
    dictionary_dir: pathlib.Path,
) -> DictionaryArtifact:
    samples, training_paths = read_training_samples(datasets_root, cohort_datasets, model.training_rule)
    if not training_paths:
        raise SystemExit(f"dictionary experiment enabled but no training samples found for cohort {cohort_label}")

    training_manifest_id = compute_training_manifest_id(
        cohort_label=cohort_label,
        scope=model.scope,
        training_rule=model.training_rule,
        samples=samples,
    )
    dictionary_dir.mkdir(parents=True, exist_ok=True)
    dict_filename = f"dict_{cohort_label}_{training_manifest_id}.zdict"
    dictionary_path = dictionary_dir / dict_filename

    train_cmd = [
        "zstd",
        "--train",
        f"--maxdict={model.training_rule.dictionary_size_bytes}",
        "-o",
        str(dictionary_path),
        *[str(path) for path in training_paths],
    ]
    subprocess.run(train_cmd, cwd=datasets_root.parent, check=True)

    dictionary_bytes = dictionary_path.read_bytes()
    dictionary_content_hash = hashlib.blake2b(dictionary_bytes, digest_size=32).hexdigest()
    dictionary_id_payload = {
        "cohort_label": cohort_label,
        "scope": model.scope,
        "training_manifest_id": training_manifest_id,
        "dictionary_content_hash": dictionary_content_hash,
    }
    dictionary_id = hashlib.blake2b(
        json.dumps(dictionary_id_payload, sort_keys=True, separators=(",", ":")).encode("utf-8"),
        digest_size=16,
    ).hexdigest()

    return DictionaryArtifact(
        enabled=True,
        scope=model.scope,
        cohort_label=cohort_label,
        dictionary_path=str(dictionary_path),
        dictionary_content_hash=dictionary_content_hash,
        dictionary_id=dictionary_id,
        dependency_kind="required_dictionary",
        training_provenance={
            "cohort_datasets": list(cohort_datasets),
            "training_manifest_id": training_manifest_id,
            "training_sample_count": len(samples),
            "training_sample_bytes": sum(sample.size_bytes for sample in samples),
            "selection_rule": {
                "sort_order": "lexicographic_relative_path",
                "max_samples": model.training_rule.max_samples,
                "sample_bytes": model.training_rule.sample_bytes,
            },
            "training_samples": [asdict(sample) for sample in samples],
        },
    )


def dictionary_disabled_artifact() -> DictionaryArtifact:
    return DictionaryArtifact(
        enabled=False,
        scope=None,
        cohort_label=None,
        dictionary_path=None,
        dictionary_content_hash=None,
        dictionary_id=None,
        dependency_kind=None,
        training_provenance={
            "cohort_datasets": [],
            "training_manifest_id": None,
            "training_sample_count": 0,
            "training_sample_bytes": 0,
            "selection_rule": None,
            "training_samples": [],
        },
    )


def build_dictionary_artifacts(
    *,
    datasets_root: pathlib.Path,
    model: DictionaryExperimentModel,
    work_root: pathlib.Path,
) -> dict[str, DictionaryArtifact]:
    if not model.enabled:
        return {}

    dictionary_dir = work_root / "dictionary_experiments"
    if model.scope == "global":
        artifact = train_dictionary_artifact(
            datasets_root=datasets_root,
            model=model,
            cohort_datasets=DATASET_NAMES,
            cohort_label="all_datasets",
            dictionary_dir=dictionary_dir,
        )
        return {dataset: artifact for dataset in DATASET_NAMES}

    artifacts: dict[str, DictionaryArtifact] = {}
    for dataset in DATASET_NAMES:
        artifacts[dataset] = train_dictionary_artifact(
            datasets_root=datasets_root,
            model=model,
            cohort_datasets=(dataset,),
            cohort_label=dataset,
            dictionary_dir=dictionary_dir,
        )
    return artifacts


def zstd_comparator_label(
    *,
    tool_name: str,
    profile: str | None,
    zstd_level: int | None,
    zstd_strategy: str | None,
) -> str:
    if tool_name.startswith("tar_zstd"):
        return f"{tool_name}_l{zstd_level or DEFAULT_LEVEL}_s{zstd_strategy or 'default'}"
    return f"{tool_name}_{profile or 'na'}"


def main() -> None:
    args = parse_args()
    datasets_root = pathlib.Path(args.datasets).resolve()
    output_path = pathlib.Path(args.output).resolve()
    work_root = pathlib.Path(args.workdir).resolve()
    crushr_bin = pathlib.Path(args.crushr_bin).resolve()
    dataset_manifest = read_dataset_manifest(datasets_root)
    dictionary_experiment = dictionary_model(
        enabled=args.dictionary_experiment == "on",
        scope=args.dictionary_scope,
        max_samples=args.dictionary_max_samples,
        sample_bytes=args.dictionary_sample_bytes,
        dictionary_size_bytes=args.dictionary_size_bytes,
    )
    zstd_experiment = zstd_experiment_model(
        levels=parse_csv_levels(args.zstd_levels),
        strategies=parse_csv_strings(args.zstd_strategies),
    )

    for tool in ("tar", "zstd", "xz"):
        require_tool(tool)
    validate_zstd_strategy_capability(
        dictionary_experiment=dictionary_experiment,
        zstd_experiment=zstd_experiment,
    )
    if not crushr_bin.exists():
        raise SystemExit(f"crushr binary not found: {crushr_bin}")

    for name in DATASET_NAMES:
        if not (datasets_root / name).is_dir():
            raise SystemExit(f"dataset missing: {datasets_root / name}")

    if work_root.exists():
        shutil.rmtree(work_root)
    work_root.mkdir(parents=True, exist_ok=True)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    dictionary_artifacts = build_dictionary_artifacts(
        datasets_root=datasets_root,
        model=dictionary_experiment,
        work_root=work_root,
    )

    run_records: list[dict[str, object]] = []
    benchmark_started_at = int(time.time())

    for dataset in DATASET_NAMES:
        input_path = datasets_root / dataset
        input_path_rel = os.path.relpath(input_path, start=datasets_root.parent)
        for comparator in comparator_set(dictionary_experiment, zstd_experiment):
            tool_name = comparator.tool
            profile = comparator.profile
            zstd_level = comparator.zstd_level
            zstd_strategy = comparator.zstd_strategy
            variant_id = zstd_comparator_label(
                tool_name=tool_name,
                profile=profile,
                zstd_level=zstd_level,
                zstd_strategy=zstd_strategy,
            )
            archive_dir = work_root / "archives" / dataset
            extract_dir = work_root / "extracted" / dataset / variant_id
            archive_dir.mkdir(parents=True, exist_ok=True)
            extract_dir.mkdir(parents=True, exist_ok=True)

            dictionary_artifact = dictionary_disabled_artifact()
            if tool_name == "tar_zstd":
                zstd_level = zstd_level or DEFAULT_LEVEL
                zstd_strategy = zstd_strategy or "default"
                archive_path = archive_dir / "archive.tar.zst"
                pack_cmd, extract_cmd = build_tar_zstd_commands(
                    archive_path=archive_path,
                    input_path_rel=input_path_rel,
                    extract_dir=extract_dir,
                    zstd_level=zstd_level,
                    zstd_strategy=zstd_strategy,
                )
            elif tool_name == "tar_zstd_dict":
                dictionary_artifact = dictionary_artifacts[dataset]
                zstd_level = zstd_level or DEFAULT_LEVEL
                zstd_strategy = zstd_strategy or "default"
                archive_path = archive_dir / "archive.tar.zst"
                pack_cmd, extract_cmd = build_tar_zstd_commands(
                    archive_path=archive_path,
                    input_path_rel=input_path_rel,
                    extract_dir=extract_dir,
                    zstd_level=zstd_level,
                    zstd_strategy=zstd_strategy,
                    dictionary_path=dictionary_artifact.dictionary_path,
                )
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
                    "comparator_label": variant_id,
                    "zstd_level": zstd_level,
                    "zstd_strategy": zstd_strategy,
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
                    "dictionary": asdict(dictionary_artifact),
                }
            )

    report = {
        "schema_version": SCHEMA_VERSION,
        "benchmark_started_unix": benchmark_started_at,
        "environment": collect_environment(),
        "dataset_manifest": dataset_manifest,
        "assumptions": {
            "level": DEFAULT_LEVEL,
            "command_set_id": assumptions_fingerprint(dictionary_experiment, zstd_experiment),
            "comparators": [
                {
                    "tool": comparator.tool,
                    "profile": comparator.profile,
                    "zstd_level": comparator.zstd_level,
                    "zstd_strategy": comparator.zstd_strategy,
                }
                for comparator in comparator_set(dictionary_experiment, zstd_experiment)
            ],
            "dictionary_experiment": {
                "enabled": dictionary_experiment.enabled,
                "scope": dictionary_experiment.scope,
                "training_rule": asdict(dictionary_experiment.training_rule),
            },
            "zstd_experiment": {
                "baseline_level": DEFAULT_LEVEL,
                "level_matrix": list(zstd_experiment.level_matrix),
                "strategy_matrix": list(zstd_experiment.strategy_matrix),
            },
        },
        "dictionary_artifacts": [asdict(artifact) for artifact in sorted(dictionary_artifacts.values(), key=lambda a: a.cohort_label or "")],
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
