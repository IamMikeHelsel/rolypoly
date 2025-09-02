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
    OUTPUT_MODE
        .get()
        .copied()
        .unwrap_or(OutputMode { json: false, progress: true })
}

pub fn print_json<T: serde::Serialize>(value: &T) {
    if let Ok(s) = serde_json::to_string(value) {
        println!("{}", s);
    }
}

