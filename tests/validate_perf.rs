use rolypoly::archive::ArchiveManager;
use rolypoly::operations::OperationManager;
use rolypoly::state::{AppStateManager, Operation};
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

#[tokio::test]
async fn test_validate_performance() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");

    // Create a test file and archive
    fs::write(&test_file, "Hello, World!").unwrap();
    let archive_manager = Arc::new(ArchiveManager::new());
    archive_manager.create_archive(&archive_path, &[&test_file]).unwrap();

    let state_manager = Arc::new(AppStateManager::new());
    let op_manager = OperationManager::new(archive_manager.clone(), state_manager.clone());

    let operation = Operation::ValidateArchive {
        archive: archive_path.clone(),
    };

    let start = Instant::now();
    let result = op_manager.execute_operation(operation).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());

    println!("Validation took {:.2?}", elapsed);

    // With the sleep loop (100 * 5ms = 500ms), this should be > 500ms
    // After optimization, it should be much faster (e.g., < 100ms for such a small file)
}
