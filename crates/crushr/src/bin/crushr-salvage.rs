// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

fn main() {
    std::process::exit(crushr::wrapper_cli::run_wrapper_env(
        "crushr-salvage",
        "crushr salvage <archive>",
        crushr::commands::salvage::dispatch,
    ));
}
