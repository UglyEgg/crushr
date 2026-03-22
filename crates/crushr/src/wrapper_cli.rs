// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::about::{BuildMetadata, render_about};

pub fn run_wrapper_env(
    wrapper_name: &str,
    canonical_usage: &str,
    dispatch: fn(Vec<String>) -> i32,
) -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "{wrapper_name} — wrapper over canonical crushr CLI\ncanonical equivalent: {canonical_usage}\n"
        );
        return dispatch(vec!["--help".to_string()]);
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", crate::product_version());
        return 0;
    }
    if matches!(args.first().map(String::as_str), Some("about")) {
        print!("{}", render_about(&BuildMetadata::from_env()));
        return 0;
    }
    dispatch(args)
}
