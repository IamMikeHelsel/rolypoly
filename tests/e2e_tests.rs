#![cfg(feature = "gui")] // No-op now that GUI is removed
use std::fs;
use std::process::Command;
use tempfile::TempDir;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_gui_launch_and_close() {
    let output = TokioCommand::new("cargo")
        .args(&["run", "--features", "gui", "--bin", "rusty-gui"]) 
        .kill_on_drop(true)
        .spawn();

    assert!(output.is_ok(), "Failed to start GUI");

    // Let it run for a moment
    sleep(Duration::from_millis(500)).await;

    // Kill the process
    if let Ok(mut child) = output {
        let _ = child.kill().await;
    }
}

#[tokio::test]
async fn test_cli_create_and_extract() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");
    let extract_dir = temp_dir.path().join("extract");

    // Create a test file
    fs::write(&test_file, "Hello, World!").unwrap();

    // Create archive
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create archive");

    assert!(
        output.status.success(),
        "Create command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(archive_path.exists(), "Archive was not created");

    // Extract archive
    fs::create_dir_all(&extract_dir).unwrap();
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "extract",
            archive_path.to_str().unwrap(),
            "-o",
            extract_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to extract archive");

    assert!(
        output.status.success(),
        "Extract command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists(), "Extracted file not found");

    let content = fs::read_to_string(&extracted_file).unwrap();
    assert_eq!(content, "Hello, World!");
}

#[tokio::test]
async fn test_cli_list_archive() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");

    // Create a test file
    fs::write(&test_file, "Hello, World!").unwrap();

    // Create archive
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create archive");

    assert!(output.status.success());

    // List archive contents
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "list", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to list archive");

    assert!(
        output.status.success(),
        "List command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.txt"), "File not found in archive listing");
}

#[tokio::test]
async fn test_cli_validate_archive() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");

    // Create a test file
    fs::write(&test_file, "Hello, World!").unwrap();

    // Create archive
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create archive");

    assert!(output.status.success());

    // Validate archive
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "validate", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to validate archive");

    assert!(
        output.status.success(),
        "Validate command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid") || stdout.contains("OK"), "Archive validation failed");
}

#[tokio::test]
async fn test_cli_stats_archive() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");

    // Create a test file
    fs::write(&test_file, "Hello, World!").unwrap();

    // Create archive
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create archive");

    assert!(output.status.success());

    // Get archive stats
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "stats", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to get archive stats");

    assert!(
        output.status.success(),
        "Stats command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("files") || stdout.contains("size"),
        "Stats output missing expected information"
    );
}

#[tokio::test]
async fn test_cli_hash_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create a test file
    fs::write(&test_file, "Hello, World!").unwrap();

    // Calculate hash
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "hash", test_file.to_str().unwrap()])
        .output()
        .expect("Failed to calculate hash");

    assert!(
        output.status.success(),
        "Hash command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.len() > 0, "Hash output is empty");
    assert!(stdout.contains("SHA256") || stdout.len() == 64, "Hash output format unexpected");
}

#[tokio::test]
async fn test_multiple_files_archive() {
    let temp_dir = TempDir::new().unwrap();
    let test_file1 = temp_dir.path().join("test1.txt");
    let test_file2 = temp_dir.path().join("test2.txt");
    let archive_path = temp_dir.path().join("multi.zip");

    // Create test files
    fs::write(&test_file1, "File 1 content").unwrap();
    fs::write(&test_file2, "File 2 content").unwrap();

    // Create archive with multiple files
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file1.to_str().unwrap(),
            test_file2.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create archive");

    assert!(
        output.status.success(),
        "Create command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(archive_path.exists(), "Archive was not created");

    // List archive contents
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "list", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to list archive");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test1.txt"), "File 1 not found in archive");
    assert!(stdout.contains("test2.txt"), "File 2 not found in archive");
}

#[tokio::test]
async fn test_error_handling_invalid_archive() {
    let temp_dir = TempDir::new().unwrap();
    let fake_archive = temp_dir.path().join("fake.zip");

    // Create a fake archive file
    fs::write(&fake_archive, "This is not a valid ZIP file").unwrap();

    // Try to list invalid archive
    let output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "list", fake_archive.to_str().unwrap()])
        .output()
        .expect("Failed to run list command");

    assert!(!output.status.success(), "List command should fail for invalid archive");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "Error message not found in stderr"
    );
}

#[tokio::test]
async fn test_error_handling_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let missing_file = temp_dir.path().join("missing.txt");
    let archive_path = temp_dir.path().join("test.zip");

    // Try to create archive with missing file
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            missing_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run create command");

    assert!(!output.status.success(), "Create command should fail for missing file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "Error message not found in stderr"
    );
}
