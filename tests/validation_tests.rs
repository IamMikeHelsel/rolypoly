use anyhow::Result;
use std::fs;
use tempfile::TempDir;

// Helper to find binary
fn get_binary_path() -> std::path::PathBuf {
    let root = std::env::current_dir().unwrap();
    let debug_bin = root.join("target/debug/rolypoly");
    let release_bin = root.join("target/release/rolypoly");

    if debug_bin.exists() {
        return debug_bin;
    }
    if release_bin.exists() {
        return release_bin;
    }
    // Assume cargo run if not found, but for these tests we prefer binary if built
    std::path::PathBuf::from("cargo")
}

fn run_rolypoly(args: &[&str]) -> std::process::Output {
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

#[test]
fn test_path_traversal_protection() -> Result<()> {
    // Create a zip file that has a file with path "../evil.txt"
    // We use the `zip` crate directly to create this malicious archive,
    // bypassing the CLI which might sanitize it during creation.

    let temp_dir = TempDir::new()?;
    let malicious_zip = temp_dir.path().join("malicious.zip");
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir(&extract_dir)?;

    {
        let file = fs::File::create(&malicious_zip)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        // Add a file with parent directory traversal
        // Note: Some zip readers strip this, but we want to ensure OUR extractor handles it safely.
        // We force the name string.
        zip.start_file("../evil.txt", options)?;
        use std::io::Write;
        zip.write_all(b"I am evil")?;
        zip.finish()?;
    }

    // Now try to extract it using rolypoly
    let output = run_rolypoly(&[
        "extract",
        malicious_zip.to_str().unwrap(),
        "-o",
        extract_dir.to_str().unwrap(),
    ]);

    // It should either fail, OR succeed but strip the "../" part.
    // It should NOT write to temp_dir/evil.txt (outside extract_dir).

    let evil_file_outside = extract_dir.parent().unwrap().join("evil.txt");
    assert!(!evil_file_outside.exists(), "Security vulnerability: Path traversal allowed writing outside extraction directory!");

    // Check if it wrote inside (sanitized)
    let evil_file_inside = extract_dir.join("evil.txt");
    let _evil_file_inside_escaped = extract_dir.join("..").join("evil.txt"); // This resolves to outside

    // If the extractor is safe, it might have stripped ".." -> "evil.txt" inside extract_dir,
    // or it might have rejected the file.

    // Let's see output
    if output.status.success() {
        // If it succeeded, verify it didn't escape. We already did that.
        // Verify where it went.
        if evil_file_inside.exists() {
            println!("Sanitized path traversal correctly.");
        } else {
             println!("Maybe rejected the file? Output: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        println!("Extraction failed safely: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

#[test]
fn test_corrupted_archive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let corrupt_zip = temp_dir.path().join("corrupt.zip");

    // Write junk
    fs::write(&corrupt_zip, b"PK\x03\x04This is not a valid zip file structure but looks like one maybe")?;

    let output = run_rolypoly(&["validate", corrupt_zip.to_str().unwrap()]);

    assert!(!output.status.success(), "Should fail to validate corrupt archive");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("invalid") || stderr.contains("failed"));

    Ok(())
}

#[test]
fn test_symlink_handling() -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        let target_file = src_dir.join("target.txt");
        fs::write(&target_file, "Target content")?;

        let link_file = src_dir.join("link.txt");
        symlink("target.txt", &link_file)?;

        let archive_path = temp_dir.path().join("symlinks.zip");

        let output = run_rolypoly(&[
            "create",
            archive_path.to_str().unwrap(),
            src_dir.to_str().unwrap(),
        ]);
        assert!(output.status.success());

        // Now extract and check
        let extract_dir = temp_dir.path().join("extract");
        fs::create_dir(&extract_dir)?;

        let output = run_rolypoly(&[
            "extract",
            archive_path.to_str().unwrap(),
            "-o",
            extract_dir.to_str().unwrap(),
        ]);
        assert!(output.status.success());

        // Check content of link.txt
        // Current implementation likely follows symlinks and stores as file
        let extracted_link = extract_dir.join("src").join("link.txt");
        assert!(extracted_link.exists());
        assert!(extracted_link.is_file()); // It should be a file, not a symlink in extraction (unless we supported preserving them)

        let content = fs::read_to_string(&extracted_link)?;
        assert_eq!(content, "Target content");
    }
    Ok(())
}
