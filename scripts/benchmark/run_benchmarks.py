#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

import argparse
import collections
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
    Comparator,
    ContentClassExperimentModel,
    ContentClassName,
    DictionaryExperimentModel,
    OrderingExperimentModel,
    OrderingStrategy,
    DictionaryTrainingRule,
    ZstdExperimentModel,
    assumptions_fingerprint,
    comparator_set,
    content_class_experiment_model,
    dictionary_model,
    ordering_experiment_model,
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


@dataclass(frozen=True)
class OrderedInputEntry:
    relpath: str
    kind: str
    size_bytes: int
    extension: str


STRUCTURED_TEXT_EXTENSIONS: frozenset[str] = frozenset(
    {"json", "jsonl", "yaml", "yml", "toml", "xml", "ini", "cfg", "csv", "tsv"}
)
TEXT_LIKE_EXTENSIONS: frozenset[str] = frozenset(
    {
        "txt",
        "md",
        "rst",
        "html",
        "htm",
        "css",
        "js",
        "ts",
        "py",
        "rs",
        "c",
        "cc",
        "cpp",
        "h",
        "hpp",
        "java",
        "go",
        "sh",
        "bash",
        "zsh",
        "sql",
    }
)
CONTENT_CLASS_ORDER: tuple[ContentClassName, ...] = (
    "structured_text_like",
    "text_like",
    "binary_like",
    "unknown_mixed",
)
CONTENT_CLASS_SAMPLE_BYTES = 4096
CONTENT_CLASS_BINARY_NULL_THRESHOLD = 1
CONTENT_CLASS_BINARY_NON_TEXT_RATIO_THRESHOLD = 0.30


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
    ordering_experiment: OrderingExperimentModel,
    content_class_experiment: ContentClassExperimentModel,
) -> None:
    comparators = comparator_set(dictionary_experiment, zstd_experiment, ordering_experiment, content_class_experiment)
    requires_strategy_flag = any(
        comparator.tool.startswith("tar_zstd") and (comparator.zstd_strategy or "default") != "default"
        for comparator in comparators
    )
    if not requires_strategy_flag:
        return
    if zstd_supports_strategy_flag():
        return

    requested = sorted(
        {
            comparator.zstd_strategy or "default"
            for comparator in comparators
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
    ordered_inputs_path: pathlib.Path,
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
        "--no-recursion",
        "--verbatim-files-from",
        "-T",
        str(ordered_inputs_path),
        "-I",
        build_zstd_cli_args(level=zstd_level, strategy=zstd_strategy, dictionary_path=dictionary_path),
        "-cf",
        str(archive_path),
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
        help=(
            "Comma-separated zstd levels for tar+zstd experiment matrix "
            "(e.g. 1,3,6 or 1-10)."
        ),
    )
    parser.add_argument(
        "--zstd-strategies",
        default="default",
        help="Comma-separated zstd strategies for tar+zstd experiment matrix.",
    )
    parser.add_argument(
        "--ordering-strategies",
        default="lexical",
        help="Comma-separated deterministic ordering strategies for tar comparators.",
    )
    parser.add_argument(
        "--content-class-strategy",
        choices=("off", "lightweight_v1"),
        default="off",
        help="Deterministic content-class clustering strategy for tar comparators.",
    )
    return parser.parse_args()


def parse_csv_levels(raw: str) -> tuple[int, ...]:
    tokens = tuple(part.strip() for part in raw.split(",") if part.strip())
    if not tokens:
        raise SystemExit("zstd level experiment matrix must not be empty")

    values: list[int] = []
    for token in tokens:
        if "-" in token:
            parts = token.split("-", 1)
            if len(parts) != 2:
                raise SystemExit(f"invalid zstd level list: {raw}")
            try:
                start = int(parts[0])
                end = int(parts[1])
            except ValueError as exc:
                raise SystemExit(f"invalid zstd level list: {raw}") from exc
            if start > end:
                raise SystemExit(
                    f"invalid zstd level range '{token}': range start must be <= range end"
                )
            values.extend(range(start, end + 1))
            continue

        try:
            values.append(int(token))
        except ValueError as exc:
            raise SystemExit(f"invalid zstd level list: {raw}") from exc

    return tuple(values)


def parse_csv_strings(raw: str) -> tuple[str, ...]:
    values = tuple(part.strip() for part in raw.split(",") if part.strip())
    if not values:
        raise SystemExit("zstd strategy experiment matrix must not be empty")
    return values


def parse_ordering_entries(dataset_root: pathlib.Path) -> list[OrderedInputEntry]:
    entries: list[OrderedInputEntry] = [
        OrderedInputEntry(
            relpath=dataset_root.relative_to(dataset_root.parent).as_posix(),
            kind="directory",
            size_bytes=0,
            extension="",
        )
    ]
    for path in sorted(dataset_root.rglob("*")):
        relpath = path.relative_to(dataset_root.parent).as_posix()
        if path.is_symlink():
            entries.append(OrderedInputEntry(relpath=relpath, kind="symlink", size_bytes=0, extension=""))
            continue
        if path.is_dir():
            entries.append(OrderedInputEntry(relpath=relpath, kind="directory", size_bytes=0, extension=""))
            continue
        if path.is_file():
            entries.append(
                OrderedInputEntry(
                    relpath=relpath,
                    kind="file",
                    size_bytes=path.stat().st_size,
                    extension=path.suffix.lower().lstrip("."),
                )
            )
    return entries


def infer_content_class(entry: OrderedInputEntry, file_path: pathlib.Path) -> ContentClassName:
    if entry.kind != "file":
        return "unknown_mixed"
    if entry.extension in STRUCTURED_TEXT_EXTENSIONS:
        return "structured_text_like"
    if entry.extension in TEXT_LIKE_EXTENSIONS:
        return "text_like"

    sample = file_path.read_bytes()[:CONTENT_CLASS_SAMPLE_BYTES]
    if not sample:
        return "unknown_mixed"

    null_count = sample.count(0)
    if null_count >= CONTENT_CLASS_BINARY_NULL_THRESHOLD:
        return "binary_like"

    printable = sum(
        1
        for byte in sample
        if byte in {9, 10, 13} or 32 <= byte <= 126
    )
    non_text_ratio = 1.0 - (printable / len(sample))
    if non_text_ratio > CONTENT_CLASS_BINARY_NON_TEXT_RATIO_THRESHOLD:
        return "binary_like"
    return "text_like"


def ordering_sort_key(entry: OrderedInputEntry, strategy: OrderingStrategy) -> tuple[object, ...]:
    kind_rank = {"directory": 0, "symlink": 1, "file": 2}.get(entry.kind, 99)
    if strategy == "lexical":
        return (entry.relpath,)
    if strategy == "size_ascending":
        return (kind_rank, entry.size_bytes, entry.extension, entry.relpath)
    if strategy == "size_descending":
        return (kind_rank, -entry.size_bytes, entry.extension, entry.relpath)
    if strategy == "extension_grouped":
        return (kind_rank, entry.extension, entry.relpath)
    if strategy == "kind_then_extension":
        return (kind_rank, entry.extension, entry.size_bytes, entry.relpath)
    raise ValueError(f"unsupported ordering strategy: {strategy}")


def ordered_inputs_file(
    *,
    input_path: pathlib.Path,
    list_base_dir: pathlib.Path,
    strategy: OrderingStrategy,
    content_class_experiment: ContentClassExperimentModel,
    order_root: pathlib.Path,
) -> tuple[pathlib.Path, dict[str, int]]:
    order_root.mkdir(parents=True, exist_ok=True)
    list_path = order_root / f"{input_path.name}.{strategy}.files.txt"
    entries = parse_ordering_entries(input_path)
    ordered = sorted(entries, key=lambda entry: ordering_sort_key(entry, strategy))
    class_counts: collections.Counter[str] = collections.Counter()
    if content_class_experiment.strategy != "off":
        class_by_relpath: dict[str, ContentClassName] = {}
        for entry in ordered:
            resolved = input_path.parent / entry.relpath
            content_class = infer_content_class(entry, resolved)
            class_by_relpath[entry.relpath] = content_class
            class_counts[content_class] += 1

        ordered = sorted(
            ordered,
            key=lambda entry: (
                CONTENT_CLASS_ORDER.index(class_by_relpath[entry.relpath]),
                *ordering_sort_key(entry, strategy),
            ),
        )
    list_lines: list[str] = []
    for entry in ordered:
        resolved_entry = (input_path.parent / entry.relpath).resolve()
        relative_entry = resolved_entry.relative_to(list_base_dir).as_posix()
        list_lines.append(f"{relative_entry}\n")
    list_path.write_text("".join(list_lines), encoding="utf-8")
    return list_path, {name: class_counts.get(name, 0) for name in CONTENT_CLASS_ORDER}


def validate_ordered_inputs_file(
    *,
    list_path: pathlib.Path,
    list_base_dir: pathlib.Path,
    dataset_name: str,
) -> list[pathlib.Path]:
    raw_lines = list_path.read_text(encoding="utf-8").splitlines()
    if not raw_lines:
        raise SystemExit(f"ordered input list is empty for dataset {dataset_name}: {list_path}")

    resolved_paths: list[pathlib.Path] = []
    for line_no, raw_entry in enumerate(raw_lines, start=1):
        entry = raw_entry.strip()
        if not entry:
            raise SystemExit(
                f"ordered input list contains blank/whitespace-only entry at line {line_no}: {list_path}"
            )
        if entry.startswith("-"):
            raise SystemExit(
                f"ordered input list contains unsafe dash-prefixed entry at line {line_no}: {entry}"
            )
        path = pathlib.Path(entry)
        if path.is_absolute():
            raise SystemExit(
                f"ordered input list entry must be relative to datasets root (line {line_no}): {entry} ({list_path})"
            )
        resolved = (list_base_dir / path).resolve()
        if not os.path.lexists(resolved):
            raise SystemExit(
                f"ordered input list entry does not resolve on filesystem (line {line_no}): {entry}"
            )
        resolved_paths.append(resolved)

    if not any(path.name == dataset_name for path in resolved_paths):
        raise SystemExit(
            f"ordered input list for dataset {dataset_name} is malformed: "
            f"missing dataset root entry in {list_path}"
        )
    return resolved_paths


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


def validate_ordering_matrix_comparators(
    *,
    comparators: tuple[Comparator, ...],
    ordering_experiment: OrderingExperimentModel,
) -> None:
    if len(ordering_experiment.strategy_matrix) <= 1:
        return

    tar_ordering = {
        comparator.ordering_strategy
        for comparator in comparators
        if getattr(comparator, "tool", "") in {"tar_zstd", "tar_zstd_dict", "tar_xz"}
    }
    if len(tar_ordering) <= 1:
        raise SystemExit(
            "ordering strategy matrix requested multiple strategies, but comparator expansion collapsed to baseline only"
        )


def zstd_comparator_label(
    *,
    tool_name: str,
    profile: str | None,
    ordering_strategy: OrderingStrategy | None,
    content_class_strategy: str,
    zstd_level: int | None,
    zstd_strategy: str | None,
) -> str:
    if tool_name.startswith("tar_zstd"):
        return (
            f"{tool_name}_ord{ordering_strategy or 'na'}_l{zstd_level or DEFAULT_LEVEL}_"
            f"s{zstd_strategy or 'default'}_cc{content_class_strategy}"
        )
    if tool_name == "tar_xz":
        return f"{tool_name}_ord{ordering_strategy or 'na'}_l{DEFAULT_LEVEL}_cc{content_class_strategy}"
    return (
        f"{tool_name}_{profile or 'na'}_ord{ordering_strategy or 'runtime_default'}_"
        f"ccruntime_default"
    )


def print_zstd_level_sweep_summary(
    *,
    run_records: list[dict[str, object]],
    dataset_manifest: dict[str, object],
    zstd_experiment: ZstdExperimentModel,
    ordering_experiment: OrderingExperimentModel,
    content_class_experiment: ContentClassExperimentModel,
) -> None:
    if len(zstd_experiment.level_matrix) <= 1:
        return
    if tuple(zstd_experiment.strategy_matrix) != ("default",):
        return
    if tuple(ordering_experiment.strategy_matrix) != ("lexical",):
        return
    if content_class_experiment.strategy != "off":
        return

    datasets = {
        dataset["name"]: int(dataset["total_bytes"])
        for dataset in dataset_manifest.get("datasets", [])
        if isinstance(dataset, dict)
        and isinstance(dataset.get("name"), str)
        and isinstance(dataset.get("total_bytes"), int)
    }
    if not datasets:
        return

    by_dataset_level: dict[tuple[str, int], list[dict[str, object]]] = {}
    for run in run_records:
        if run.get("tool") != "tar_zstd":
            continue
        if run.get("zstd_strategy") != "default":
            continue
        level = run.get("zstd_level")
        dataset = run.get("dataset")
        if not isinstance(level, int) or not isinstance(dataset, str):
            continue
        by_dataset_level.setdefault((dataset, level), []).append(run)

    if not by_dataset_level:
        return

    print("Zstd level sweep summary (tar+zstd, default strategy, lexical ordering)")
    for dataset in sorted(datasets):
        dataset_bytes = datasets[dataset]
        print(f"  Dataset: {dataset} ({dataset_bytes} input bytes)")
        print("    level | archive_bytes | ratio | pack_ms | extract_ms")
        for level in zstd_experiment.level_matrix:
            runs = by_dataset_level.get((dataset, level), [])
            if not runs:
                continue
            archive_bytes = int(sum(int(run["archive_size_bytes"]) for run in runs) / len(runs))
            pack_ms = int(sum(int(run["pack_time_ms"]) for run in runs) / len(runs))
            extract_ms = int(sum(int(run["extract_time_ms"]) for run in runs) / len(runs))
            ratio = archive_bytes / dataset_bytes if dataset_bytes > 0 else 0.0
            print(
                f"{level:>9} | {archive_bytes:>13} | {ratio:>5.3f} |"
                f" {pack_ms:>7} | {extract_ms:>10}"
            )


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
    ordering_experiment = ordering_experiment_model(
        strategies=parse_csv_strings(args.ordering_strategies),
    )
    content_class_experiment = content_class_experiment_model(strategy=args.content_class_strategy)

    for tool in ("tar", "zstd", "xz"):
        require_tool(tool)
    validate_zstd_strategy_capability(
        dictionary_experiment=dictionary_experiment,
        zstd_experiment=zstd_experiment,
        ordering_experiment=ordering_experiment,
        content_class_experiment=content_class_experiment,
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

    comparators = comparator_set(dictionary_experiment, zstd_experiment, ordering_experiment, content_class_experiment)
    validate_ordering_matrix_comparators(comparators=comparators, ordering_experiment=ordering_experiment)

    run_records: list[dict[str, object]] = []
    benchmark_started_at = int(time.time())

    for dataset in DATASET_NAMES:
        input_path = datasets_root / dataset
        for comparator in comparators:
            tool_name = comparator.tool
            profile = comparator.profile
            ordering_strategy = comparator.ordering_strategy
            zstd_level = comparator.zstd_level
            zstd_strategy = comparator.zstd_strategy
            content_class_strategy = comparator.content_class_strategy
            variant_id = zstd_comparator_label(
                tool_name=tool_name,
                profile=profile,
                ordering_strategy=ordering_strategy,
                content_class_strategy=content_class_strategy,
                zstd_level=zstd_level,
                zstd_strategy=zstd_strategy,
            )
            archive_dir = work_root / "archives" / dataset
            extract_dir = work_root / "extracted" / dataset / variant_id
            archive_dir.mkdir(parents=True, exist_ok=True)
            extract_dir.mkdir(parents=True, exist_ok=True)

            dictionary_artifact = dictionary_disabled_artifact()
            if tool_name == "tar_zstd":
                if ordering_strategy is None:
                    raise SystemExit("internal error: tar_zstd comparator missing ordering strategy")
                zstd_level = zstd_level or DEFAULT_LEVEL
                zstd_strategy = zstd_strategy or "default"
                archive_path = archive_dir / f"archive_{variant_id}.tar.zst"
                ordered_inputs_path, content_class_counts = ordered_inputs_file(
                    input_path=input_path,
                    list_base_dir=datasets_root.parent,
                    strategy=ordering_strategy,
                    content_class_experiment=content_class_experiment,
                    order_root=work_root / "ordering_inputs" / dataset,
                )
                validate_ordered_inputs_file(
                    list_path=ordered_inputs_path,
                    list_base_dir=datasets_root.parent,
                    dataset_name=dataset,
                )
                pack_cmd, extract_cmd = build_tar_zstd_commands(
                    archive_path=archive_path,
                    ordered_inputs_path=ordered_inputs_path,
                    extract_dir=extract_dir,
                    zstd_level=zstd_level,
                    zstd_strategy=zstd_strategy,
                )
            elif tool_name == "tar_zstd_dict":
                if ordering_strategy is None:
                    raise SystemExit("internal error: tar_zstd_dict comparator missing ordering strategy")
                dictionary_artifact = dictionary_artifacts[dataset]
                zstd_level = zstd_level or DEFAULT_LEVEL
                zstd_strategy = zstd_strategy or "default"
                archive_path = archive_dir / f"archive_{variant_id}.tar.zst"
                ordered_inputs_path, content_class_counts = ordered_inputs_file(
                    input_path=input_path,
                    list_base_dir=datasets_root.parent,
                    strategy=ordering_strategy,
                    content_class_experiment=content_class_experiment,
                    order_root=work_root / "ordering_inputs" / dataset,
                )
                validate_ordered_inputs_file(
                    list_path=ordered_inputs_path,
                    list_base_dir=datasets_root.parent,
                    dataset_name=dataset,
                )
                pack_cmd, extract_cmd = build_tar_zstd_commands(
                    archive_path=archive_path,
                    ordered_inputs_path=ordered_inputs_path,
                    extract_dir=extract_dir,
                    zstd_level=zstd_level,
                    zstd_strategy=zstd_strategy,
                    dictionary_path=dictionary_artifact.dictionary_path,
                )
            elif tool_name == "tar_xz":
                if ordering_strategy is None:
                    raise SystemExit("internal error: tar_xz comparator missing ordering strategy")
                archive_path = archive_dir / f"archive_{variant_id}.tar.xz"
                ordered_inputs_path, content_class_counts = ordered_inputs_file(
                    input_path=input_path,
                    list_base_dir=datasets_root.parent,
                    strategy=ordering_strategy,
                    content_class_experiment=content_class_experiment,
                    order_root=work_root / "ordering_inputs" / dataset,
                )
                validate_ordered_inputs_file(
                    list_path=ordered_inputs_path,
                    list_base_dir=datasets_root.parent,
                    dataset_name=dataset,
                )
                pack_cmd = [
                    "tar",
                    "--sort=name",
                    "--mtime=@0",
                    "--owner=0",
                    "--group=0",
                    "--numeric-owner",
                    "--pax-option=delete=atime,delete=ctime",
                    "--no-recursion",
                    "--verbatim-files-from",
                    "-T",
                    str(ordered_inputs_path),
                    "-I",
                    f"xz -{DEFAULT_LEVEL}",
                    "-cf",
                    str(archive_path),
                ]
                extract_cmd = ["tar", "-xf", str(archive_path), "-C", str(extract_dir)]
            else:
                content_class_counts = {name: 0 for name in CONTENT_CLASS_ORDER}
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
                    "ordering_strategy": ordering_strategy,
                    "content_class_strategy": content_class_strategy,
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
                    "content_classification": {
                        "classifier_version": "lightweight_v1",
                        "class_order": list(CONTENT_CLASS_ORDER),
                        "class_counts": content_class_counts,
                    },
                }
            )

    if len(ordering_experiment.strategy_matrix) > 1:
        observed_tar_ordering = {
            run["ordering_strategy"]
            for run in run_records
            if run["tool"] in {"tar_zstd", "tar_zstd_dict", "tar_xz"}
        }
        if len(observed_tar_ordering) <= 1:
            raise SystemExit(
                "ordering strategy matrix requested multiple strategies, but benchmark output only recorded baseline ordering"
            )

    report = {
        "schema_version": SCHEMA_VERSION,
        "benchmark_started_unix": benchmark_started_at,
        "environment": collect_environment(),
        "dataset_manifest": dataset_manifest,
        "assumptions": {
            "level": DEFAULT_LEVEL,
            "command_set_id": assumptions_fingerprint(
                dictionary_experiment,
                zstd_experiment,
                ordering_experiment,
                content_class_experiment,
            ),
            "comparators": [
                {
                    "tool": comparator.tool,
                    "profile": comparator.profile,
                    "ordering_strategy": comparator.ordering_strategy,
                    "content_class_strategy": comparator.content_class_strategy,
                    "zstd_level": comparator.zstd_level,
                    "zstd_strategy": comparator.zstd_strategy,
                }
                for comparator in comparators
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
            "ordering_experiment": {
                "baseline_strategy": ordering_experiment.baseline_strategy,
                "strategy_matrix": list(ordering_experiment.strategy_matrix),
                "applies_to_tools": ["tar_zstd", "tar_xz", "tar_zstd_dict"],
            },
            "content_class_experiment": {
                "strategy": content_class_experiment.strategy,
                "applies_to_tools": list(content_class_experiment.applies_to_tools),
                "class_labels": list(CONTENT_CLASS_ORDER),
                "classifier_version": "lightweight_v1",
                "sample_bytes": CONTENT_CLASS_SAMPLE_BYTES,
                "null_byte_binary_threshold": CONTENT_CLASS_BINARY_NULL_THRESHOLD,
                "non_text_ratio_binary_threshold": CONTENT_CLASS_BINARY_NON_TEXT_RATIO_THRESHOLD,
            },
        },
        "dictionary_artifacts": [asdict(artifact) for artifact in sorted(dictionary_artifacts.values(), key=lambda a: a.cohort_label or "")],
        "runs": run_records,
    }
    output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote benchmark results: {output_path}")
    print_zstd_level_sweep_summary(
        run_records=run_records,
        dataset_manifest=dataset_manifest,
        zstd_experiment=zstd_experiment,
        ordering_experiment=ordering_experiment,
        content_class_experiment=content_class_experiment,
    )


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        print(f"command failed with exit code {exc.returncode}", file=sys.stderr)
        sys.exit(exc.returncode)
