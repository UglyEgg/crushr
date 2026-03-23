// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualToken {
    TitleProductLine,
    SectionHeader,
    PrimaryLabel,
    SecondaryText,
    ActiveRunning,
    Pending,
    CompleteSuccess,
    WarningDegraded,
    FailureRefusal,
    InformationalNote,
    TrustCanonical,
    TrustRecoveredNamed,
    TrustRecoveredAnonymous,
    TrustUnrecoverable,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustClass {
    Canonical,
    RecoveredNamed,
    RecoveredAnonymous,
    Unrecoverable,
}

impl TrustClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "CANONICAL",
            Self::RecoveredNamed => "RECOVERED_NAMED",
            Self::RecoveredAnonymous => "RECOVERED_ANONYMOUS",
            Self::Unrecoverable => "UNRECOVERABLE",
        }
    }

    fn token(self) -> VisualToken {
        match self {
            Self::Canonical => VisualToken::TrustCanonical,
            Self::RecoveredNamed => VisualToken::TrustRecoveredNamed,
            Self::RecoveredAnonymous => VisualToken::TrustRecoveredAnonymous,
            Self::Unrecoverable => VisualToken::TrustUnrecoverable,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusWord {
    Pending,
    Running,
    Complete,
    Degraded,
    Failed,
    Refused,
    Verified,
    Scanning,
    Writing,
    Finalizing,
    Ok,
    Partial,
}

impl StatusWord {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Running => "RUNNING",
            Self::Complete => "COMPLETE",
            Self::Degraded | Self::Partial => "DEGRADED",
            Self::Failed => "FAILED",
            Self::Refused => "REFUSED",
            Self::Verified => "VERIFIED",
            Self::Ok => "OK",
            Self::Scanning => "RUNNING",
            Self::Writing => "RUNNING",
            Self::Finalizing => "RUNNING",
        }
    }

    fn token(self) -> VisualToken {
        match self {
            Self::Pending => VisualToken::Pending,
            Self::Running | Self::Scanning | Self::Writing | Self::Finalizing => {
                VisualToken::ActiveRunning
            }
            Self::Complete | Self::Verified | Self::Ok => VisualToken::CompleteSuccess,
            Self::Degraded | Self::Partial => VisualToken::WarningDegraded,
            Self::Failed | Self::Refused => VisualToken::FailureRefusal,
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
        println!(
            "{}",
            self.paint_token(
                VisualToken::TitleProductLine,
                &format!("{}  /  {}", self.tool, self.action)
            )
        );
        println!("{}", "-".repeat(72));
    }

    pub fn section(&self, title: &str) {
        if self.silent {
            return;
        }
        println!();
        println!("{}", self.paint_token(VisualToken::SectionHeader, title));
    }

    pub fn kv(&self, key: &str, value: impl std::fmt::Display) {
        if self.silent {
            return;
        }
        println!(
            "  {:<width$} {}",
            self.paint_token(VisualToken::PrimaryLabel, key),
            value,
            width = self.label_width
        );
    }

    pub fn kv_muted(&self, key: &str, value: impl std::fmt::Display) {
        self.kv(
            key,
            self.paint_token(VisualToken::SecondaryText, &value.to_string()),
        );
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

    pub fn trust_kv(&self, key: &str, trust: TrustClass) {
        self.kv(key, self.paint_token(trust.token(), trust.as_str()));
    }

    pub fn item(&self, status: StatusWord, message: &str) {
        if self.silent {
            return;
        }
        println!("  - {} {}", self.paint_status(status), message);
    }

    pub fn info_note(&self, message: &str) {
        if self.silent {
            return;
        }
        println!(
            "  - {}",
            self.paint_token(VisualToken::InformationalNote, message)
        );
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
        self.paint_token(status.token(), status.as_str())
    }

    fn paint_token(&self, token: VisualToken, value: &str) -> String {
        if self.use_color
            && let Some(code) = token.color_code()
        {
            return format!("{code}{value}\x1b[0m");
        }
        value.to_string()
    }
}

impl VisualToken {
    fn color_code(self) -> Option<&'static str> {
        match self {
            Self::TitleProductLine => Some("\x1b[1;37m"),
            Self::SectionHeader => Some("\x1b[1;36m"),
            Self::PrimaryLabel => Some("\x1b[37m"),
            Self::SecondaryText => Some("\x1b[90m"),
            Self::ActiveRunning => Some("\x1b[36m"),
            Self::Pending => Some("\x1b[90m"),
            Self::CompleteSuccess | Self::TrustCanonical => Some("\x1b[32m"),
            Self::WarningDegraded | Self::TrustRecoveredNamed => Some("\x1b[33m"),
            Self::TrustRecoveredAnonymous => Some("\x1b[93m"),
            Self::FailureRefusal | Self::TrustUnrecoverable => Some("\x1b[31m"),
            Self::InformationalNote => Some("\x1b[94m"),
        }
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
