use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Copy, Clone)]
pub struct OutputMode {
    pub json: bool,
    pub progress: bool,
}

static OUTPUT_MODE: OnceLock<OutputMode> = OnceLock::new();

pub fn set_output_mode(json: bool, progress: bool) {
    // ignore if already set within process; subsequent calls are no-ops
    let _ = OUTPUT_MODE.set(OutputMode { json, progress });
}

pub fn output_mode() -> OutputMode {
    OUTPUT_MODE.get().copied().unwrap_or(OutputMode {
        json: false,
        progress: true,
    })
}

pub fn print_json<T: serde::Serialize>(value: &T) {
    if let Ok(s) = serde_json::to_string(value) {
        println!("{}", s);
    }
}

pub fn create_progress_bar<F>(total: u64, json_msg_fn: F) -> Option<ProgressBar>
where
    F: FnOnce() -> Value,
{
    let mode = output_mode();
    if mode.progress && !mode.json {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>5}/{len:<5} {percent:>3}% {eta_precise} | {msg}",
                )
                .unwrap()
                .progress_chars("█· "),
        );
        Some(pb)
    } else {
        if mode.json {
            print_json(&json_msg_fn());
        }
        None
    }
}
