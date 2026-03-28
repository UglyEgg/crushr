#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import random
import shutil
from dataclasses import asdict, dataclass

from contract import DATASET_NAMES, FIXED_MTIME, MANIFEST_VERSION, SEED


@dataclass
class DatasetSummary:
    name: str
    file_count: int
    directory_count: int
    symlink_count: int
    total_payload_bytes: int
    xattr_files: int


def deterministic_bytes(label: str, size: int) -> bytes:
    blocks: list[bytes] = []
    produced = 0
    cursor = 0
    while produced < size:
        digest = hashlib.blake2b(f"{label}:{cursor}".encode("utf-8"), digest_size=64).digest()
        blocks.append(digest)
        produced += len(digest)
        cursor += 1
    return b"".join(blocks)[:size]


def write_file(path: pathlib.Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(payload)
    os.utime(path, (FIXED_MTIME, FIXED_MTIME), follow_symlinks=False)


def maybe_set_xattr(path: pathlib.Path, key: bytes, value: bytes, enabled: bool) -> bool:
    if not enabled:
        return False
    if not hasattr(os, "setxattr"):
        return False
    try:
        os.setxattr(path, key, value, follow_symlinks=False)
        return True
    except OSError:
        return False


def create_small_mixed_tree(root: pathlib.Path, *, enable_xattrs: bool) -> DatasetSummary:
    dataset_root = root / "small_mixed_tree"
    dataset_root.mkdir(parents=True, exist_ok=True)
    rng = random.Random(SEED + 1)

    file_count = 0
    directory_count = 0
    symlink_count = 0
    total_payload_bytes = 0
    xattr_files = 0

    for bucket in range(18):
        bucket_dir = dataset_root / f"bucket_{bucket:02d}"
        bucket_dir.mkdir(parents=True, exist_ok=True)
        directory_count += 1
        os.utime(bucket_dir, (FIXED_MTIME, FIXED_MTIME), follow_symlinks=False)

        for idx in range(64):
            file_path = bucket_dir / f"entry_{idx:04d}.dat"
            size = 64 + ((bucket * 97 + idx * 37) % 12_288)
            payload = deterministic_bytes(f"small:{bucket}:{idx}", size)
            write_file(file_path, payload)
            file_count += 1
            total_payload_bytes += size

            if rng.random() < 0.08 and maybe_set_xattr(
                file_path,
                b"user.crushr.benchmark",
                f"small-{bucket}-{idx}".encode("utf-8"),
                enable_xattrs,
            ):
                xattr_files += 1

    for idx in range(24):
        empty_dir = dataset_root / "empty_dirs" / f"empty_{idx:02d}"
        empty_dir.mkdir(parents=True, exist_ok=True)
        directory_count += 1
        os.utime(empty_dir, (FIXED_MTIME, FIXED_MTIME), follow_symlinks=False)

    links_root = dataset_root / "links"
    links_root.mkdir(parents=True, exist_ok=True)
    link_target_file = links_root / "entry_0000.dat"
    write_file(link_target_file, deterministic_bytes("small:link-target", 4_096))
    file_count += 1
    total_payload_bytes += 4_096
    for idx in range(16):
        link_path = links_root / f"alias_{idx:02d}.lnk"
        if link_path.exists() or link_path.is_symlink():
            link_path.unlink()
        os.symlink("entry_0000.dat", link_path)
        symlink_count += 1

    return DatasetSummary(
        name="small_mixed_tree",
        file_count=file_count,
        directory_count=directory_count,
        symlink_count=symlink_count,
        total_payload_bytes=total_payload_bytes,
        xattr_files=xattr_files,
    )


def create_medium_realistic_tree(root: pathlib.Path, *, enable_xattrs: bool) -> DatasetSummary:
    dataset_root = root / "medium_realistic_tree"
    dataset_root.mkdir(parents=True, exist_ok=True)
    rng = random.Random(SEED + 2)

    file_count = 0
    directory_count = 0
    symlink_count = 0
    total_payload_bytes = 0
    xattr_files = 0

    for project in range(80):
        project_root = dataset_root / f"project_{project:03d}"
        for section in ("src", "include", "docs", "tests", "assets"):
            section_root = project_root / section
            section_root.mkdir(parents=True, exist_ok=True)
            directory_count += 1
            os.utime(section_root, (FIXED_MTIME, FIXED_MTIME), follow_symlinks=False)

            for idx in range(64):
                if section in {"src", "include", "docs", "tests"}:
                    extension = "txt"
                    size = 200 + ((project * 13 + idx * 29) % 14_000)
                else:
                    extension = "bin"
                    size = 1_024 + ((project * 101 + idx * 53) % 256_000)
                file_path = section_root / f"{section}_{idx:04d}.{extension}"
                payload = deterministic_bytes(f"medium:{project}:{section}:{idx}", size)
                write_file(file_path, payload)
                file_count += 1
                total_payload_bytes += size
                if rng.random() < 0.02 and maybe_set_xattr(
                    file_path,
                    b"user.crushr.benchmark",
                    f"medium-{project}-{section}-{idx}".encode("utf-8"),
                    enable_xattrs,
                ):
                    xattr_files += 1

        canonical_link = project_root / "src" / "src_0000.txt"
        link_path = project_root / "README.link"
        if link_path.exists() or link_path.is_symlink():
            link_path.unlink()
        os.symlink(os.path.relpath(canonical_link, start=project_root), link_path)
        symlink_count += 1

    return DatasetSummary(
        name="medium_realistic_tree",
        file_count=file_count,
        directory_count=directory_count,
        symlink_count=symlink_count,
        total_payload_bytes=total_payload_bytes,
        xattr_files=xattr_files,
    )


def create_large_stress_tree(root: pathlib.Path, *, enable_xattrs: bool) -> DatasetSummary:
    dataset_root = root / "large_stress_tree"
    dataset_root.mkdir(parents=True, exist_ok=True)
    rng = random.Random(SEED + 3)

    file_count = 0
    directory_count = 0
    symlink_count = 0
    total_payload_bytes = 0
    xattr_files = 0

    repeated_blob = deterministic_bytes("large:repeated", 2_097_152)
    for idx in range(180):
        file_path = dataset_root / "repeated_blobs" / f"repeat_{idx:04d}.bin"
        write_file(file_path, repeated_blob)
        file_count += 1
        total_payload_bytes += len(repeated_blob)
        if idx == 0:
            directory_count += 1

    for shard in range(120):
        shard_root = dataset_root / "fanout" / f"shard_{shard:03d}"
        shard_root.mkdir(parents=True, exist_ok=True)
        directory_count += 1
        os.utime(shard_root, (FIXED_MTIME, FIXED_MTIME), follow_symlinks=False)
        for idx in range(520):
            size = 256 + ((shard * 61 + idx * 17) % 4_096)
            file_path = shard_root / f"f_{idx:04d}.txt"
            payload = deterministic_bytes(f"large:fanout:{shard}:{idx}", size)
            write_file(file_path, payload)
            file_count += 1
            total_payload_bytes += size
            if rng.random() < 0.005 and maybe_set_xattr(
                file_path,
                b"user.crushr.benchmark",
                f"large-{shard}-{idx}".encode("utf-8"),
                enable_xattrs,
            ):
                xattr_files += 1

    links_root = dataset_root / "links"
    links_root.mkdir(parents=True, exist_ok=True)
    link_target = links_root / "repeat_anchor.bin"
    write_file(link_target, repeated_blob)
    file_count += 1
    total_payload_bytes += len(repeated_blob)
    directory_count += 1

    for idx in range(24):
        link_dst = links_root / f"repeat_alias_{idx:04d}.lnk"
        if link_dst.exists() or link_dst.is_symlink():
            link_dst.unlink()
        os.symlink("repeat_anchor.bin", link_dst)
        symlink_count += 1

    return DatasetSummary(
        name="large_stress_tree",
        file_count=file_count,
        directory_count=directory_count,
        symlink_count=symlink_count,
        total_payload_bytes=total_payload_bytes,
        xattr_files=xattr_files,
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate deterministic crushr benchmark datasets.")
    parser.add_argument(
        "--output",
        default=".bench/datasets",
        help="Directory where benchmark datasets will be created.",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Remove existing output directory before generation.",
    )
    parser.add_argument(
        "--xattrs",
        choices=("off", "on"),
        default="off",
        help="Whether to set benchmark xattrs. Default is off for host-independent reproducibility.",
    )
    return parser.parse_args()


def dataset_identity_digest(output_root: pathlib.Path) -> str:
    digest = hashlib.blake2b(digest_size=32)
    for path in sorted(output_root.rglob("*")):
        rel = path.relative_to(output_root).as_posix()
        if rel == "dataset_manifest.json":
            continue
        if path.is_symlink():
            digest.update(b"L")
            digest.update(rel.encode("utf-8"))
            digest.update(b"\0")
            digest.update(os.readlink(path).encode("utf-8"))
        elif path.is_file():
            digest.update(b"F")
            digest.update(rel.encode("utf-8"))
            digest.update(b"\0")
            digest.update(path.read_bytes())
        elif path.is_dir():
            digest.update(b"D")
            digest.update(rel.encode("utf-8"))
            digest.update(b"\0")
    return digest.hexdigest()


def main() -> None:
    args = parse_args()
    output_root = pathlib.Path(args.output).resolve()

    if args.clean and output_root.exists():
        shutil.rmtree(output_root)
    output_root.mkdir(parents=True, exist_ok=True)

    enable_xattrs = args.xattrs == "on"

    summaries = [
        create_small_mixed_tree(output_root, enable_xattrs=enable_xattrs),
        create_medium_realistic_tree(output_root, enable_xattrs=enable_xattrs),
        create_large_stress_tree(output_root, enable_xattrs=enable_xattrs),
    ]

    observed_names = tuple(summary.name for summary in summaries)
    if observed_names != DATASET_NAMES:
        raise RuntimeError(f"dataset mismatch: expected {DATASET_NAMES}, observed {observed_names}")

    manifest = {
        "manifest_version": MANIFEST_VERSION,
        "generator": "scripts/benchmark/generate_datasets.py",
        "seed": SEED,
        "fixed_mtime_epoch": FIXED_MTIME,
        "xattrs_mode": args.xattrs,
        "datasets": [asdict(summary) for summary in summaries],
        "dataset_identity": dataset_identity_digest(output_root),
    }
    manifest_path = output_root / "dataset_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote deterministic benchmark datasets to: {output_root}")
    print(f"Manifest: {manifest_path}")


if __name__ == "__main__":
    main()
