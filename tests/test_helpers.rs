#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use std::env;

/// Returns the path to the built binary, or falls back to "cargo run"
pub fn get_binary_path() -> PathBuf {
    // Try to find the binary in typical locations
    // For tests, we are usually in the root.
    // We can try to use CARGO_BIN_EXE_rolypoly if set (integration tests usually set this)
    // But these are component tests via a separate file, so might not be set automatically for these helpers.

    // Check typical debug/release paths
    let root = env::current_dir().unwrap();
    let debug_bin = root.join("target/debug/rolypoly");
    let release_bin = root.join("target/release/rolypoly");

    if debug_bin.exists() {
        return debug_bin;
    }
    if release_bin.exists() {
        return release_bin;
    }

    // Fallback to "cargo" but we will return a dummy path and handle it in the run_command
    PathBuf::from("cargo")
}

fn run_command(args: &[&str]) -> std::process::Output {
    let bin = get_binary_path();
    if bin.to_string_lossy() == "cargo" {
         std::process::Command::new("cargo")
            .args(["run", "--bin", "rolypoly", "--"])
            .args(args)
            .output()
            .expect("Failed to run cargo")
    } else {
        std::process::Command::new(bin)
            .args(args)
            .output()
            .expect("Failed to run binary")
    }
}

pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).unwrap();
    file_path
}

pub fn create_test_archive(dir: &TempDir, files: &[(&str, &str)]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.zip");

    // Create test files
    let mut file_paths = Vec::new();
    for (name, content) in files {
        let file_path = create_test_file(dir, name, content);
        file_paths.push(file_path);
    }

    // Create archive using our CLI
    let mut args = vec!["create", archive_path.to_str().unwrap()];
    for p in &file_paths {
        args.push(p.to_str().unwrap());
    }

    let output = run_command(&args);

    if !output.status.success() {
        panic!("Failed to create test archive: {}", String::from_utf8_lossy(&output.stderr));
    }

    archive_path
}

pub fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<(), String> {
    let output = run_command(&[
            "extract",
            archive_path.to_str().unwrap(),
            "-o",
            output_dir.to_str().unwrap(),
    ]);

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn list_archive_contents(archive_path: &Path) -> Result<String, String> {
    let output = run_command(&["list", archive_path.to_str().unwrap()]);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn validate_archive(archive_path: &Path) -> Result<String, String> {
    let output = run_command(&["validate", archive_path.to_str().unwrap()]);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[allow(dead_code)]
pub fn get_archive_stats(archive_path: &Path) -> Result<String, String> {
    let output = run_command(&["stats", archive_path.to_str().unwrap()]);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn calculate_file_hash(file_path: &Path) -> Result<String, String> {
    let output = run_command(&["hash", file_path.to_str().unwrap()]);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
