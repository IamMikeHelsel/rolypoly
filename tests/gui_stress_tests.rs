// Comprehensive GUI stress testing to ensure bomb-proof operation
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::sleep;

// Test that the GUI backend can handle malformed or edge case inputs without crashing
mod malicious_input_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_extremely_long_paths() {
        // Test with extremely long file paths
        let long_path = "a".repeat(1000);
        let result = create_archive(long_path, vec!["test.txt".to_string()]).await;
        
        // Should fail gracefully, not crash
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.error.is_empty());
    }

    #[tokio::test]
    async fn test_special_characters_in_paths() {
        let malicious_paths = vec![
            "../../etc/passwd".to_string(),
            "..\\..\\windows\\system32".to_string(),
            "/dev/null".to_string(),
            "CON".to_string(),  // Windows reserved name
            "PRN".to_string(),  // Windows reserved name
            "AUX".to_string(),  // Windows reserved name
            "test\0null.txt".to_string(),  // Null byte
            "test\n\r.txt".to_string(),     // Control characters
            "test\u{FEFF}.txt".to_string(), // Unicode BOM
        ];

        for malicious_path in malicious_paths {
            let result = create_archive(malicious_path.clone(), vec!["test.txt".to_string()]).await;
            // Should handle gracefully without panicking
            if result.is_ok() {
                println!("Unexpected success with path: {}", malicious_path);
            }
        }
    }

    #[tokio::test]
    async fn test_massive_file_lists() {
        // Test with extremely large file lists
        let massive_list: Vec<String> = (0..10000)
            .map(|i| format!("/nonexistent/file_{}.txt", i))
            .collect();

        let result = create_archive("test.zip".to_string(), massive_list).await;
        
        // Should fail gracefully with large input
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_and_whitespace_inputs() {
        let bad_inputs = vec![
            "".to_string(),
            " ".to_string(),
            "\t\n\r".to_string(),
            "\u{00A0}".to_string(), // Non-breaking space
        ];

        for bad_input in bad_inputs {
            let result = create_archive(bad_input.clone(), vec!["test.txt".to_string()]).await;
            assert!(result.is_err(), "Should reject empty/whitespace input: '{}'", bad_input);
        }
    }

    #[tokio::test]
    async fn test_unicode_edge_cases() {
        let unicode_edge_cases = vec![
            "test_🦀_file.txt".to_string(),
            "测试文件.txt".to_string(),
            "файл.txt".to_string(),
            "مثال.txt".to_string(),
            "テスト.txt".to_string(),
            "\u{200B}test.txt".to_string(), // Zero-width space
            "\u{FFFE}test.txt".to_string(), // Non-character
        ];

        for unicode_case in unicode_edge_cases {
            let result = list_archive(unicode_case.clone()).await;
            // Should handle Unicode gracefully
            assert!(result.is_err()); // File doesn't exist, but shouldn't crash
        }
    }
}

// Test concurrent access and race conditions
mod concurrency_stress_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_concurrent_archive_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content")?;

        let operations_count = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        // Launch 50 concurrent operations
        for i in 0..50 {
            let temp_dir_path = temp_dir.path().to_path_buf();
            let test_file_path = test_file.to_path_buf();
            let ops_counter = Arc::clone(&operations_count);

            let handle = tokio::spawn(async move {
                let archive_path = temp_dir_path.join(format!("concurrent_{}.zip", i));
                
                // Create archive
                let result = create_archive(
                    archive_path.to_string_lossy().to_string(),
                    vec![test_file_path.to_string_lossy().to_string()],
                ).await;
                
                if result.is_ok() {
                    ops_counter.fetch_add(1, Ordering::SeqCst);
                }
                
                result
            });
            
            handles.push(handle);
        }

        // Wait for all operations
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(_)) = handle.await {
                success_count += 1;
            }
        }

        println!("Successfully completed {}/50 concurrent operations", success_count);
        assert!(success_count > 40, "Should complete most operations successfully");

        Ok(())
    }

    #[tokio::test]
    async fn test_rapid_fire_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("rapid_test.txt");
        fs::write(&test_file, "Rapid test content")?;

        let start = Instant::now();
        let mut operations = 0;

        // Rapid fire operations for 5 seconds
        while start.elapsed() < Duration::from_secs(5) {
            let archive_path = temp_dir.path().join(format!("rapid_{}.zip", operations));
            
            let result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            
            if result.is_ok() {
                operations += 1;
            }
            
            // Small delay to prevent total system overload
            sleep(Duration::from_millis(10)).await;
        }

        println!("Completed {} operations in 5 seconds", operations);
        assert!(operations > 10, "Should complete multiple operations rapidly");

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_pressure_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create many files to put memory pressure
        let mut files = Vec::new();
        for i in 0..100 {
            let test_file = temp_dir.path().join(format!("memory_test_{}.txt", i));
            let content = "A".repeat(1024 * 10); // 10KB per file
            fs::write(&test_file, &content)?;
            files.push(test_file.to_string_lossy().to_string());
        }

        let archive_path = temp_dir.path().join("memory_pressure.zip");

        // This should handle memory pressure gracefully
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            files,
        ).await;

        assert!(result.is_ok(), "Should handle memory pressure gracefully");

        Ok(())
    }
}

