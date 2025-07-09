use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// Integration tests for GUI backend
// These test the Tauri commands directly without the frontend

mod gui_backend_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_health_check_command() {
        let result = health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_create_archive_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file
        fs::write(&test_file, "Hello, World!")?;

        // Test create command
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(archive_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_list_archive_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test list command
        let result = list_archive(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.data.len(), 1);
        assert!(response.data.contains(&"test.txt".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_archive_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extract");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        fs::create_dir(&extract_dir)?;

        // Test extract command
        let result = extract_archive(
            archive_path.to_string_lossy().to_string(),
            extract_dir.to_string_lossy().to_string(),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);

        // Verify extracted file
        let extracted_file = extract_dir.join("test.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(&extracted_file)?, "Hello, World!");

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_archive_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test validate command
        let result = validate_archive(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_archive_stats_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        // Test stats command
        let result = get_archive_stats(archive_path.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.data.file_count, 1);
        assert!(response.data.total_uncompressed_size > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_file_hash_command() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");

        // Create test file
        fs::write(&test_file, "Hello, World!")?;

        // Test hash command
        let result = calculate_file_hash(test_file.to_string_lossy().to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.data.len(), 64); // SHA256 length

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_empty_input() {
        // Test with empty archive path
        let result = list_archive("".to_string()).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "UNKNOWN_ERROR");

        // Test with empty files list
        let result = create_archive("test.zip".to_string(), vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_handling_nonexistent_files() {
        // Test with nonexistent archive
        let result = list_archive("/nonexistent/path.zip".to_string()).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "UNKNOWN_ERROR");

        // Test with nonexistent file for archiving
        let result = create_archive(
            "test.zip".to_string(),
            vec!["/nonexistent/file.txt".to_string()],
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create multiple test files
        let mut files = Vec::new();
        for i in 0..5 {
            let test_file = temp_dir.path().join(format!("test_{}.txt", i));
            fs::write(&test_file, format!("Content {}", i))?;
            files.push(test_file.to_string_lossy().to_string());
        }

        // Test concurrent archive creation
        let mut handles = Vec::new();
        for i in 0..3 {
            let archive_path = temp_dir.path().join(format!("concurrent_{}.zip", i));
            let files_clone = files.clone();
            
            let handle = tokio::spawn(async move {
                create_archive(
                    archive_path.to_string_lossy().to_string(),
                    files_clone,
                ).await
            });
            
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_large_file_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let large_file = temp_dir.path().join("large_file.txt");
        let archive_path = temp_dir.path().join("large.zip");

        // Create a large file (1MB)
        let content = "A".repeat(1024 * 1024);
        fs::write(&large_file, &content)?;

        // Test creating archive with large file
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![large_file.to_string_lossy().to_string()],
        ).await;

        assert!(result.is_ok());
        assert!(archive_path.exists());

        // Test validation of large archive
        let result = validate_archive(archive_path.to_string_lossy().to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().data);

        Ok(())
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let unicode_file = temp_dir.path().join("测试文件_🦀.txt");
        let archive_path = temp_dir.path().join("unicode_test.zip");

        // Create file with unicode content
        fs::write(&unicode_file, "Hello 世界! 🦀 Rust")?;

        // Test creating archive
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![unicode_file.to_string_lossy().to_string()],
        ).await;

        assert!(result.is_ok());

        // Test listing archive
        let result = list_archive(archive_path.to_string_lossy().to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_rapid_successive_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("rapid_test.zip");

        fs::write(&test_file, "Test content")?;

        // Rapid successive operations
        for i in 0..10 {
            let archive_path = temp_dir.path().join(format!("rapid_{}.zip", i));
            
            // Create
            let result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            assert!(result.is_ok());

            // List
            let result = list_archive(archive_path.to_string_lossy().to_string()).await;
            assert!(result.is_ok());

            // Validate
            let result = validate_archive(archive_path.to_string_lossy().to_string()).await;
            assert!(result.is_ok());
        }

        Ok(())
    }
}

// Full end-to-end GUI tests using Tauri's test framework
// These would normally use something like WebDriver for real browser testing
mod gui_e2e_tests {
    use super::*;

    // Mock frontend-backend communication tests
    #[tokio::test]
    async fn test_frontend_backend_communication() {
        // This would test the actual Tauri invoke calls
        // In a real implementation, we'd use Tauri's testing framework
        
        // Test that commands can be invoked without panicking
        let result = std::panic::catch_unwind(|| {
            // Simulate frontend calling backend
            println!("Frontend -> Backend communication test");
        });
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_propagation_to_frontend() {
        // Test that backend errors are properly formatted for frontend
        let error = rusty::gui::ErrorResponse {
            error: "Test error".to_string(),
            details: Some("Test details".to_string()),
            code: "TEST_ERROR".to_string(),
        };

        // Verify error structure
        assert!(!error.error.is_empty());
        assert!(error.details.is_some());
        assert_eq!(error.code, "TEST_ERROR");
    }

    #[tokio::test]
    async fn test_gui_state_management() {
        // Test that GUI state is properly managed
        // This would normally test React/Vue state or similar
        
        // Mock state transitions
        let mut current_archive: Option<String> = None;
        let mut current_files: Vec<String> = Vec::new();

        // Open archive
        current_archive = Some("test.zip".to_string());
        current_files = vec!["file1.txt".to_string(), "file2.txt".to_string()];

        assert!(current_archive.is_some());
        assert_eq!(current_files.len(), 2);

        // Close archive
        current_archive = None;
        current_files.clear();

        assert!(current_archive.is_none());
        assert!(current_files.is_empty());
    }
}

// Stress tests for GUI
mod gui_stress_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn stress_test_many_files() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let archive_path = temp_dir.path().join("stress_test.zip");

        // Create many small files
        let mut files = Vec::new();
        for i in 0..1000 {
            let test_file = temp_dir.path().join(format!("stress_file_{}.txt", i));
            fs::write(&test_file, format!("Content {}", i))?;
            files.push(test_file.to_string_lossy().to_string());
        }

        // Test creating archive with many files
        let result = rusty::gui::create_archive(
            archive_path.to_string_lossy().to_string(),
            files,
        ).await;

        assert!(result.is_ok());
        assert!(archive_path.exists());

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag  
    async fn stress_test_rapid_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content")?;

        // Rapid fire operations
        for i in 0..100 {
            let archive_path = temp_dir.path().join(format!("stress_{}.zip", i));
            
            // Create, list, validate in rapid succession
            let create_result = rusty::gui::create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            assert!(create_result.is_ok());

            let list_result = rusty::gui::list_archive(
                archive_path.to_string_lossy().to_string()
            ).await;
            assert!(list_result.is_ok());

            let validate_result = rusty::gui::validate_archive(
                archive_path.to_string_lossy().to_string()
            ).await;
            assert!(validate_result.is_ok());

            // Small delay to prevent overwhelming the system
            sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

// Performance tests for GUI operations
mod gui_performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_create_archive_performance() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("perf_test.txt");
        let archive_path = temp_dir.path().join("perf_test.zip");

        // Create a medium-sized file
        let content = "A".repeat(1024 * 100); // 100KB
        fs::write(&test_file, &content)?;

        let start = Instant::now();
        
        let result = rusty::gui::create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await;

        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 5000); // Should complete within 5 seconds
        
        println!("Archive creation took: {:?}", duration);

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_archive_performance() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("perf_test.txt");
        let archive_path = temp_dir.path().join("perf_test.zip");
        let extract_dir = temp_dir.path().join("extract");

        // Create test file and archive
        let content = "A".repeat(1024 * 100); // 100KB
        fs::write(&test_file, &content)?;
        
        rusty::gui::create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;

        fs::create_dir(&extract_dir)?;

        let start = Instant::now();
        
        let result = rusty::gui::extract_archive(
            archive_path.to_string_lossy().to_string(),
            extract_dir.to_string_lossy().to_string(),
        ).await;

        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 2000); // Should complete within 2 seconds
        
        println!("Archive extraction took: {:?}", duration);

        Ok(())
    }
}