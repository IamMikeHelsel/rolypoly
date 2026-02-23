use rolypoly::archive::ArchiveManager;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

#[test]
fn test_zip_slip_vulnerability() -> anyhow::Result<()> {
    // 1. Setup temporary directory
    let temp_dir = TempDir::new()?;
    let zip_path = temp_dir.path().join("malicious.zip");
    let extract_dir = temp_dir.path().join("extract");
    let evil_path = temp_dir.path().join("evil.txt");

    // 2. Create a malicious ZIP file with path traversal
    {
        let file = File::create(&zip_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        // Attempt to create a file entry that points outside the extraction directory
        // In the zip crate, paths are stored as given.
        zip.start_file("../evil.txt", options)?;
        zip.write_all(b"I am evil")?;
        zip.finish()?;
    }

    // 3. Create extraction directory
    fs::create_dir(&extract_dir)?;

    // 4. Attempt to extract
    let manager = ArchiveManager::new();
    let result = manager.extract_archive(&zip_path, &extract_dir);

    // 5. Assertions
    // If the vulnerability exists, the file will be created at `evil_path` (outside `extract_dir`).
    if evil_path.exists() {
        // Fail the test if the file was created outside
        panic!(
            "SECURITY VULNERABILITY: File created outside extraction directory at {}",
            evil_path.display()
        );
    }

    // We also expect the extraction to return an error for invalid paths,
    // but the critical security check is that the file is NOT created outside.
    // If the library silently skips invalid paths, that's also secure (though less informative).
    // However, our fix will return an error.
    assert!(result.is_err(), "Extraction should fail for malicious archives");

    Ok(())
}
