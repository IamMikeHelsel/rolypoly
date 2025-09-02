use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

mod test_helpers;

/// Test that CLI and GUI (through ArchiveManager) produce identical results
#[tokio::test]
async fn test_cli_gui_parity_create_archive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for parity check")?;

    // Create archive using CLI
    let cli_archive = temp_dir.path().join("cli_archive.zip");
    let cli_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            cli_archive.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()?;

    assert!(cli_output.status.success(), "CLI archive creation failed");
    assert!(cli_archive.exists(), "CLI archive not created");

    // Create archive using GUI backend (ArchiveManager)
    let gui_archive = temp_dir.path().join("gui_archive.zip");
    let archive_manager = rusty::archive::ArchiveManager::new();
    archive_manager.create_archive(&gui_archive, &[&test_file])?;

    assert!(gui_archive.exists(), "GUI archive not created");

    // Compare archive contents
    let cli_contents =
        test_helpers::list_archive_contents(&cli_archive).map_err(|e| anyhow::anyhow!(e))?;
    let gui_contents =
        test_helpers::list_archive_contents(&gui_archive).map_err(|e| anyhow::anyhow!(e))?;

    // Extract just the file listings (skip the header line)
    let cli_files: Vec<&str> = cli_contents.lines().skip(1).map(|l| l.trim()).collect();
    let gui_files: Vec<&str> = gui_contents.lines().skip(1).map(|l| l.trim()).collect();

    assert_eq!(cli_files, gui_files, "CLI and GUI archives have different contents");

    // Verify both archives can be extracted and produce identical results
    let cli_extract_dir = temp_dir.path().join("cli_extract");
    let gui_extract_dir = temp_dir.path().join("gui_extract");

    fs::create_dir_all(&cli_extract_dir)?;
    fs::create_dir_all(&gui_extract_dir)?;

    test_helpers::extract_archive(&cli_archive, &cli_extract_dir)
        .map_err(|e| anyhow::anyhow!(e))?;
    test_helpers::extract_archive(&gui_archive, &gui_extract_dir)
        .map_err(|e| anyhow::anyhow!(e))?;

    let cli_extracted = fs::read_to_string(cli_extract_dir.join("test.txt"))?;
    let gui_extracted = fs::read_to_string(gui_extract_dir.join("test.txt"))?;

    assert_eq!(cli_extracted, gui_extracted, "Extracted files differ between CLI and GUI");

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_extract_archive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for extraction parity")?;

    // Create archive using CLI
    let archive_path = temp_dir.path().join("test_archive.zip");
    let create_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()?;

    assert!(create_output.status.success(), "Archive creation failed");

    // Extract using CLI
    let cli_extract_dir = temp_dir.path().join("cli_extract");
    fs::create_dir_all(&cli_extract_dir)?;
    let cli_extract_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "extract",
            archive_path.to_str().unwrap(),
            "-o",
            cli_extract_dir.to_str().unwrap(),
        ])
        .output()?;

    assert!(cli_extract_output.status.success(), "CLI extraction failed");

    // Extract using GUI backend (ArchiveManager)
    let gui_extract_dir = temp_dir.path().join("gui_extract");
    fs::create_dir_all(&gui_extract_dir)?;
    let archive_manager = rusty::archive::ArchiveManager::new();
    archive_manager.extract_archive(&archive_path, &gui_extract_dir)?;

    // Compare extracted contents
    let cli_extracted = fs::read_to_string(cli_extract_dir.join("test.txt"))?;
    let gui_extracted = fs::read_to_string(gui_extract_dir.join("test.txt"))?;

    assert_eq!(cli_extracted, gui_extracted, "CLI and GUI extraction results differ");

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_validate_archive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for validation parity")?;

    // Create archive
    let archive_path = temp_dir.path().join("test_archive.zip");
    let create_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            archive_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()?;

    assert!(create_output.status.success(), "Archive creation failed");

    // Validate using CLI
    let cli_validate_output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "validate", archive_path.to_str().unwrap()])
        .output()?;

    let cli_success = cli_validate_output.status.success();

    // Validate using GUI backend (ArchiveManager)
    let archive_manager = rusty::archive::ArchiveManager::new();
    let gui_result = archive_manager.validate_archive(&archive_path);

    match (cli_success, gui_result) {
        (true, Ok(true)) => {
            // Both succeeded - good
        }
        (false, Err(_)) => {
            // Both failed - consistent
        }
        _ => {
            panic!("CLI and GUI validation results are inconsistent");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_hash_calculation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for hash parity")?;

    // Calculate hash using CLI
    let cli_hash_output = Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "hash", test_file.to_str().unwrap()])
        .output()?;

    assert!(cli_hash_output.status.success(), "CLI hash calculation failed");
    let cli_output = String::from_utf8_lossy(&cli_hash_output.stdout);
    
    // Extract just the hash value from CLI output (format: "SHA256: <hash>")
    let cli_hash = cli_output
        .lines()
        .find(|line| line.starts_with("SHA256: "))
        .and_then(|line| line.strip_prefix("SHA256: "))
        .unwrap_or("")
        .trim()
        .to_string();

    // Calculate hash using GUI backend (ArchiveManager)
    let archive_manager = rusty::archive::ArchiveManager::new();
    let gui_hash = archive_manager.calculate_file_hash(&test_file)?;

    assert_eq!(cli_hash, gui_hash, "CLI and GUI hash calculations differ");

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_large_file_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let large_file = temp_dir.path().join("large_file.txt");

    // Create a 1MB file
    let large_content = "A".repeat(1024 * 1024);
    fs::write(&large_file, &large_content)?;

    // Create archive using CLI
    let cli_archive = temp_dir.path().join("cli_large.zip");
    let cli_create_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            cli_archive.to_str().unwrap(),
            large_file.to_str().unwrap(),
        ])
        .output()?;

    assert!(cli_create_output.status.success(), "CLI large file archive creation failed");

    // Create archive using GUI backend
    let gui_archive = temp_dir.path().join("gui_large.zip");
    let archive_manager = rusty::archive::ArchiveManager::new();
    archive_manager.create_archive(&gui_archive, &[&large_file])?;

    // Extract both archives and compare
    let cli_extract_dir = temp_dir.path().join("cli_extract");
    let gui_extract_dir = temp_dir.path().join("gui_extract");

    fs::create_dir_all(&cli_extract_dir)?;
    fs::create_dir_all(&gui_extract_dir)?;

    test_helpers::extract_archive(&cli_archive, &cli_extract_dir)
        .map_err(|e| anyhow::anyhow!(e))?;
    archive_manager.extract_archive(&gui_archive, &gui_extract_dir)?;

    let cli_extracted = fs::read_to_string(cli_extract_dir.join("large_file.txt"))?;
    let gui_extracted = fs::read_to_string(gui_extract_dir.join("large_file.txt"))?;

    assert_eq!(cli_extracted, gui_extracted, "Large file extraction results differ");
    assert_eq!(cli_extracted.len(), large_content.len(), "Extracted file size incorrect");

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_multiple_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut files = Vec::new();

    // Create multiple test files
    for i in 0..5 {
        let file_path = temp_dir.path().join(format!("test_{}.txt", i));
        fs::write(&file_path, format!("Content of file {}", i))?;
        files.push(file_path);
    }

    // Create archive using CLI
    let cli_archive = temp_dir.path().join("cli_multi.zip");
    let mut cli_args = vec!["run", "--bin", "rusty", "--", "create", cli_archive.to_str().unwrap()];
    for file in &files {
        cli_args.push(file.to_str().unwrap());
    }

    let cli_create_output = Command::new("cargo").args(&cli_args).output()?;

    assert!(cli_create_output.status.success(), "CLI multi-file archive creation failed");

    // Create archive using GUI backend
    let gui_archive = temp_dir.path().join("gui_multi.zip");
    let archive_manager = rusty::archive::ArchiveManager::new();
    let file_refs: Vec<&std::path::PathBuf> = files.iter().collect();
    archive_manager.create_archive(&gui_archive, &file_refs)?;

    // List contents of both archives
    let cli_contents =
        test_helpers::list_archive_contents(&cli_archive).map_err(|e| anyhow::anyhow!(e))?;
    let gui_contents =
        test_helpers::list_archive_contents(&gui_archive).map_err(|e| anyhow::anyhow!(e))?;

    // Both should contain all files
    for i in 0..5 {
        let filename = format!("test_{}.txt", i);
        assert!(cli_contents.contains(&filename), "CLI archive missing file: {}", filename);
        assert!(gui_contents.contains(&filename), "GUI archive missing file: {}", filename);
    }

    Ok(())
}

