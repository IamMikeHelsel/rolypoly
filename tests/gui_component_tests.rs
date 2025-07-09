use anyhow::Result;
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_create_archive_component() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file_to_compress = temp_dir.path().join("test_file.txt");
    fs::write(&file_to_compress, "Hello from GUI test!")?;

    let _archive_path = temp_dir.path().join("gui_test.zip");

    // Use our test helper to create archive
    let result = test_helpers::create_test_archive(&temp_dir, &[("test_file.txt", "Hello from GUI test!")]);
    
    assert!(result.exists());
    Ok(())
}

#[tokio::test]
async fn test_extract_archive_component() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    
    // Create test archive
    let archive_path = test_helpers::create_test_archive(&temp_dir, &[
        ("file1.txt", "Content 1"),
        ("file2.txt", "Content 2"),
    ]);
    
    // Extract archive
    let result = test_helpers::extract_archive(&archive_path, &extract_dir);
    assert!(result.is_ok());
    
    // Verify extracted files
    assert!(extract_dir.join("file1.txt").exists());
    assert!(extract_dir.join("file2.txt").exists());
    
    let content1 = fs::read_to_string(extract_dir.join("file1.txt"))?;
    let content2 = fs::read_to_string(extract_dir.join("file2.txt"))?;
    
    assert_eq!(content1, "Content 1");
    assert_eq!(content2, "Content 2");
    
    Ok(())
}

#[tokio::test]
async fn test_list_archive_component() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test archive
    let archive_path = test_helpers::create_test_archive(&temp_dir, &[
        ("file1.txt", "Content 1"),
        ("file2.txt", "Content 2"),
    ]);
    
    // List archive contents
    let result = test_helpers::list_archive_contents(&archive_path);
    assert!(result.is_ok());
    
    let contents = result.unwrap();
    assert!(contents.contains("file1.txt"));
    assert!(contents.contains("file2.txt"));
    
    Ok(())
}

#[tokio::test]
async fn test_validate_archive_component() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test archive
    let archive_path = test_helpers::create_test_archive(&temp_dir, &[
        ("file1.txt", "Content 1"),
    ]);
    
    // Validate archive
    let result = test_helpers::validate_archive(&archive_path);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.contains("valid") || output.contains("OK"));
    
    Ok(())
}

#[tokio::test]
async fn test_hash_file_component() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = test_helpers::create_test_file(&temp_dir, "test.txt", "Hello World!");
    
    // Calculate hash
    let result = test_helpers::calculate_file_hash(&test_file);
    assert!(result.is_ok());
    
    let hash = result.unwrap();
    assert!(!hash.is_empty());
    assert!(hash.len() >= 64); // SHA256 hash length
    
    Ok(())
}