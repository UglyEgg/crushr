// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

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

#[derive(Debug, Clone)]
pub struct CliPresenter {
    tool: &'static str,
    action: &'static str,
    silent: bool,
}

#[allow(dead_code)]
impl CliPresenter {
    pub fn new(tool: &'static str, action: &'static str, silent: bool) -> Self {
        Self {
            tool,
            action,
            silent,
        }
    }

    pub fn header(&self) {
        if self.silent {
            return;
        }
        println!("== {} | {} ==", self.tool, self.action);
    }

    pub fn section(&self, title: &str) {
        if self.silent {
            return;
        }
        println!("-- {title} --");
    }

    pub fn kv(&self, key: &str, value: impl std::fmt::Display) {
        if self.silent {
            return;
        }
        println!("{key}: {value}");
    }

    pub fn stage(&self, stage: &str, status: StatusWord) {
        if self.silent {
            return;
        }
        println!("[{}] stage={stage}", status.as_str());
    }

    pub fn outcome(&self, status: StatusWord, message: &str) {
        if self.silent {
            return;
        }
        println!("[{}] {message}", status.as_str());
    }

    pub fn item(&self, status: StatusWord, message: &str) {
        if self.silent {
            return;
        }
        println!("* [{}] {message}", status.as_str());
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
}
