// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski
use std::io::IsTerminal;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusWord {
    Verified,
    Ok,
    Complete,
    Partial,
    Refused,
    Failed,
    Running,
    Scanning,
    Writing,
    Finalizing,
}

impl StatusWord {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "VERIFIED",
            Self::Ok => "OK",
            Self::Complete => "COMPLETE",
            Self::Partial => "PARTIAL",
            Self::Refused => "REFUSED",
            Self::Failed => "FAILED",
            Self::Running => "RUNNING",
            Self::Scanning => "SCANNING",
            Self::Writing => "WRITING",
            Self::Finalizing => "FINALIZING",
        }
    }
}

impl StatusWord {
    fn color_code(self) -> Option<&'static str> {
        match self {
            Self::Verified | Self::Complete => Some("\x1b[32m"),
            Self::Partial => Some("\x1b[33m"),
            Self::Refused | Self::Failed => Some("\x1b[31m"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliPresenter {
    tool: &'static str,
    action: &'static str,
    silent: bool,
    use_color: bool,
    label_width: usize,
}

#[allow(dead_code)]
impl CliPresenter {
    pub fn new(tool: &'static str, action: &'static str, silent: bool) -> Self {
        #[allow(clippy::disallowed_methods)]
        let use_color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        Self {
            tool,
            action,
            silent,
            use_color,
            label_width: 22,
        }
    }

    pub fn header(&self) {
        if self.silent {
            return;
        }
        println!("{}  /  {}", self.tool, self.action);
        println!("{}", "-".repeat(72));
    }

    pub fn section(&self, title: &str) {
        if self.silent {
            return;
        }
        println!();
        println!("{title}");
    }

    pub fn kv(&self, key: &str, value: impl std::fmt::Display) {
        if self.silent {
            return;
        }
        println!("  {:<width$} {}", key, value, width = self.label_width);
    }

    pub fn kv_number(&self, key: &str, value: u64) {
        self.kv(key, group_u64(value));
    }

    pub fn stage(&self, stage: &str, status: StatusWord) {
        if self.silent {
            return;
        }
        self.kv(stage, self.paint_status(status));
    }

    pub fn outcome(&self, status: StatusWord, message: &str) {
        if self.silent {
            return;
        }
        self.kv("status", self.paint_status(status));
        self.kv("message", message);
    }

    pub fn item(&self, status: StatusWord, message: &str) {
        if self.silent {
            return;
        }
        println!("  - {} {}", self.paint_status(status), message);
    }

    pub fn silent_summary(&self, status: StatusWord, fields: &[(&str, String)]) {
        if !self.silent {
            return;
        }
        let mut out = format!("{} status={}", self.tool, status.as_str());
        for (k, v) in fields {
            out.push(' ');
            out.push_str(k);
            out.push('=');
            out.push_str(v);
        }
        println!("{out}");
    }

    fn paint_status(&self, status: StatusWord) -> String {
        if self.use_color {
            if let Some(code) = status.color_code() {
                return format!("{code}{}\x1b[0m", status.as_str());
            }
        }
        status.as_str().to_string()
    }
}

pub fn group_u64(value: u64) -> String {
    let mut out = String::new();
    for (idx, ch) in value.to_string().chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}
