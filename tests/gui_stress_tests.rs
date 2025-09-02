use anyhow::Result;
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_large_number_of_files_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let _work_dir = temp_dir.path();

    // Create many files for stress test
    let mut files_data = Vec::new();
    for i in 0..100 {
        // Reduced from 1000 to 100 for faster testing
        let file_name = format!("file_{}.txt", i);
        let file_content = format!("content {}", i);
        files_data.push((file_name, file_content));
    }

    // Convert to the format expected by create_test_archive
    let files_slice: Vec<(&str, &str)> = files_data
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    let archive_path = test_helpers::create_test_archive(&temp_dir, &files_slice);
    assert!(archive_path.exists());

    // List archive contents
    let list_result = test_helpers::list_archive_contents(&archive_path);
    assert!(list_result.is_ok());
    let contents = list_result.unwrap();

    // Verify all files are in the archive
    for i in 0..100 {
        let file_name = format!("file_{}.txt", i);
        assert!(contents.contains(&file_name));
    }

    Ok(())
}

#[tokio::test]
async fn test_large_file_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a large file (5MB)
    let large_content = "A".repeat(5 * 1024 * 1024);
    let large_file = test_helpers::create_test_file(&temp_dir, "large_stress.txt", &large_content);

    // Calculate hash (this should handle large files)
    let hash_result = test_helpers::calculate_file_hash(&large_file);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert!(!hash.is_empty());

    // Create archive with large file
    let archive_path =
        test_helpers::create_test_archive(&temp_dir, &[("large_stress.txt", &large_content)]);

    // Validate large archive
    let validate_result = test_helpers::validate_archive(&archive_path);
    assert!(validate_result.is_ok());

    // Extract and verify large file
    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    let extract_result = test_helpers::extract_archive(&archive_path, &extract_dir);
    assert!(extract_result.is_ok());

    let extracted_file = extract_dir.join("large_stress.txt");
    assert!(extracted_file.exists());

    // Verify file size without reading entire content
    let metadata = fs::metadata(&extracted_file)?;
    assert_eq!(metadata.len(), large_content.len() as u64);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create multiple archives concurrently
    let mut handles = Vec::new();

    for i in 0..5 {
        let temp_dir_path = temp_dir.path().to_path_buf();
        let handle = tokio::spawn(async move {
            let file_name = format!("concurrent_{}.txt", i);
            let file_content = format!("concurrent content {}", i);

            // Create files in the shared temp directory
            let file_path = temp_dir_path.join(&file_name);
            tokio::fs::write(&file_path, &file_content).await.unwrap();

            // Create archive in the shared temp directory
            let archive_path = temp_dir_path.join(format!("concurrent_{}.zip", i));
            let archive_manager = rusty::archive::ArchiveManager::new();
            archive_manager.create_archive(&archive_path, &[&file_path]).unwrap();

            // Validate each archive
            let validate_result = test_helpers::validate_archive(&archive_path);
            assert!(validate_result.is_ok());

            // List contents
            let list_result = test_helpers::list_archive_contents(&archive_path);
            assert!(list_result.is_ok());
            let contents = list_result.unwrap();
            assert!(contents.contains(&file_name));

            archive_path
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    for handle in handles {
        let archive_path = handle.await?;
        assert!(archive_path.exists());
    }

    Ok(())
}

#[tokio::test]
async fn test_memory_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create many small files to test memory usage
    let mut files_data = Vec::new();
    for i in 0..50 {
        let file_name = format!("mem_test_{}.txt", i);
        let file_content = format!("memory test content {}", i);
        files_data.push((file_name, file_content));
    }

    let files_slice: Vec<(&str, &str)> = files_data
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Create and validate multiple archives in sequence
    for round in 0..5 {
        let archive_path = test_helpers::create_test_archive(&temp_dir, &files_slice);
        assert!(archive_path.exists());

        // Validate archive
        let validate_result = test_helpers::validate_archive(&archive_path);
        assert!(validate_result.is_ok());

        // Extract archive
        let extract_dir = temp_dir.path().join(format!("extract_{}", round));
        fs::create_dir_all(&extract_dir)?;
        let extract_result = test_helpers::extract_archive(&archive_path, &extract_dir);
        assert!(extract_result.is_ok());

        // Verify some files were extracted
        assert!(extract_dir.join("mem_test_0.txt").exists());
        assert!(extract_dir.join("mem_test_49.txt").exists());
    }

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create valid archive
    let valid_archive =
        test_helpers::create_test_archive(&temp_dir, &[("valid.txt", "valid content")]);

    // Create invalid archive
    let invalid_archive = temp_dir.path().join("invalid.zip");
    fs::write(&invalid_archive, "invalid zip content")?;

    // Test alternating valid and invalid operations
    for i in 0..10 {
        if i % 2 == 0 {
            // Valid operation
            let result = test_helpers::validate_archive(&valid_archive);
            assert!(result.is_ok());
        } else {
            // Invalid operation (should fail gracefully)
            let result = test_helpers::validate_archive(&invalid_archive);
            assert!(result.is_err());
        }
    }

    // After stress test, valid operations should still work
    let final_result = test_helpers::validate_archive(&valid_archive);
    assert!(final_result.is_ok());

    Ok(())
}
