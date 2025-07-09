// Comprehensive GUI component testing
use std::fs;
use tempfile::TempDir;

// Test all GUI backend commands systematically
mod gui_backend_component_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_health_check_component() {
        println!("🔧 Testing health_check component...");
        
        let result = health_check().await;
        assert!(result.is_ok(), "Health check should succeed");
        
        let response = result.unwrap();
        assert!(response.success, "Health check should return success=true");
        assert!(!response.data.is_empty(), "Health check should return non-empty message");
        
        println!("✅ health_check component working");
    }

    #[tokio::test]
    async fn test_create_archive_component() -> anyhow::Result<()> {
        println!("🔧 Testing create_archive component...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("component_test.txt");
        let archive_path = temp_dir.path().join("component_test.zip");

        // Create test file
        fs::write(&test_file, "Component test content")?;

        // Test the component
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await;

        assert!(result.is_ok(), "create_archive component should succeed");
        let response = result.unwrap();
        assert!(response.success, "create_archive should return success=true");
        assert!(archive_path.exists(), "Archive file should be created");
        assert!(!response.data.is_empty(), "Should return success message");

        println!("✅ create_archive component working");
        Ok(())
    }

    #[tokio::test]
    async fn test_list_archive_component() -> anyhow::Result<()> {
        println!("🔧 Testing list_archive component...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("list_test.txt");
        let archive_path = temp_dir.path().join("list_test.zip");

        // Create test setup
        fs::write(&test_file, "List test content")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test the component
        let result = list_archive(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok(), "list_archive component should succeed");
        let response = result.unwrap();
        assert!(response.success, "list_archive should return success=true");
        assert_eq!(response.data.len(), 1, "Should list exactly 1 file");
        assert!(response.data.contains(&"list_test.txt".to_string()), "Should contain our test file");

        println!("✅ list_archive component working");
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_archive_component() -> anyhow::Result<()> {
        println!("🔧 Testing validate_archive component...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("validate_test.txt");
        let archive_path = temp_dir.path().join("validate_test.zip");

        // Create test setup
        fs::write(&test_file, "Validate test content")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test the component
        let result = validate_archive(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok(), "validate_archive component should succeed");
        let response = result.unwrap();
        assert!(response.success, "validate_archive should return success=true");
        assert!(response.data, "Archive should be valid");

        println!("✅ validate_archive component working");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_archive_stats_component() -> anyhow::Result<()> {
        println!("🔧 Testing get_archive_stats component...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("stats_test.txt");
        let archive_path = temp_dir.path().join("stats_test.zip");

        // Create test setup
        fs::write(&test_file, "Stats test content with some length")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test the component
        let result = get_archive_stats(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok(), "get_archive_stats component should succeed");
        let response = result.unwrap();
        assert!(response.success, "get_archive_stats should return success=true");
        assert_eq!(response.data.file_count, 1, "Should report 1 file");
        assert!(response.data.total_uncompressed_size > 0, "Should report non-zero uncompressed size");

        println!("✅ get_archive_stats component working");
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_file_hash_component() -> anyhow::Result<()> {
        println!("🔧 Testing calculate_file_hash component...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("hash_test.txt");

        // Create test setup
        fs::write(&test_file, "Hash test content")?;

        // Test the component
        let result = calculate_file_hash(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_ok(), "calculate_file_hash component should succeed");
        let response = result.unwrap();
        assert!(response.success, "calculate_file_hash should return success=true");
        assert_eq!(response.data.len(), 64, "SHA256 hash should be 64 characters");
        assert!(response.data.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex digits");

        println!("✅ calculate_file_hash component working");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_app_info_component() {
        println!("🔧 Testing get_app_info component...");
        
        let result = get_app_info().await;
        assert!(result.is_ok(), "get_app_info component should succeed");
        
        let response = result.unwrap();
        assert!(response.success, "get_app_info should return success=true");
        
        // Check that we get a JSON object with expected fields
        assert!(response.data.is_object(), "App info should be a JSON object");
        let info = response.data.as_object().unwrap();
        assert!(info.contains_key("name"), "Should contain 'name' field");
        assert!(info.contains_key("version"), "Should contain 'version' field");
        
        println!("✅ get_app_info component working");
    }
}

// Test error handling in GUI components
mod gui_error_handling_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_create_archive_error_handling() {
        println!("🔧 Testing create_archive error handling...");
        
        // Test with empty archive path
        let result = create_archive("".to_string(), vec!["test.txt".to_string()]).await;
        assert!(result.is_err(), "Should fail with empty archive path");
        let error = result.unwrap_err();
        assert!(!error.error.is_empty(), "Error message should not be empty");
        assert_eq!(error.code, "UNKNOWN_ERROR", "Should return appropriate error code");

        // Test with empty files list
        let result = create_archive("test.zip".to_string(), vec![]).await;
        assert!(result.is_err(), "Should fail with empty files list");

        // Test with nonexistent file
        let result = create_archive("test.zip".to_string(), vec!["/nonexistent/file.txt".to_string()]).await;
        assert!(result.is_err(), "Should fail with nonexistent file");

        println!("✅ create_archive error handling working");
    }

    #[tokio::test]
    async fn test_list_archive_error_handling() {
        println!("🔧 Testing list_archive error handling...");
        
        // Test with empty path
        let result = list_archive("".to_string()).await;
        assert!(result.is_err(), "Should fail with empty path");

        // Test with nonexistent archive
        let result = list_archive("/nonexistent/archive.zip".to_string()).await;
        assert!(result.is_err(), "Should fail with nonexistent archive");

        // Test with invalid archive (create a text file with .zip extension)
        let temp_dir = TempDir::new().unwrap();
        let fake_archive = temp_dir.path().join("fake.zip");
        fs::write(&fake_archive, "This is not a ZIP file").unwrap();
        
        let result = list_archive(fake_archive.to_string_lossy().to_string()).await;
        assert!(result.is_err(), "Should fail with invalid archive");

        println!("✅ list_archive error handling working");
    }

    #[tokio::test]
    async fn test_validate_archive_error_handling() {
        println!("🔧 Testing validate_archive error handling...");
        
        // Test with empty path
        let result = validate_archive("".to_string()).await;
        assert!(result.is_err(), "Should fail with empty path");

        // Test with nonexistent archive
        let result = validate_archive("/nonexistent/archive.zip".to_string()).await;
        assert!(result.is_err(), "Should fail with nonexistent archive");

        println!("✅ validate_archive error handling working");
    }

    #[tokio::test]
    async fn test_calculate_file_hash_error_handling() {
        println!("🔧 Testing calculate_file_hash error handling...");
        
        // Test with empty path
        let result = calculate_file_hash("".to_string()).await;
        assert!(result.is_err(), "Should fail with empty path");

        // Test with nonexistent file
        let result = calculate_file_hash("/nonexistent/file.txt".to_string()).await;
        assert!(result.is_err(), "Should fail with nonexistent file");

        // Test with directory instead of file
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();
        
        let result = calculate_file_hash(dir_path.to_string_lossy().to_string()).await;
        assert!(result.is_err(), "Should fail when given a directory");

        println!("✅ calculate_file_hash error handling working");
    }
}

// Test fun messages functionality
mod gui_fun_messages_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_fun_error_messages() {
        println!("🔧 Testing fun error messages...");
        
        // Test file not found message
        let result = list_archive("/this/file/does/not/exist.zip".to_string()).await;
        assert!(result.is_err(), "Should fail for nonexistent file");
        let error = result.unwrap_err();
        
        // Check that we get a fun message, not just a boring technical error
        let error_text = &error.error;
        let is_fun_message = error_text.contains("🕵️") || 
                           error_text.contains("👻") || 
                           error_text.contains("🎯") ||
                           error_text.contains("🚀") ||
                           error_text.contains("🎪") ||
                           error_text.contains("📍") ||
                           error_text.contains("🌌") ||
                           error_text.contains("vacation") ||
                           error_text.contains("vanished") ||
                           error_text.contains("hide and seek") ||
                           error_text.contains("circus") ||
                           error_text.contains("escaped") ||
                           error_text.contains("transcended");
        
        assert!(is_fun_message, "Should contain fun error message, got: {}", error_text);
        
        // Test invalid archive message
        let temp_dir = TempDir::new().unwrap();
        let fake_archive = temp_dir.path().join("fake.zip");
        fs::write(&fake_archive, "Not a ZIP file").unwrap();
        
        let result = list_archive(fake_archive.to_string_lossy().to_string()).await;
        assert!(result.is_err(), "Should fail for invalid archive");
        let error = result.unwrap_err();
        
        let error_text = &error.error;
        let is_fun_invalid_message = error_text.contains("🗑️") ||
                                   error_text.contains("🤷") ||
                                   error_text.contains("🎪") ||
                                   error_text.contains("🔧") ||
                                   error_text.contains("🌪️") ||
                                   error_text.contains("🎲") ||
                                   error_text.contains("🦄") ||
                                   error_text.contains("⚡") ||
                                   error_text.contains("🔮") ||
                                   error_text.contains("🔍") ||
                                   error_text.contains("🎭") ||
                                   error_text.contains("🤖") ||
                                   error_text.contains("📦") ||
                                   error_text.contains("🚫") ||
                                   error_text.contains("💭") ||
                                   error_text.contains("identity crisis") ||
                                   error_text.contains("trash") ||
                                   error_text.contains("pretending") ||
                                   error_text.contains("circus") ||
                                   error_text.contains("gremlin") ||
                                   error_text.contains("off-script") ||
                                   error_text.contains("glitch") ||
                                   error_text.contains("dice") ||
                                   error_text.contains("mythical") ||
                                   error_text.contains("plot twist") ||
                                   error_text.contains("oracle") ||
                                   error_text.contains("Reply hazy");
        
        assert!(is_fun_invalid_message, "Should contain fun invalid archive message, got: {}", error_text);

        println!("✅ Fun error messages working");
    }

    #[tokio::test]
    async fn test_fun_success_messages() -> anyhow::Result<()> {
        println!("🔧 Testing fun success messages...");
        
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("success_test.txt");
        let archive_path = temp_dir.path().join("success_test.zip");

        fs::write(&test_file, "Success test content")?;

        // Test creation success message
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        let success_text = &result.data;
        let is_fun_create_message = success_text.contains("🎉") ||
                                  success_text.contains("📦") ||
                                  success_text.contains("✨") ||
                                  success_text.contains("🦀") ||
                                  success_text.contains("🔧") ||
                                  success_text.contains("🎯") ||
                                  success_text.contains("📁") ||
                                  success_text.contains("🚀") ||
                                  success_text.contains("magic") ||
                                  success_text.contains("assembled") ||
                                  success_text.contains("bundled") ||
                                  success_text.contains("Engineered") ||
                                  success_text.contains("awesomeness");

        assert!(is_fun_create_message, "Should contain fun creation message, got: {}", success_text);

        println!("✅ Fun success messages working");
        Ok(())
    }
}

// Test concurrent operations don't interfere with each other
mod gui_concurrency_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_concurrent_gui_operations() -> anyhow::Result<()> {
        println!("🔧 Testing concurrent GUI operations...");
        
        let temp_dir = TempDir::new()?;
        let mut handles = Vec::new();

        // Launch multiple operations concurrently
        for i in 0..5 {
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            let handle = tokio::spawn(async move {
                let test_file = temp_dir_path.join(format!("concurrent_test_{}.txt", i));
                let archive_path = temp_dir_path.join(format!("concurrent_test_{}.zip", i));
                
                // Create test file
                fs::write(&test_file, format!("Concurrent test content {}", i)).unwrap();
                
                // Create archive
                let result = create_archive(
                    archive_path.to_string_lossy().to_string(),
                    vec![test_file.to_string_lossy().to_string()],
                ).await;
                
                result.is_ok()
            });
            
            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap() {
                success_count += 1;
            }
        }

        assert!(success_count >= 4, "At least 4 out of 5 concurrent operations should succeed");

        println!("✅ Concurrent GUI operations working");
        Ok(())
    }
}

// Integration test that combines multiple components
mod gui_integration_workflow_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_complete_gui_workflow() -> anyhow::Result<()> {
        println!("🔧 Testing complete GUI workflow...");
        
        let temp_dir = TempDir::new()?;
        
        // Step 1: Health check
        let health = health_check().await?;
        assert!(health.success, "Health check should pass");
        println!("  ✓ Health check passed");

        // Step 2: Create multiple test files
        let mut test_files = Vec::new();
        for i in 1..=3 {
            let test_file = temp_dir.path().join(format!("workflow_test_{}.txt", i));
            fs::write(&test_file, format!("Workflow test content {}", i))?;
            test_files.push(test_file.to_string_lossy().to_string());
        }
        println!("  ✓ Created test files");

        // Step 3: Create archive
        let archive_path = temp_dir.path().join("workflow_test.zip");
        let create_result = create_archive(
            archive_path.to_string_lossy().to_string(),
            test_files.clone(),
        ).await?;
        assert!(create_result.success, "Archive creation should succeed");
        assert!(archive_path.exists(), "Archive file should exist");
        println!("  ✓ Created archive");

        // Step 4: List archive contents
        let list_result = list_archive(archive_path.to_string_lossy().to_string()).await?;
        assert!(list_result.success, "Archive listing should succeed");
        assert_eq!(list_result.data.len(), 3, "Should list 3 files");
        println!("  ✓ Listed archive contents");

        // Step 5: Validate archive
        let validate_result = validate_archive(archive_path.to_string_lossy().to_string()).await?;
        assert!(validate_result.success, "Archive validation should succeed");
        assert!(validate_result.data, "Archive should be valid");
        println!("  ✓ Validated archive");

        // Step 6: Get archive statistics
        let stats_result = get_archive_stats(archive_path.to_string_lossy().to_string()).await?;
        assert!(stats_result.success, "Archive stats should succeed");
        assert_eq!(stats_result.data.file_count, 3, "Should report 3 files");
        assert!(stats_result.data.total_uncompressed_size > 0, "Should have non-zero size");
        println!("  ✓ Retrieved archive statistics");

        // Step 7: Test hash calculation on one of the original files
        let hash_result = calculate_file_hash(test_files[0].clone()).await?;
        assert!(hash_result.success, "Hash calculation should succeed");
        assert_eq!(hash_result.data.len(), 64, "Hash should be 64 characters");
        println!("  ✓ Calculated file hash");

        // Step 8: Get app info
        let info_result = get_app_info().await?;
        assert!(info_result.success, "App info should succeed");
        println!("  ✓ Retrieved app info");

        println!("✅ Complete GUI workflow test passed");
        Ok(())
    }
}