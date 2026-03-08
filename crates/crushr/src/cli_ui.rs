use crate::progress::{ProgressEvent, ProgressSink};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};

pub fn make_sink(ui: u8, no_color: bool) -> Arc<dyn ProgressSink> {
    match ui {
        0 => Arc::new(NullSink),
        2 => Arc::new(Ui2::new(!no_color)),
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
        Self {
            enabled,
            inner: Mutex::new(None),
        }
    }
}
impl ProgressSink for Ui1 {
    fn on_event(&self, ev: ProgressEvent) {
        if !self.enabled {
            return;
        }
        match ev {
            ProgressEvent::Start {
                op,
                phase: _,
                total_bytes,
            } => {
                let pb = ProgressBar::new(total_bytes);
                pb.set_style(
                    ProgressStyle::with_template("{bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")
                        .unwrap(),
                );
                pb.set_message(format!("{:?}", op).to_lowercase());
                *self.inner.lock().unwrap() = Some(pb);
            }
            ProgressEvent::Phase { phase, total_bytes } => {
                if let Some(pb) = self.inner.lock().unwrap().as_ref() {
                    pb.set_message(format!("{:?}", phase).to_lowercase());
                    if let Some(t) = total_bytes {
                        pb.set_length(t);
                    }
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
                    if ok {
                        pb.finish_with_message("done");
                    } else {
                        pb.finish_with_message("failed");
                    }
                }
            }
        }
    }
}

// Mock UI tier 2: multi-progress layout without real metrics (placeholders only).

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

        Self {
            enabled,
            mp,
            overall,
            scan,
            dict,
            comp,
            index,
            tail,
            started: Mutex::new(false),
        }
    }
}

impl ProgressSink for Ui2 {
    fn on_event(&self, ev: ProgressEvent) {
        if !self.enabled {
            return;
        }
        match ev {
            ProgressEvent::Start { op, .. } => {
                *self.started.lock().unwrap() = true;
                self.overall
                    .set_message(format!("overall: {:?}", op).to_lowercase());
            }
            ProgressEvent::Message { msg } => {
                let _ = self.mp.println(msg);
            }
            ProgressEvent::Finish { ok } => {
                if ok {
                    self.scan.finish_with_message("scan: done");
                    self.dict.finish_with_message("dict: done");
                    self.comp.finish_with_message("compress: done");
                    self.index.finish_with_message("index: done");
                    self.tail.finish_with_message("tail: done");
                    self.overall.finish_with_message("done");
                } else {
                    self.scan.abandon_with_message("scan: failed");
                    self.dict.abandon_with_message("dict: failed");
                    self.comp.abandon_with_message("compress: failed");
                    self.index.abandon_with_message("index: failed");
                    self.tail.abandon_with_message("tail: failed");
                    self.overall.abandon_with_message("failed");
                }
            }
            ProgressEvent::Phase { .. } | ProgressEvent::AdvanceBytes { .. } => {}
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
        Self {
            enabled,
            pb,
            started: Mutex::new(false),
        }
    }
}
impl ProgressSink for MockUi3 {
    fn on_event(&self, ev: ProgressEvent) {
        if !self.enabled {
            return;
        }
        match ev {
            ProgressEvent::Start { op, .. } => {
                *self.started.lock().unwrap() = true;
                self.pb.set_message(
                    format!(
                        "chart: [mock] op={:?} blocks/sec: -- ratio: -- cache: -- dict: --",
                        op
                    )
                    .to_lowercase(),
                );
            }
            ProgressEvent::Phase { phase, .. } => {
                self.pb.set_message(
                    format!(
                        "chart: [mock] phase={:?} blocks/sec: -- ratio: -- cache: -- dict: --",
                        phase
                    )
                    .to_lowercase(),
                );
            }
            ProgressEvent::Finish { ok } => {
                self.pb
                    .finish_with_message(if ok { "done" } else { "failed" });
            }
            _ => {}
        }
    }
}
