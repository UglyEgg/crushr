use crate::progress::{ProgressEvent, ProgressSink, ProgressPhase};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Mutex, Arc};

pub fn make_sink(ui: u8, no_color: bool) -> Arc<dyn ProgressSink> {
    match ui {
        0 => Arc::new(NullSink),
        2 => Arc::new(MockUi2::new(!no_color)),
        3 => Arc::new(MockUi3::new(!no_color)),
        _ => Arc::new(Ui1::new(!no_color)),
    }
}

struct NullSink;
impl ProgressSink for NullSink {
    fn on_event(&self, _ev: ProgressEvent) {}
}

struct Ui1 {
    enabled: bool,
    inner: Mutex<Option<ProgressBar>>,
}
impl Ui1 {
    fn new(enabled: bool) -> Self {
        Self { enabled, inner: Mutex::new(None) }
    }
}
impl ProgressSink for Ui1 {
    fn on_event(&self, ev: ProgressEvent) {
        if !self.enabled { return; }
        match ev {
            ProgressEvent::Start { op, phase: _, total_bytes } => {
                let pb = ProgressBar::new(total_bytes);
                pb.set_style(ProgressStyle::with_template("{bar:40.cyan/blue} {bytes}/{total_bytes} {msg}").unwrap());
                pb.set_message(format!("{:?}", op).to_lowercase());
                *self.inner.lock().unwrap() = Some(pb);
            }
            ProgressEvent::Phase { phase, total_bytes } => {
                if let Some(pb) = self.inner.lock().unwrap().as_ref() {
                    pb.set_message(format!("{:?}", phase).to_lowercase());
                    if let Some(t) = total_bytes { pb.set_length(t); }
                }
            }
            ProgressEvent::AdvanceBytes { bytes } => {
                if let Some(pb) = self.inner.lock().unwrap().as_ref() {
                    pb.inc(bytes);
                }
            }
            ProgressEvent::Message { msg } => {
                if let Some(pb) = self.inner.lock().unwrap().as_ref() {
                    pb.set_message(msg);
                }
            }
            ProgressEvent::Finish { ok } => {
                if let Some(pb) = self.inner.lock().unwrap().take() {
                    if ok { pb.finish_with_message("done"); } else { pb.finish_with_message("failed"); }
                }
            }
        }
    }
}

// Mock UI tier 2: multi-progress layout without real metrics (placeholders only).

struct Ui2State {
    current_phase: ProgressPhase,
}

struct Ui2 {
    enabled: bool,
    #[allow(dead_code)]
    mp: MultiProgress,
    overall: ProgressBar,
    scan: ProgressBar,
    dict: ProgressBar,
    comp: ProgressBar,
    index: ProgressBar,
    tail: ProgressBar,
    started: Mutex<bool>,
}
impl Ui2 {
    fn new(enabled: bool) -> Self {
        let mp = MultiProgress::new();
        let style = ProgressStyle::with_template("{spinner} {msg}").unwrap();
        let overall = mp.add(ProgressBar::new_spinner());
        overall.set_style(style.clone());
        overall.set_message("overall: (tbd)");
        overall.enable_steady_tick(std::time::Duration::from_millis(120));

        let mk = |label: &str, mp: &MultiProgress, style: &ProgressStyle| {
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(style.clone());
            pb.set_message(format!("{label}: (tbd)"));
            pb.enable_steady_tick(std::time::Duration::from_millis(120));
            pb
        };

        let scan = mk("scan", &mp, &style);
        let dict = mk("dict", &mp, &style);
        let comp = mk("compress", &mp, &style);
        let index = mk("index", &mp, &style);
        let tail = mk("tail", &mp, &style);

        Self { enabled, mp, overall, scan, dict, comp, index, tail, started: Mutex::nfn on_event(&self, e: ProgressEvent) {
        use ProgressEvent::*;
        let mut st = self.state.lock().unwrap();
        match e {
            Start { op: _, phase, total_bytes } => {
                st.current_phase = phase;
                self.overall.set_length(total_bytes);
                self.overall.set_position(0);
                self.overall.set_message(format!("{:?}", phase));
                self.phase.set_message(format!("{:?}", phase));
                self.phase.set_length(total_bytes);
                self.phase.set_position(0);
                self.phase.enable_steady_tick(std::time::Duration::from_millis(120));
            }
            Phase { phase, total_bytes } => {
                st.current_phase = phase;
                self.phase.set_message(format!("{:?}", phase));
                if let Some(tb) = total_bytes {
                    self.phase.set_length(tb);
                    self.phase.set_position(0);
                } else {
                    // Unknown total for this phase: show an indeterminate spinner.
                    self.phase.set_length(0);
                    self.phase.set_position(0);
                }
                self.overall.set_message(format!("{:?}", phase));
            }
            AdvanceBytes { bytes } => {
                self.overall.inc(bytes);
                self.phase.inc(bytes);
            }
            Message { msg } => {
                let _ = self.mp.println(msg);
            }
            Finish { ok } => {
                if ok {
                    self.phase.finish_and_clear();
                    self.overall.finish_with_message("done");
                } else {
                    self.phase.abandon_with_message("failed");
                    self.overall.abandon_with_message("failed");
                }
            }
        }
    }
}

// Mock UI tier 3: placeholder "chart" (no real data). Implemented as a single progress line.
struct MockUi3 {
    enabled: bool,
    pb: ProgressBar,
    started: Mutex<bool>,
}
impl MockUi3 {
    fn new(enabled: bool) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
        pb.set_message("chart: [mock] blocks/sec: --  ratio: --  cache: --  dict: --");
        pb.enable_steady_tick(std::time::Duration::from_millis(120));
        Self { enabled, pb, started: Mutex::new(false) }
    }
}
impl ProgressSink for MockUi3 {
    fn on_event(&self, ev: ProgressEvent) {
        if !self.enabled { return; }
        match ev {
            ProgressEvent::Start { op, .. } => {
                *self.started.lock().unwrap() = true;
                self.pb.set_message(format!("chart: [mock] op={:?} blocks/sec: -- ratio: -- cache: -- dict: --", op).to_lowercase());
            }
            ProgressEvent::Phase { phase, .. } => {
                self.pb.set_message(format!("chart: [mock] phase={:?} blocks/sec: -- ratio: -- cache: -- dict: --", phase).to_lowercase());
            }
            ProgressEvent::Finish { ok } => {
                self.pb.finish_with_message(if ok { "done" } else { "failed" });
            }
            _ => {}
        }
    }
}
