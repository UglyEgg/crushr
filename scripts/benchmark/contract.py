#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

from dataclasses import dataclass

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


COMPARATORS: tuple[Comparator, ...] = (
    Comparator(tool="tar_zstd", profile=None),
    Comparator(tool="tar_xz", profile=None),
    Comparator(tool="crushr", profile="full"),
    Comparator(tool="crushr", profile="basic"),
)
