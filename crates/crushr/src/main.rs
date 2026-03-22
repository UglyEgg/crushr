// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

mod cli_app;

fn main() {
    std::process::exit(cli_app::run_env());
}
