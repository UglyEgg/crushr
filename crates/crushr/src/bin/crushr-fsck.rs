// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

fn main() {
    eprintln!(
        "crushr-fsck is retired. Use `crushr-extract --verify <archive>` for strict verification, or `crushr-salvage` for recovery-oriented analysis."
    );
    std::process::exit(2);
}