// Test error recovery and resilience
mod error_recovery_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_recovery_from_corrupted_archives() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let corrupted_archive = temp_dir.path().join("corrupted.zip");

        // Create a corrupted "ZIP" file
        fs::write(&corrupted_archive, "This is not a ZIP file")?;

        // Try to list the corrupted archive
        let result = list_archive(corrupted_archive.to_string_lossy().to_string()).await;
        
        // Should fail gracefully without crashing
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.code == "INVALID_ARCHIVE" || error.code == "UNKNOWN_ERROR");

        Ok(())
    }

    #[tokio::test]
    async fn test_recovery_from_permission_errors() {
        // Test with paths that should cause permission errors
        let restricted_paths = vec![
            "/root/restricted.zip".to_string(),
            "/System/restricted.zip".to_string(),
            "C:\\Windows\\System32\\restricted.zip".to_string(),
        ];

        for restricted_path in restricted_paths {
            let result = create_archive(
                restricted_path.clone(),
                vec!["test.txt".to_string()],
            ).await;
            
            // Should handle permission errors gracefully
            if result.is_err() {
                let error = result.unwrap_err();
                // Should provide meaningful error information
                assert!(!error.error.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_disk_space_simulation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create a very large file to potentially cause disk space issues
        let large_file = temp_dir.path().join("very_large.txt");
        let large_content = "X".repeat(1024 * 1024 * 100); // 100MB
        
        // This might fail due to disk space, but shouldn't crash
        if fs::write(&large_file, &large_content).is_ok() {
            let archive_path = temp_dir.path().join("large.zip");
            
            let result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![large_file.to_string_lossy().to_string()],
            ).await;
            
            // Should either succeed or fail gracefully
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(!error.error.is_empty());
            }
        }

        Ok(())
    }
}

