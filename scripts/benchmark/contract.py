#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Literal

FIXED_MTIME = 1_700_000_000
SEED = 0xC2A5_2026
DEFAULT_LEVEL = 3
SCHEMA_VERSION = "crushr-benchmark-run.v1"
MANIFEST_VERSION = "crushr-benchmark-dataset-manifest.v1"

DATASET_NAMES: tuple[str, ...] = (
    "small_mixed_tree",
    "medium_realistic_tree",
    "large_stress_tree",
)

OrderingStrategy = Literal[
    "lexical",
    "size_ascending",
    "size_descending",
    "extension_grouped",
    "kind_then_extension",
]


@dataclass(frozen=True)
class Comparator:
    tool: str
    profile: str | None
    ordering_strategy: OrderingStrategy | None = None
    zstd_level: int | None = None
    zstd_strategy: str | None = None


BASELINE_COMPARATORS: tuple[Comparator, ...] = (
    Comparator(
        tool="tar_zstd",
        profile=None,
        ordering_strategy="lexical",
        zstd_level=DEFAULT_LEVEL,
        zstd_strategy="default",
    ),
    Comparator(tool="tar_xz", profile=None, ordering_strategy="lexical"),
    Comparator(tool="crushr", profile="full"),
    Comparator(tool="crushr", profile="basic"),
)


DictionaryScope = Literal["per_dataset", "global"]
ZstdStrategy = Literal["default", "fast", "dfast", "greedy", "lazy", "lazy2", "btlazy2", "btopt"]


@dataclass(frozen=True)
class DictionaryTrainingRule:
    max_samples: int
    sample_bytes: int
    dictionary_size_bytes: int


@dataclass(frozen=True)
class DictionaryExperimentModel:
    enabled: bool
    scope: DictionaryScope
    training_rule: DictionaryTrainingRule


@dataclass(frozen=True)
class ZstdExperimentModel:
    level_matrix: tuple[int, ...]
    strategy_matrix: tuple[ZstdStrategy, ...]


@dataclass(frozen=True)
class OrderingExperimentModel:
    baseline_strategy: OrderingStrategy
    strategy_matrix: tuple[OrderingStrategy, ...]


def dictionary_model(
    *,
    enabled: bool,
    scope: DictionaryScope,
    max_samples: int,
    sample_bytes: int,
    dictionary_size_bytes: int,
) -> DictionaryExperimentModel:
    if max_samples < 1:
        raise ValueError("dictionary max_samples must be >= 1")
    if sample_bytes < 64:
        raise ValueError("dictionary sample_bytes must be >= 64")
    if dictionary_size_bytes < 256:
        raise ValueError("dictionary_size_bytes must be >= 256")
    return DictionaryExperimentModel(
        enabled=enabled,
        scope=scope,
        training_rule=DictionaryTrainingRule(
            max_samples=max_samples,
            sample_bytes=sample_bytes,
            dictionary_size_bytes=dictionary_size_bytes,
        ),
    )


def zstd_experiment_model(*, levels: tuple[int, ...], strategies: tuple[str, ...]) -> ZstdExperimentModel:
    normalized_levels = tuple(sorted(set(levels)))
    if not normalized_levels:
        normalized_levels = (DEFAULT_LEVEL,)
    for level in normalized_levels:
        if level < 1 or level > 22:
            raise ValueError("zstd level values must be in range 1..22")

    normalized_strategies = tuple(dict.fromkeys(strategies))
    if not normalized_strategies:
        normalized_strategies = ("default",)
    valid: set[str] = {"default", "fast", "dfast", "greedy", "lazy", "lazy2", "btlazy2", "btopt"}
    if not set(normalized_strategies).issubset(valid):
        raise ValueError(f"zstd strategies must be one of {sorted(valid)}")
    return ZstdExperimentModel(
        level_matrix=normalized_levels,
        strategy_matrix=normalized_strategies,  # type: ignore[arg-type]
    )


