// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

pub fn dispatch(args: Vec<String>) -> anyhow::Result<i32> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        println!(
            "usage:\n  crushr lab pack-experimental <input>... -o <archive> [experimental flags]\n  crushr lab <crushr-lab command> [args...]"
        );
        return Ok(0);
    }

    if let Some("pack-experimental") = args.first().map(String::as_str) {
        let code = super::pack::dispatch_lab_experimental(args.into_iter().skip(1).collect());
        return Ok(code);
    }

    crushr_lab::dispatch(args)
}
