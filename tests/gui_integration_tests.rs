#![cfg(feature = "gui")]
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_full_archive_workflow_gui() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    fs::write(&test_file, "GUI integration test")?;

    let _archive_path = work_dir.join("gui_archive.zip");

    // 1. Create archive
    let archive_result =
        test_helpers::create_test_archive(&temp_dir, &[("test.txt", "GUI integration test")]);
    assert!(archive_result.exists());

    // 2. List archive
    let list_result = test_helpers::list_archive_contents(&archive_result);
    assert!(list_result.is_ok());
    let contents = list_result.unwrap();
    assert!(contents.contains("test.txt"));

    // 3. Validate archive
    let validate_result = test_helpers::validate_archive(&archive_result);
    assert!(validate_result.is_ok());
    let validation = validate_result.unwrap();
    assert!(validation.contains("valid") || validation.contains("OK"));

    // 4. Get stats
    let stats_result = test_helpers::get_archive_stats(&archive_result);
    assert!(stats_result.is_ok());
    let stats = stats_result.unwrap();
    assert!(stats.contains("files") || stats.contains("size"));

    // 5. Extract archive
    let extract_dir = work_dir.join("extracted");
    fs::create_dir(&extract_dir)?;
    let extract_result = test_helpers::extract_archive(&archive_result, &extract_dir);
    assert!(extract_result.is_ok());
    assert!(extract_dir.join("test.txt").exists());

    // 6. Calculate hash
    let hash_result = test_helpers::calculate_file_hash(&test_file);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert!(!hash.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_archive_with_multiple_files() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create archive with multiple files
    let archive_path = test_helpers::create_test_archive(
        &temp_dir,
        &[
            ("file1.txt", "Content 1"),
            ("file2.txt", "Content 2"),
            ("file3.txt", "Content 3"),
        ],
    );

    // List contents
    let list_result = test_helpers::list_archive_contents(&archive_path);
    assert!(list_result.is_ok());
    let contents = list_result.unwrap();
    assert!(contents.contains("file1.txt"));
    assert!(contents.contains("file2.txt"));
    assert!(contents.contains("file3.txt"));

    // Extract all files
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    let extract_result = test_helpers::extract_archive(&archive_path, &extract_dir);
    assert!(extract_result.is_ok());

    // Verify all files extracted
    assert!(extract_dir.join("file1.txt").exists());
    assert!(extract_dir.join("file2.txt").exists());
    assert!(extract_dir.join("file3.txt").exists());

    // Verify content
    let content1 = fs::read_to_string(extract_dir.join("file1.txt"))?;
    let content2 = fs::read_to_string(extract_dir.join("file2.txt"))?;
    let content3 = fs::read_to_string(extract_dir.join("file3.txt"))?;

    assert_eq!(content1, "Content 1");
    assert_eq!(content2, "Content 2");
    assert_eq!(content3, "Content 3");

    Ok(())
}

#[tokio::test]
async fn test_error_handling_invalid_archive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_archive = temp_dir.path().join("invalid.zip");

    // Create invalid archive
    fs::write(&invalid_archive, "This is not a valid ZIP file")?;

    // Try to list contents (should fail)
    let list_result = test_helpers::list_archive_contents(&invalid_archive);
    assert!(list_result.is_err());

    // Try to validate (should fail)
    let validate_result = test_helpers::validate_archive(&invalid_archive);
    assert!(validate_result.is_err());

    // Try to extract (should fail)
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    let extract_result = test_helpers::extract_archive(&invalid_archive, &extract_dir);
    assert!(extract_result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_large_file_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a large file (1MB)
    let large_content = "A".repeat(1024 * 1024);
    let large_file = test_helpers::create_test_file(&temp_dir, "large.txt", &large_content);

    // Calculate hash of large file
    let hash_result = test_helpers::calculate_file_hash(&large_file);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert!(!hash.is_empty());

    // Create archive with large file
    let archive_path =
        test_helpers::create_test_archive(&temp_dir, &[("large.txt", &large_content)]);

    // Validate archive
    let validate_result = test_helpers::validate_archive(&archive_path);
    assert!(validate_result.is_ok());

    // Extract and verify
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    let extract_result = test_helpers::extract_archive(&archive_path, &extract_dir);
    assert!(extract_result.is_ok());

    let extracted_file = extract_dir.join("large.txt");
    assert!(extracted_file.exists());

    let extracted_content = fs::read_to_string(&extracted_file)?;
    assert_eq!(extracted_content, large_content);

    Ok(())
}