#[tokio::test]
async fn test_cli_gui_parity_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let nonexistent_file = temp_dir.path().join("nonexistent.txt");
    let output_archive = temp_dir.path().join("error_test.zip");

    // Test CLI error handling
    let cli_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            output_archive.to_str().unwrap(),
            nonexistent_file.to_str().unwrap(),
        ])
        .output()?;

    let cli_failed = !cli_output.status.success();

    // Test GUI error handling
    let archive_manager = rusty::archive::ArchiveManager::new();
    let gui_result = archive_manager.create_archive(&output_archive, &[&nonexistent_file]);
    let gui_failed = gui_result.is_err();

    // Both should fail consistently
    assert_eq!(cli_failed, gui_failed, "CLI and GUI error handling inconsistent");

    Ok(())
}

#[tokio::test]
async fn test_performance_parity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("perf_test.txt");
    let content = "Performance test content".repeat(10000);
    fs::write(&test_file, &content)?;

    // Measure CLI performance
    let cli_start = std::time::Instant::now();
    let cli_archive = temp_dir.path().join("cli_perf.zip");
    let cli_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "rusty",
            "--",
            "create",
            cli_archive.to_str().unwrap(),
            test_file.to_str().unwrap(),
        ])
        .output()?;
    let cli_duration = cli_start.elapsed();

    assert!(cli_output.status.success(), "CLI performance test failed");

    // Measure GUI performance
    let gui_start = std::time::Instant::now();
    let gui_archive = temp_dir.path().join("gui_perf.zip");
    let archive_manager = rusty::archive::ArchiveManager::new();
    archive_manager.create_archive(&gui_archive, &[&test_file])?;
    let gui_duration = gui_start.elapsed();

    // Since CLI includes cargo run overhead, we expect GUI to be faster
    // Just verify both completed successfully and in reasonable time
    assert!(
        cli_duration.as_secs() < 10,
        "CLI took too long: {}ms",
        cli_duration.as_millis()
    );
    assert!(
        gui_duration.as_secs() < 2,
        "GUI took too long: {}ms", 
        gui_duration.as_millis()
    );

    // Log the performance for informational purposes
    eprintln!(
        "Performance: CLI={}ms (includes cargo run overhead), GUI={}ms (direct call)",
        cli_duration.as_millis(),
        gui_duration.as_millis()
    );

    Ok(())
}
