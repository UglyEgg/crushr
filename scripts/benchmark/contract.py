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


@dataclass(frozen=True)
class Comparator:
    tool: str
    profile: str | None


BASELINE_COMPARATORS: tuple[Comparator, ...] = (
    Comparator(tool="tar_zstd", profile=None),
    Comparator(tool="tar_xz", profile=None),
    Comparator(tool="crushr", profile="full"),
    Comparator(tool="crushr", profile="basic"),
)


DictionaryScope = Literal["per_dataset", "global"]


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


def comparator_set(model: DictionaryExperimentModel) -> tuple[Comparator, ...]:
    if not model.enabled:
        return BASELINE_COMPARATORS
    return (*BASELINE_COMPARATORS, Comparator(tool="tar_zstd_dict", profile=None))


def assumptions_fingerprint(model: DictionaryExperimentModel) -> str:
    comparators = comparator_set(model)
    data = {
        "comparators": [
            {"tool": comparator.tool, "profile": comparator.profile, "level": DEFAULT_LEVEL}
            for comparator in comparators
        ],
        "datasets": DATASET_NAMES,
        "dictionary_experiment": {
            "enabled": model.enabled,
            "scope": model.scope,
            "training_rule": {
                "max_samples": model.training_rule.max_samples,
                "sample_bytes": model.training_rule.sample_bytes,
                "dictionary_size_bytes": model.training_rule.dictionary_size_bytes,
            },
        },
    }
    encoded = json.dumps(data, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.blake2b(encoded, digest_size=16).hexdigest()