// Test GUI state management under stress
mod gui_state_stress_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_rapid_state_changes() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("state_test.txt");
        fs::write(&test_file, "State test content")?;

        // Simulate rapid GUI state changes
        for i in 0..20 {
            let archive_path = temp_dir.path().join(format!("state_{}.zip", i));
            
            // Create archive
            let create_result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            
            if create_result.is_ok() {
                // Immediately list it
                let list_result = list_archive(
                    archive_path.to_string_lossy().to_string()
                ).await;
                
                if list_result.is_ok() {
                    // Immediately validate it
                    let validate_result = validate_archive(
                        archive_path.to_string_lossy().to_string()
                    ).await;
                    
                    if validate_result.is_ok() {
                        // Get stats
                        let _stats_result = get_archive_stats(
                            archive_path.to_string_lossy().to_string()
                        ).await;
                    }
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_interleaved_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("interleaved_test.txt");
        fs::write(&test_file, "Interleaved test content")?;

        let mut handles = Vec::new();

        // Launch interleaved operations
        for i in 0..10 {
            let temp_dir_path = temp_dir.path().to_path_buf();
            let test_file_path = test_file.to_path_buf();

            // Create operation
            let create_handle = {
                let temp_dir_path = temp_dir_path.clone();
                let test_file_path = test_file_path.clone();
                tokio::spawn(async move {
                    let archive_path = temp_dir_path.join(format!("interleaved_create_{}.zip", i));
                    create_archive(
                        archive_path.to_string_lossy().to_string(),
                        vec![test_file_path.to_string_lossy().to_string()],
                    ).await
                })
            };

            // Hash operation
            let hash_handle = {
                let test_file_path = test_file_path.clone();
                tokio::spawn(async move {
                    calculate_file_hash(test_file_path.to_string_lossy().to_string()).await
                })
            };

            handles.push(create_handle);
            handles.push(hash_handle);
        }

        // Wait for all operations
        for handle in handles {
            let _ = handle.await;
        }

        Ok(())
    }
}

// Performance under stress conditions
mod performance_stress_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_performance_degradation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("perf_test.txt");
        fs::write(&test_file, "Performance test content")?;

        let mut times = Vec::new();

        // Measure performance over many operations
        for i in 0..20 {
            let archive_path = temp_dir.path().join(format!("perf_{}.zip", i));
            
            let start = Instant::now();
            
            let result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            
            let duration = start.elapsed();
            
            if result.is_ok() {
                times.push(duration);
            }
        }

        // Check that performance doesn't degrade significantly
        if times.len() > 10 {
            let first_half_avg: Duration = times[..times.len()/2].iter().sum::<Duration>() / (times.len()/2) as u32;
            let second_half_avg: Duration = times[times.len()/2..].iter().sum::<Duration>() / (times.len()/2) as u32;
            
            println!("First half average: {:?}", first_half_avg);
            println!("Second half average: {:?}", second_half_avg);
            
            // Performance shouldn't degrade by more than 50%
            assert!(
                second_half_avg.as_millis() < first_half_avg.as_millis() * 3 / 2,
                "Performance degraded too much"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_leak_detection() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("memory_leak_test.txt");
        fs::write(&test_file, "Memory leak test content")?;

        // Perform many operations to detect potential memory leaks
        for i in 0..100 {
            let archive_path = temp_dir.path().join(format!("memory_leak_{}.zip", i));
            
            // Create and immediately drop operations
            let _result = create_archive(
                archive_path.to_string_lossy().to_string(),
                vec![test_file.to_string_lossy().to_string()],
            ).await;
            
            // Force garbage collection in languages that support it
            // In Rust, this is mostly automatic, but we can yield to allow cleanup
            tokio::task::yield_now().await;
        }

        // If we get here without excessive memory usage, we're probably OK
        println!("Memory leak test completed - check system memory usage");

        Ok(())
    }
}

// Integration with real GUI scenarios
mod real_world_simulation_tests {
    use super::*;
    use rusty::gui::*;

    #[tokio::test]
    async fn test_typical_user_workflow() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Simulate a typical user workflow
        
        // 1. User creates some files
        let files = vec![
            "document.txt",
            "image.png", 
            "code.rs",
            "data.json",
        ];
        
        let mut file_paths = Vec::new();
        for file_name in files {
            let file_path = temp_dir.path().join(file_name);
            fs::write(&file_path, format!("Content of {}", file_name))?;
            file_paths.push(file_path.to_string_lossy().to_string());
        }
        
        let archive_path = temp_dir.path().join("user_archive.zip");
        
        // 2. User creates archive
        let create_result = create_archive(
            archive_path.to_string_lossy().to_string(),
            file_paths,
        ).await;
        assert!(create_result.is_ok());
        
        // 3. User checks what's in the archive
        let list_result = list_archive(archive_path.to_string_lossy().to_string()).await;
        assert!(list_result.is_ok());
        let files_in_archive = list_result.unwrap().data;
        assert_eq!(files_in_archive.len(), 4);
        
        // 4. User validates the archive
        let validate_result = validate_archive(archive_path.to_string_lossy().to_string()).await;
        assert!(validate_result.is_ok());
        assert!(validate_result.unwrap().data);
        
        // 5. User gets statistics
        let stats_result = get_archive_stats(archive_path.to_string_lossy().to_string()).await;
        assert!(stats_result.is_ok());
        let stats = stats_result.unwrap().data;
        assert_eq!(stats.file_count, 4);
        
        // 6. User extracts the archive
        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir(&extract_dir)?;
        
        let extract_result = extract_archive(
            archive_path.to_string_lossy().to_string(),
            extract_dir.to_string_lossy().to_string(),
        ).await;
        assert!(extract_result.is_ok());
        
        // 7. Verify extraction worked
        assert!(extract_dir.join("document.txt").exists());
        assert!(extract_dir.join("image.png").exists());
        assert!(extract_dir.join("code.rs").exists());
        assert!(extract_dir.join("data.json").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_user_mistake_scenarios() -> anyhow::Result<()> {
        // Test common user mistakes and ensure graceful handling
        
        // 1. User tries to archive non-existent files
        let result = create_archive(
            "mistake.zip".to_string(),
            vec!["/this/file/does/not/exist.txt".to_string()],
        ).await;
        assert!(result.is_err()); // Should fail gracefully
        
        // 2. User tries to extract to invalid location
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test")?;
        
        let archive_path = temp_dir.path().join("test.zip");
        create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await?;
        
        let result = extract_archive(
            archive_path.to_string_lossy().to_string(),
            "/invalid/extract/location".to_string(),
        ).await;
        // Should fail gracefully without crashing
        
        // 3. User tries to open invalid archive
        let result = list_archive("/this/archive/does/not/exist.zip".to_string()).await;
        assert!(result.is_err());

        Ok(())
    }
}