def ordering_experiment_model(*, strategies: tuple[str, ...]) -> OrderingExperimentModel:
    normalized_strategies = tuple(dict.fromkeys(strategies))
    if not normalized_strategies:
        normalized_strategies = ("lexical",)
    valid: set[str] = {
        "lexical",
        "size_ascending",
        "size_descending",
        "extension_grouped",
        "kind_then_extension",
    }
    if not set(normalized_strategies).issubset(valid):
        raise ValueError(f"ordering strategies must be one of {sorted(valid)}")

    ordered_matrix = tuple(strategy for strategy in valid_ordering_strategy_list() if strategy in normalized_strategies)
    return OrderingExperimentModel(
        baseline_strategy="lexical",
        strategy_matrix=ordered_matrix,  # type: ignore[arg-type]
    )


def valid_ordering_strategy_list() -> tuple[OrderingStrategy, ...]:
    return (
        "lexical",
        "size_ascending",
        "size_descending",
        "extension_grouped",
        "kind_then_extension",
    )


def comparator_set(
    dictionary: DictionaryExperimentModel,
    zstd_experiment: ZstdExperimentModel,
    ordering_experiment: OrderingExperimentModel,
) -> tuple[Comparator, ...]:
    comparators: list[Comparator] = [comparator for comparator in BASELINE_COMPARATORS if comparator.tool == "crushr"]

    tar_zstd_variants: list[tuple[int, str]] = [(DEFAULT_LEVEL, "default")]
    for level in zstd_experiment.level_matrix:
        if level == DEFAULT_LEVEL:
            continue
        tar_zstd_variants.append((level, "default"))
    for strategy in zstd_experiment.strategy_matrix:
        if strategy == "default":
            continue
        tar_zstd_variants.append((DEFAULT_LEVEL, strategy))

    seen: set[tuple[str, OrderingStrategy, int | None, str | None]] = set()

    for ordering_strategy in ordering_experiment.strategy_matrix:
        comparators.append(Comparator(tool="tar_xz", profile=None, ordering_strategy=ordering_strategy))

        for zstd_level, zstd_strategy in tar_zstd_variants:
            key = ("tar_zstd", ordering_strategy, zstd_level, zstd_strategy)
            if key in seen:
                continue
            seen.add(key)
            comparators.append(
                Comparator(
                    tool="tar_zstd",
                    profile=None,
                    ordering_strategy=ordering_strategy,
                    zstd_level=zstd_level,
                    zstd_strategy=zstd_strategy,
                )
            )

        if dictionary.enabled:
            comparators.append(
                Comparator(
                    tool="tar_zstd_dict",
                    profile=None,
                    ordering_strategy=ordering_strategy,
                    zstd_level=DEFAULT_LEVEL,
                    zstd_strategy="default",
                )
            )

    return tuple(comparators)


def assumptions_fingerprint(
    dictionary: DictionaryExperimentModel,
    zstd_experiment: ZstdExperimentModel,
    ordering_experiment: OrderingExperimentModel,
) -> str:
    comparators = comparator_set(dictionary, zstd_experiment, ordering_experiment)
    data = {
        "comparators": [
            {
                "tool": comparator.tool,
                "profile": comparator.profile,
                "ordering_strategy": comparator.ordering_strategy,
                "zstd_level": comparator.zstd_level,
                "zstd_strategy": comparator.zstd_strategy,
            }
            for comparator in comparators
        ],
        "datasets": DATASET_NAMES,
        "dictionary_experiment": {
            "enabled": dictionary.enabled,
            "scope": dictionary.scope,
            "training_rule": {
                "max_samples": dictionary.training_rule.max_samples,
                "sample_bytes": dictionary.training_rule.sample_bytes,
                "dictionary_size_bytes": dictionary.training_rule.dictionary_size_bytes,
            },
        },
        "zstd_experiment": {
            "baseline_level": DEFAULT_LEVEL,
            "level_matrix": list(zstd_experiment.level_matrix),
            "strategy_matrix": list(zstd_experiment.strategy_matrix),
        },
        "ordering_experiment": {
            "baseline_strategy": ordering_experiment.baseline_strategy,
            "strategy_matrix": list(ordering_experiment.strategy_matrix),
            "applies_to_tools": ["tar_zstd", "tar_xz"],
        },
    }
    encoded = json.dumps(data, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.blake2b(encoded, digest_size=16).hexdigest()
