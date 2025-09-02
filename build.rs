use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Compute build metadata
    let run_number = env::var("BUILD_NUMBER")
        .ok()
        .or_else(|| env::var("GITHUB_RUN_NUMBER").ok())
        .unwrap_or_else(|| {
            // Fallback to git commit count
            Command::new("git")
                .args(["rev-list", "--count", "HEAD"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "0".to_string())
        });

    let git_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let build_time = chrono::Utc::now().to_rfc3339();
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    let short_version = format!("{}+build.{} ({})", version, run_number, git_sha);
    let long_version = format!(
        "{}\n  build: {}\n  commit: {}\n  time: {}",
        version, run_number, git_sha, build_time
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("build_info.rs");
    let contents = format!(
        r#"pub const BUILD_NUMBER: &str = "{run}";
pub const GIT_SHA: &str = "{sha}";
pub const BUILD_TIME: &str = "{time}";
pub const SHORT_VERSION: &str = "{short_v}";
pub const LONG_VERSION: &str = "{long_v}";
"#,
        run = run_number,
        sha = git_sha,
        time = build_time,
        short_v = short_version,
        long_v = long_version
    );
    fs::write(&dest, contents).expect("write build_info.rs");

    // Re-run build script when these change
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=BUILD_NUMBER");
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_NUMBER");
}

