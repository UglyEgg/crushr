<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Containerized build guide (crushr)

This repository builds `crushr` inside a container so you get a predictable toolchain and reproducible output.

You do **not** need to know Docker internals to use this. Think of the container as a clean build machine that runs on demand.

## What you get

- A release artifact under `dist/`:
  - `dist/crushr-<version>-<target>.tar.gz`
- Version source: root `VERSION` (strict SemVer) is canonical for build artifact naming.
  - Contains `bin/crushr` and selected documentation files.

- Two build environments:
  - **musl (Alpine)**: produces a mostly-static binary (default).
  - **glibc (Debian)**: used when you request a non-musl target.

## Requirements on your host machine

On your host (your laptop/workstation), install:

- Bash
- Python 3 (only used for small config parsing)
- A container engine:
  - Podman (recommended), or
  - Docker

The build script will fail fast if something is missing.

## Files involved

- `dev/build.sh`  
  The entrypoint script you run. It orchestrates everything.

- `dev/build.toml`  
  Configuration (project name, binary name, default targets, dist layout).

- `dev/Containerfile.build`  
  Alpine build image (musl target). Used for `x86_64-unknown-linux-musl`.

- `dev/Containerfile.build.debian`  
  Debian build image (glibc target). Used for `x86_64-unknown-linux-gnu`.

- `dev/lib/*.sh`  
  Small reusable Bash modules used by `dev/build.sh`.

## Basic usage

From the repository root:

### Build a release artifact (default)
```bash
./dev/build.sh --release
```

### Build a faster debug artifact
```bash
./dev/build.sh --dev
```

### Run tests in the container
```bash
./dev/build.sh --test
```

### Build for glibc (Debian)
```bash
TARGET=x86_64-unknown-linux-gnu ./dev/build.sh --release
```

### Use Docker instead of Podman
```bash
ENGINE=docker ./dev/build.sh --release
```

## What `--clean` does

```bash
./dev/build.sh --clean
```

This removes:
- container images built by this repo (the builder image tag), and
- container volumes used as caches (`target/` and Cargo registry cache).

Use it if:
- you want to reclaim disk space, or
- your cache got into a weird state and you want a “fresh build machine.”

## Where output goes

After a successful run, you will find:

- `dist/crushr-<version>-<target>.tar.gz`

Extract it like this:

```bash
tar -xzf dist/crushr-*-*.tar.gz
./crushr-*/bin/crushr --help
```

## How the container build works (conceptual)

1. `dev/build.sh` decides which image to use based on `TARGET`
   - musl target → Alpine image
   - non-musl target → Debian image

2. It builds the image if needed.

3. It runs `cargo build` inside the container, but mounts two caches:
   - a persistent `target/` directory (speeds up incremental builds)
   - a persistent Cargo registry cache (speeds up dependency downloads)

4. It copies the resulting binary into a clean `dist/` bundle.

## Troubleshooting

### “It says my engine command is missing”
Install Podman or Docker and ensure it’s on your `PATH`.

### “Build works on my machine but fails in the container”
That’s usually a missing package inside the build image (e.g., `pkg-config`, `openssl-dev`, etc.).
Copy/paste the error output and we’ll adjust the Containerfile.

### “I want to change default targets or image tags”
Edit `dev/build.toml`:
- `[container.musl]`
- `[container.glibc]`

Then rerun the build.

## Why this exists

Builds are where projects quietly rot.

Containers prevent “it worked last week on my laptop” from becoming a lifestyle.
