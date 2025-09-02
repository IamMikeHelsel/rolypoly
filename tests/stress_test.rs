use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

fn create_large_test_dataset(dir: &Path) -> Result<(usize, f64), Box<dyn std::error::Error>> {
    let mut total_size = 0u64;
    let mut file_count = 0;

    // Create many small files (simulating source code project)
    for i in 0..1000 {
        let content =
            format!("// File {i}\nfn main() {{\n    println!(\"Hello from file {i}\");\n}}\n");
        let file_path = dir.join(format!("source_{i}.rs"));
        fs::write(&file_path, &content)?;
        total_size += content.len() as u64;
        file_count += 1;
    }

    // Create some larger files
    for i in 0..50 {
        let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(1000);
        let file_path = dir.join(format!("doc_{i}.txt"));
        fs::write(&file_path, &content)?;
        total_size += content.len() as u64;
        file_count += 1;
    }

    // Create binary-like files
    for i in 0..10 {
        let binary_content: Vec<u8> = (0..1024 * 100).map(|j| ((i + j) % 256) as u8).collect();
        let file_path = dir.join(format!("binary_{i}.dat"));
        fs::write(&file_path, &binary_content)?;
        total_size += binary_content.len() as u64;
        file_count += 1;
    }

    Ok((file_count, total_size as f64 / 1024.0 / 1024.0))
}

#[test]
#[ignore] // Run with cargo test -- --ignored stress_test
fn stress_test_large_archive() -> Result<(), Box<dyn std::error::Error>> {
    // Build rolypoly if needed
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(&["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("stress_test");
    fs::create_dir(&test_dir)?;

    println!("Creating large test dataset...");
    let (file_count, total_size_mb) = create_large_test_dataset(&test_dir)?;
    println!("Created {file_count} files, {total_size_mb:.2} MB total");

    let archive_path = temp_dir.path().join("stress_test.zip");

    // Test creation
    println!("Testing archive creation...");
    let start = Instant::now();

    let mut cmd = Command::new("./target/release/rolypoly");
    cmd.arg("create").arg(&archive_path);

    // Add all files
    for entry in fs::read_dir(&test_dir)? {
        let entry = entry?;
        cmd.arg(entry.path());
    }

    let output = cmd.output()?;
    let create_time = start.elapsed();

    assert!(
        output.status.success(),
        "Archive creation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(archive_path.exists(), "Archive was not created");

    let archive_size_mb = fs::metadata(&archive_path)?.len() as f64 / 1024.0 / 1024.0;
    let compression_ratio = archive_size_mb / total_size_mb;
    let create_throughput = total_size_mb / create_time.as_secs_f64();

    println!("Creation completed:");
    println!("  Time: {:.2}s", create_time.as_secs_f64());
    println!("  Throughput: {create_throughput:.2} MB/s");
    println!(
        "  Compression: {:.1}% ({:.2} MB -> {:.2} MB)",
        compression_ratio * 100.0,
        total_size_mb,
        archive_size_mb
    );

    // Test validation
    println!("\nTesting archive validation...");
    let start = Instant::now();
    let output = Command::new("./target/release/rolypoly")
        .args(["validate", archive_path.to_str().unwrap()])
        .output()?;
    let validate_time = start.elapsed();

    assert!(
        output.status.success(),
        "Validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("Validation completed in {:.2}s", validate_time.as_secs_f64());

    // Test extraction
    println!("\nTesting archive extraction...");
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir(&extract_dir)?;

    let start = Instant::now();
    let output = Command::new("./target/release/rolypoly")
        .args(["extract", archive_path.to_str().unwrap(), "-o", extract_dir.to_str().unwrap()])
        .output()?;
    let extract_time = start.elapsed();

    assert!(
        output.status.success(),
        "Extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let extract_throughput = total_size_mb / extract_time.as_secs_f64();
    println!("Extraction completed:");
    println!("  Time: {:.2}s", extract_time.as_secs_f64());
    println!("  Throughput: {:.2} MB/s", extract_throughput);

    // Verify extraction
    let extracted_files = fs::read_dir(&extract_dir)?.count();
    assert_eq!(extracted_files, file_count, "Not all files were extracted");
    println!("  All {} files extracted successfully", extracted_files);

    // Performance expectations for stress test
    assert!(
        create_throughput > 5.0,
        "Create throughput should be > 5 MB/s, got {:.2}",
        create_throughput
    );
    assert!(
        extract_throughput > 20.0,
        "Extract throughput should be > 20 MB/s, got {:.2}",
        extract_throughput
    );
    assert!(
        compression_ratio < 0.8,
        "Should achieve reasonable compression, got {:.1}%",
        compression_ratio * 100.0
    );

    println!("\n✅ Stress test passed!");

    Ok(())
}

#[test]
fn memory_usage_test() -> Result<(), Box<dyn std::error::Error>> {
    // Build rolypoly if needed
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(&["build", "--release"]).status()?;
    }

    let temp_dir = TempDir::new()?;

    // Create a large single file (10MB)
    let large_file = temp_dir.path().join("large_file.txt");
    let content = "A".repeat(10 * 1024 * 1024); // 10MB
    fs::write(&large_file, &content)?;

    let archive_path = temp_dir.path().join("large.zip");

    // Test with large file - this should not cause memory issues
    let output = Command::new("./target/release/rolypoly")
        .args(&["create", archive_path.to_str().unwrap(), large_file.to_str().unwrap()])
        .output()?;

    assert!(
        output.status.success(),
        "Large file archiving failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Validate the large archive
    let output = Command::new("./target/release/rolypoly")
        .args(["validate", archive_path.to_str().unwrap()])
        .output()?;

    assert!(
        output.status.success(),
        "Large file validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✅ Memory usage test with 10MB file passed");

    Ok(())
}
