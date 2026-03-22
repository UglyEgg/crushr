// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::sync::OnceLock;

const VERSION_RAW: &str = include_str!("../../../VERSION");

pub fn product_version() -> &'static str {
    static VERSION: OnceLock<&'static str> = OnceLock::new();
    VERSION.get_or_init(|| {
        let version = VERSION_RAW.trim_end_matches(['\r', '\n']);
        assert!(
            version == version.trim(),
            "VERSION must contain only a strict SemVer string"
        );
        assert!(
            validate_semver_strict(version),
            "VERSION is not strict SemVer: {version}"
        );
        version
    })
}

pub fn validate_semver_strict(value: &str) -> bool {
    if value.is_empty() || value.starts_with('v') {
        return false;
    }

    let (without_build, build) = split_once(value, '+');
    let (core, pre) = split_once(without_build, '-');

    if !validate_core(core) {
        return false;
    }

    if let Some(pre) = pre
        && !validate_identifiers(pre, true)
    {
        return false;
    }

    if let Some(build) = build
        && !validate_identifiers(build, false)
    {
        return false;
    }

    true
}

fn split_once(input: &str, ch: char) -> (&str, Option<&str>) {
    if let Some((left, right)) = input.split_once(ch) {
        (left, Some(right))
    } else {
        (input, None)
    }
}

fn validate_core(core: &str) -> bool {
    let mut parts = core.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    valid_numeric_identifier(major)
        && valid_numeric_identifier(minor)
        && valid_numeric_identifier(patch)
}

fn validate_identifiers(value: &str, enforce_numeric_no_leading_zero: bool) -> bool {
    if value.is_empty() {
        return false;
    }
    for ident in value.split('.') {
        if ident.is_empty() || !ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
        if enforce_numeric_no_leading_zero
            && ident.chars().all(|c| c.is_ascii_digit())
            && !valid_numeric_identifier(ident)
        {
            return false;
        }
    }
    true
}

fn valid_numeric_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|c| c.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

#[cfg(test)]
mod tests {
    use super::validate_semver_strict;

    #[test]
    fn accepts_strict_semver_examples() {
        assert!(validate_semver_strict("0.2.2"));
        assert!(validate_semver_strict("0.3.0-rc.1"));
        assert!(validate_semver_strict("1.2.3-alpha.10+build.77"));
    }

    #[test]
    fn rejects_non_strict_semver_values() {
        assert!(!validate_semver_strict("v0.2.2"));
        assert!(!validate_semver_strict("01.2.3"));
        assert!(!validate_semver_strict("1.2"));
        assert!(!validate_semver_strict("1.2.3-01"));
        assert!(!validate_semver_strict("1.2.3+meta?"));
    }
}
