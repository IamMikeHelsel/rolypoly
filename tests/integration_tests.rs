use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to run rolypoly command and capture output
fn run_rp_command(args: &[&str]) -> Result<std::process::Output> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }
    let output = Command::new("./target/release/rolypoly").args(args).output()?;
    Ok(output)
}

/// Helper function to create test files with specific content
fn create_test_files(dir: &Path) -> Result<()> {
    fs::write(dir.join("small.txt"), "Hello World")?;
    fs::write(dir.join("medium.txt"), "A".repeat(1024))?;
    fs::write(dir.join("large.txt"), "B".repeat(10240))?;

    // Create a subdirectory with files
    let subdir = dir.join("subdir");
    fs::create_dir(&subdir)?;
    fs::write(subdir.join("nested.txt"), "Nested content")?;
    fs::write(subdir.join("another.txt"), "Another nested file")?;

    // Create empty file
    fs::write(dir.join("empty.txt"), "")?;

    // Create binary-like file
    fs::write(dir.join("binary.dat"), [0u8, 1, 2, 255, 128, 64])?;

    Ok(())
}

#[test]
fn test_end_to_end_archive_workflow() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Create test files
    create_test_files(work_dir)?;

    let archive_path = work_dir.join("test_archive.zip");
    let extract_dir = work_dir.join("extracted");

    // Step 1: Create archive
    let output = run_rp_command(&[
        "create",
        archive_path.to_str().unwrap(),
        work_dir.join("small.txt").to_str().unwrap(),
        work_dir.join("medium.txt").to_str().unwrap(),
        work_dir.join("large.txt").to_str().unwrap(),
        work_dir.join("subdir").to_str().unwrap(),
        work_dir.join("empty.txt").to_str().unwrap(),
        work_dir.join("binary.dat").to_str().unwrap(),
    ])?;

    assert!(
        output.status.success(),
        "Archive creation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(archive_path.exists(), "Archive file was not created");

    // Step 2: List archive contents
    let output = run_rp_command(&["list", archive_path.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "List command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let list_output = String::from_utf8_lossy(&output.stdout);
    assert!(list_output.contains("small.txt"));
    assert!(list_output.contains("medium.txt"));
    assert!(list_output.contains("large.txt"));
    assert!(list_output.contains("nested.txt"));
    assert!(list_output.contains("another.txt"));
    assert!(list_output.contains("empty.txt"));
    assert!(list_output.contains("binary.dat"));

    // Step 3: Validate archive integrity
    let output = run_rp_command(&["validate", archive_path.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let validation_output = String::from_utf8_lossy(&output.stdout);
    assert!(validation_output.contains("Archive is valid"));

    // Step 4: Get archive statistics
    let output = run_rp_command(&["stats", archive_path.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Stats command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stats_output = String::from_utf8_lossy(&output.stdout);
    assert!(stats_output.contains("Files:"));
    assert!(stats_output.contains("Uncompressed size:"));
    assert!(stats_output.contains("Compressed size:"));
    assert!(stats_output.contains("Compression ratio:"));

    // Step 5: Extract archive
    fs::create_dir(&extract_dir)?;
    let output = run_rp_command(&[
        "extract",
        archive_path.to_str().unwrap(),
        "-o",
        extract_dir.to_str().unwrap(),
    ])?;

    assert!(
        output.status.success(),
        "Extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 6: Verify extracted files
    assert!(extract_dir.join("small.txt").exists());
    assert!(extract_dir.join("medium.txt").exists());
    assert!(extract_dir.join("large.txt").exists());
    assert!(extract_dir.join("subdir").exists());
    assert!(extract_dir.join("subdir/nested.txt").exists());
    assert!(extract_dir.join("subdir/another.txt").exists());
    assert!(extract_dir.join("empty.txt").exists());
    assert!(extract_dir.join("binary.dat").exists());

    // Step 7: Verify file contents
    assert_eq!(fs::read_to_string(extract_dir.join("small.txt"))?, "Hello World");
    assert_eq!(fs::read_to_string(extract_dir.join("medium.txt"))?, "A".repeat(1024));
    assert_eq!(fs::read_to_string(extract_dir.join("large.txt"))?, "B".repeat(10240));
    assert_eq!(fs::read_to_string(extract_dir.join("subdir/nested.txt"))?, "Nested content");
    assert_eq!(
        fs::read_to_string(extract_dir.join("subdir/another.txt"))?,
        "Another nested file"
    );
    assert_eq!(fs::read_to_string(extract_dir.join("empty.txt"))?, "");
    assert_eq!(fs::read(extract_dir.join("binary.dat"))?, [0u8, 1, 2, 255, 128, 64]);

    // Step 8: Test hash calculation
    let output = run_rp_command(&["hash", work_dir.join("small.txt").to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Hash command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let hash_output = String::from_utf8_lossy(&output.stdout);
    assert!(hash_output.contains("SHA256:"));
    assert!(hash_output.len() > 80); // SHA256 is 64 chars + prefix + newlines

    Ok(())
}

#[test]
fn test_error_handling_scenarios() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Test 1: Create archive with non-existent file
    let output = run_rp_command(&[
        "create",
        work_dir.join("test.zip").to_str().unwrap(),
        work_dir.join("nonexistent.txt").to_str().unwrap(),
    ])?;

    assert!(!output.status.success(), "Should fail with non-existent file");
    let error_output = String::from_utf8_lossy(&output.stderr);
    assert!(error_output.contains("does not exist") || error_output.contains("not found"));

    // Test 2: Create archive with no files
    let output = run_rp_command(&["create", work_dir.join("test.zip").to_str().unwrap()])?;

    assert!(!output.status.success(), "Should fail with no files");

    // Test 3: Extract non-existent archive
    let output = run_rp_command(&["extract", work_dir.join("nonexistent.zip").to_str().unwrap()])?;

    assert!(!output.status.success(), "Should fail with non-existent archive");

    // Test 4: List non-existent archive
    let output = run_rp_command(&["list", work_dir.join("nonexistent.zip").to_str().unwrap()])?;

    assert!(!output.status.success(), "Should fail with non-existent archive");

    // Test 5: Validate non-existent archive
    let output = run_rp_command(&["validate", work_dir.join("nonexistent.zip").to_str().unwrap()])?;

    assert!(!output.status.success(), "Should fail with non-existent archive");

    // Test 6: Hash non-existent file
    let output = run_rp_command(&["hash", work_dir.join("nonexistent.txt").to_str().unwrap()])?;

    assert!(!output.status.success(), "Should fail with non-existent file");

    Ok(())
}

#[test]
fn test_large_file_handling() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Create a large file (1MB)
    let large_file = work_dir.join("large_file.txt");
    let content = "X".repeat(1024 * 1024); // 1MB of X's
    fs::write(&large_file, &content)?;

    let archive_path = work_dir.join("large_test.zip");
    let extract_dir = work_dir.join("extracted");

    // Create archive with large file
    let output =
        run_rp_command(&["create", archive_path.to_str().unwrap(), large_file.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Large file archiving failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(archive_path.exists());

    // Validate the archive
    let output = run_rp_command(&["validate", archive_path.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Large file validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Extract and verify
    fs::create_dir(&extract_dir)?;
    let output = run_rp_command(&[
        "extract",
        archive_path.to_str().unwrap(),
        "-o",
        extract_dir.to_str().unwrap(),
    ])?;

    assert!(
        output.status.success(),
        "Large file extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let extracted_content = fs::read_to_string(extract_dir.join("large_file.txt"))?;
    assert_eq!(extracted_content, content, "Large file content mismatch");

    Ok(())
}

#[test]
fn test_special_characters_and_unicode() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Create files with special characters and unicode
    let files = vec![
        ("unicode_🦀.txt", "Hello 🦀 Rust! 你好世界"),
        ("spaces in name.txt", "File with spaces"),
        ("special!@#$%^&*().txt", "Special characters"),
    ];

    for (filename, content) in &files {
        fs::write(work_dir.join(filename), content)?;
    }

    let archive_path = work_dir.join("unicode_test.zip");
    let extract_dir = work_dir.join("extracted");

    // Create archive
    let mut args = vec!["create", archive_path.to_str().unwrap()];
    let file_paths: Vec<String> = files
        .iter()
        .map(|(filename, _)| work_dir.join(filename).to_string_lossy().to_string())
        .collect();
    for path in &file_paths {
        args.push(path);
    }

    let output = run_rp_command(&args)?;
    assert!(
        output.status.success(),
        "Unicode file archiving failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Extract and verify
    fs::create_dir(&extract_dir)?;
    let output = run_rp_command(&[
        "extract",
        archive_path.to_str().unwrap(),
        "-o",
        extract_dir.to_str().unwrap(),
    ])?;

    assert!(
        output.status.success(),
        "Unicode file extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify content
    for (filename, expected_content) in &files {
        let extracted_content = fs::read_to_string(extract_dir.join(filename))?;
        assert_eq!(
            &extracted_content, expected_content,
            "Unicode content mismatch for {}",
            filename
        );
    }

    Ok(())
}

#[test]
fn test_compression_effectiveness() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Create highly compressible file
    let compressible_file = work_dir.join("compressible.txt");
    let compressible_content = "AAAAAAAAAA\n".repeat(1000); // Very repetitive content
    fs::write(&compressible_file, &compressible_content)?;

    // Create incompressible file (random-like)
    let incompressible_file = work_dir.join("incompressible.txt");
    let incompressible_content = (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>();
    fs::write(&incompressible_file, &incompressible_content)?;

    let archive_path = work_dir.join("compression_test.zip");

    // Create archive
    let output = run_rp_command(&[
        "create",
        archive_path.to_str().unwrap(),
        compressible_file.to_str().unwrap(),
        incompressible_file.to_str().unwrap(),
    ])?;

    assert!(
        output.status.success(),
        "Compression test archiving failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Get statistics
    let output = run_rp_command(&["stats", archive_path.to_str().unwrap()])?;

    assert!(
        output.status.success(),
        "Compression stats failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stats_output = String::from_utf8_lossy(&output.stdout);
    assert!(stats_output.contains("Compression ratio:"));
    assert!(stats_output.contains("Files: 2"));

    // The compressible file should result in good compression
    // This is more of a sanity check that compression is working

    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();

    // Create test files
    for i in 0..5 {
        fs::write(work_dir.join(format!("file_{}.txt", i)), format!("Content {}", i))?;
    }

    // Create multiple archives concurrently (simulate concurrent usage)
    let mut handles = vec![];

    for i in 0..3 {
        let work_dir = work_dir.to_path_buf();
        let handle = std::thread::spawn(move || -> Result<()> {
            let archive_path = work_dir.join(format!("concurrent_{}.zip", i));
            let output = run_rp_command(&[
                "create",
                archive_path.to_str().unwrap(),
                work_dir.join("file_0.txt").to_str().unwrap(),
                work_dir.join("file_1.txt").to_str().unwrap(),
            ])?;

            assert!(output.status.success(), "Concurrent operation {} failed", i);
            assert!(archive_path.exists());

            // Validate the archive
            let output = run_rp_command(&["validate", archive_path.to_str().unwrap()])?;

            assert!(output.status.success(), "Concurrent validation {} failed", i);
            Ok(())
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[test]
fn test_cli_help_and_version() -> Result<()> {
    // Ensure release binary exists
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    // Test --help
    let output = run_rp_command(&["--help"])?;
    assert!(output.status.success(), "Help command failed");
    let help_output = String::from_utf8_lossy(&output.stdout);
    assert!(help_output.contains("Usage:"));
    assert!(help_output.contains("Commands:"));
    assert!(help_output.contains("create"));
    assert!(help_output.contains("extract"));
    assert!(help_output.contains("list"));
    assert!(help_output.contains("validate"));
    assert!(help_output.contains("stats"));
    assert!(help_output.contains("hash"));

    // Test --version
    let output = run_rp_command(&["--version"])?;
    assert!(output.status.success(), "Version command failed");
    let version_output = String::from_utf8_lossy(&output.stdout);
    assert!(version_output.contains("0.1.0"));

    // Test help for specific commands
    let output = run_rp_command(&["create", "--help"])?;
    assert!(output.status.success(), "Create help failed");
    let create_help = String::from_utf8_lossy(&output.stdout);
    assert!(create_help.contains("Create a new ZIP archive"));

    Ok(())
}
