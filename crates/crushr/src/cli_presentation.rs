// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski
use std::io::{IsTerminal, Write};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    WarningBanner,
    FailureBanner,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerLevel {
    Info,
    Warning,
    Failure,
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
    is_tty: bool,
    use_color: bool,
    label_width: usize,
    motion_mode: MotionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotionMode {
    Off,
    Reduced,
    Full,
}

#[allow(dead_code)]
impl CliPresenter {
    pub fn new(tool: &'static str, action: &'static str, silent: bool) -> Self {
        #[allow(clippy::disallowed_methods)]
        let is_tty = std::io::stdout().is_terminal();
        let use_color = is_tty && std::env::var_os("NO_COLOR").is_none();
        Self {
            tool,
            action,
            silent,
            is_tty,
            use_color,
            label_width: 22,
            motion_mode: MotionMode::from_env(),
        }
    }

    pub fn header(&self) {
        self.title_block(self.tool, Some(self.action));
    }

    pub fn title_block(&self, tool: &str, context: Option<&str>) {
        if self.silent {
            return;
        }
        let title = if let Some(context) = context {
            format!("{tool}  /  {context}")
        } else {
            tool.to_string()
        };
        println!(
            "{}",
            self.paint_token(VisualToken::TitleProductLine, &title)
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

    pub fn phase(&self, phase: &str, status: StatusWord, detail: Option<&str>) {
        if self.silent {
            return;
        }
        let value = match detail {
            Some(detail) => format!("{} ({detail})", self.paint_status(status)),
            None => self.paint_status(status),
        };
        self.kv(phase, value);
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

    pub fn banner(&self, level: BannerLevel, message: &str) {
        if self.silent {
            return;
        }
        let (token, label) = match level {
            BannerLevel::Info => (VisualToken::InformationalNote, "INFO"),
            BannerLevel::Warning => (VisualToken::WarningBanner, "WARNING"),
            BannerLevel::Failure => (VisualToken::FailureBanner, "FAILURE"),
        };
        println!();
        println!(
            "  {}",
            self.paint_token(token, &format!("{label}: {message}"))
        );
    }

    pub fn result_summary(&self, status: StatusWord, message: &str, rows: &[(&str, String)]) {
        if self.silent {
            return;
        }
        self.section("Result");
        for (key, value) in rows {
            self.kv(key, value);
        }
        self.outcome(status, message);
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

    pub fn begin_active_phase(&self, phase: &str, detail: Option<&str>) -> ActivePhase<'_> {
        if self.silent {
            return ActivePhase::disabled(self, phase);
        }

        if self.motion_mode.should_animate(self.is_tty) {
            ActivePhase::animated(self, phase, detail.map(ToOwned::to_owned))
        } else {
            ActivePhase::static_running(self, phase)
        }
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

pub struct ActivePhase<'a> {
    presenter: &'a CliPresenter,
    phase: String,
    detail: Arc<Mutex<Option<String>>>,
    running: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<()>>,
    rendered_running_row: bool,
}

impl<'a> ActivePhase<'a> {
    fn disabled(presenter: &'a CliPresenter, phase: &str) -> Self {
        Self {
            presenter,
            phase: phase.to_string(),
            detail: Arc::new(Mutex::new(None)),
            running: None,
            handle: None,
            rendered_running_row: false,
        }
    }

    fn static_running(presenter: &'a CliPresenter, phase: &str) -> Self {
        Self {
            presenter,
            phase: phase.to_string(),
            detail: Arc::new(Mutex::new(None)),
            running: None,
            handle: None,
            rendered_running_row: false,
        }
    }

    fn animated(presenter: &'a CliPresenter, phase: &str, detail: Option<String>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_for_thread = Arc::clone(&running);
        let detail_state = Arc::new(Mutex::new(detail));
        let detail_for_thread = Arc::clone(&detail_state);
        let line_label = presenter.paint_token(VisualToken::PrimaryLabel, phase);
        let status = presenter.paint_status(StatusWord::Running);
        let width = presenter.label_width;
        let frames = presenter.motion_mode.frames();
        let cadence = presenter.motion_mode.cadence();
        let handle = thread::spawn(move || {
            let mut frame_idx = 0usize;
            while running_for_thread.load(Ordering::Relaxed) {
                let frame = frames[frame_idx % frames.len()];
                frame_idx = frame_idx.wrapping_add(1);
                let detail_suffix = detail_for_thread
                    .lock()
                    .ok()
                    .and_then(|value| value.clone())
                    .map(|value| format!(" ({value})"))
                    .unwrap_or_default();
                print!(
                    "\r  {:<width$} {} {frame}{detail_suffix}\x1b[K",
                    line_label,
                    status,
                    width = width
                );
                #[allow(clippy::disallowed_methods)]
                let _ = std::io::stdout().flush();
                thread::sleep(cadence);
            }
        });

        Self {
            presenter,
            phase: phase.to_string(),
            detail: detail_state,
            running: Some(running),
            handle: Some(handle),
            rendered_running_row: false,
        }
    }

    pub fn set_detail(&self, detail: impl Into<String>) {
        if let Ok(mut slot) = self.detail.lock() {
            *slot = Some(detail.into());
        }
    }

    pub fn settle(mut self, status: StatusWord, detail: Option<&str>) {
        self.stop_animation();
        self.presenter.phase(&self.phase, status, detail);
        self.rendered_running_row = true;
    }

    fn stop_animation(&mut self) {
        if let Some(running) = self.running.take() {
            running.store(false, Ordering::Relaxed);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
            print!("\r\x1b[2K");
            #[allow(clippy::disallowed_methods)]
            let _ = std::io::stdout().flush();
        }
    }
}

impl Drop for ActivePhase<'_> {
    fn drop(&mut self) {
        self.stop_animation();
        if !self.rendered_running_row && !self.presenter.silent {
            self.presenter.phase(&self.phase, StatusWord::Failed, None);
        }
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
            Self::WarningBanner => Some("\x1b[1;33m"),
            Self::FailureBanner => Some("\x1b[1;31m"),
        }
    }
}

impl MotionMode {
    fn from_env() -> Self {
        if std::env::var_os("CRUSHR_NO_MOTION").is_some() {
            return Self::Off;
        }
        let Some(value) = std::env::var_os("CRUSHR_MOTION") else {
            return Self::Full;
        };
        match value.to_string_lossy().trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "0" => Self::Off,
            "reduced" | "minimal" | "min" => Self::Reduced,
            "full" | "on" | "1" => Self::Full,
            _ => Self::Full,
        }
    }

    fn should_animate(self, is_tty: bool) -> bool {
        is_tty && !matches!(self, Self::Off)
    }

    fn cadence(self) -> Duration {
        match self {
            Self::Off => Duration::from_millis(0),
            Self::Reduced => Duration::from_millis(240),
            Self::Full => Duration::from_millis(120),
        }
    }

    fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Off => &[""],
            Self::Reduced => &[".", " "],
            Self::Full => &[".  ", ".. ", "...", " .."],
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
