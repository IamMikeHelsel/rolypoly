use crate::archive::ArchiveManager;
use crate::state::{AppEvent, AppStateManager, Operation, OperationResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct OperationManager {
    archive_manager: Arc<ArchiveManager>,
    state_manager: Arc<AppStateManager>,
    operation_semaphore: Arc<Semaphore>,
    active_operations: Arc<tokio::sync::Mutex<HashMap<u64, Arc<AtomicBool>>>>,
    next_op_id: AtomicU64,
}

impl OperationManager {
    pub fn new(archive_manager: Arc<ArchiveManager>, state_manager: Arc<AppStateManager>) -> Self {
        Self {
            archive_manager,
            state_manager,
            operation_semaphore: Arc::new(Semaphore::new(3)), // Max 3 concurrent operations
            active_operations: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            next_op_id: AtomicU64::new(0),
        }
    }

    pub async fn execute_operation(&self, operation: Operation) -> Result<OperationResult, String> {
        // Acquire semaphore permit for concurrency control
        let _permit = self.operation_semaphore.acquire().await.map_err(|e| e.to_string())?;

        self.state_manager.emit_event(AppEvent::OperationStarted(operation.clone()));

        let result = match operation.clone() {
            Operation::CreateArchive { output, files } => {
                self.create_archive_with_progress(output, files).await
            }
            Operation::ExtractArchive { archive, output } => {
                self.extract_archive_with_progress(archive, output).await
            }
            Operation::ValidateArchive { archive } => {
                self.validate_archive_with_progress(archive).await
            }
            Operation::CalculateHash { file } => self.calculate_hash_with_progress(file).await,
        };

        match &result {
            Ok(op_result) => {
                self.state_manager
                    .emit_event(AppEvent::OperationCompleted(operation, op_result.clone()));
            }
            Err(error) => {
                self.state_manager
                    .emit_event(AppEvent::OperationFailed(operation, error.clone()));
            }
        }

        result
    }

    async fn run_cancellable<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(Arc<AtomicBool>) -> T + Send + 'static,
        T: Send + 'static,
    {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let id = self.next_op_id.fetch_add(1, Ordering::Relaxed);

        {
            let mut ops = self.active_operations.lock().await;
            ops.insert(id, flag);
        }

        let handle = tokio::task::spawn_blocking(move || f(flag_clone));
        let result = handle.await;

        {
            let mut ops = self.active_operations.lock().await;
            ops.remove(&id);
        }

        result.map_err(|e| e.to_string())
    }

    async fn create_archive_with_progress(
        &self,
        output: PathBuf,
        files: Vec<PathBuf>,
    ) -> Result<OperationResult, String> {
        let archive_manager = self.archive_manager.clone();
        let state_manager = self.state_manager.clone();
        let operation = Operation::CreateArchive {
            output: output.clone(),
            files: files.clone(),
        };
        let output_clone = output.clone();

        let result = self
            .run_cancellable(move |cancel_flag| {
                // Simulate progress updates
                for i in 0..=100 {
                    if cancel_flag.load(Ordering::Relaxed) {
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }

                    let progress = i as f64 / 100.0;
                    state_manager
                        .emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                    if i < 100 {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }

                if cancel_flag.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Operation cancelled"));
                }

                // Perform actual archive creation
                let file_refs: Vec<&PathBuf> = files.iter().collect();
                archive_manager.create_archive(&output, &file_refs)
            })
            .await?;

        result
            .map(|_| OperationResult::ArchiveCreated(output_clone))
            .map_err(|e| e.to_string())
    }

    async fn extract_archive_with_progress(
        &self,
        archive: PathBuf,
        output: PathBuf,
    ) -> Result<OperationResult, String> {
        let archive_manager = self.archive_manager.clone();
        let state_manager = self.state_manager.clone();
        let operation = Operation::ExtractArchive {
            archive: archive.clone(),
            output: output.clone(),
        };
        let output_clone = output.clone();

        let result = self
            .run_cancellable(move |cancel_flag| {
                // Simulate progress updates
                for i in 0..=100 {
                    if cancel_flag.load(Ordering::Relaxed) {
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }

                    let progress = i as f64 / 100.0;
                    state_manager
                        .emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                    if i < 100 {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }

                if cancel_flag.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Operation cancelled"));
                }

                // Perform actual extraction
                archive_manager.extract_archive(&archive, &output)
            })
            .await?;

        result
            .map(|_| OperationResult::ArchiveExtracted(output_clone))
            .map_err(|e| e.to_string())
    }

    async fn validate_archive_with_progress(
        &self,
        archive: PathBuf,
    ) -> Result<OperationResult, String> {
        let archive_manager = self.archive_manager.clone();
        let state_manager = self.state_manager.clone();
        let operation = Operation::ValidateArchive {
            archive: archive.clone(),
        };

        let result = self
            .run_cancellable(move |cancel_flag| {
                // Simulate progress updates
                for i in 0..=100 {
                    if cancel_flag.load(Ordering::Relaxed) {
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }

                    let progress = i as f64 / 100.0;
                    state_manager
                        .emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                    if i < 100 {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                }

                if cancel_flag.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Operation cancelled"));
                }

                // Perform actual validation
                archive_manager.validate_archive(&archive)
            })
            .await?;

        result
            .map(OperationResult::ArchiveValidated)
            .map_err(|e| e.to_string())
    }

    async fn calculate_hash_with_progress(&self, file: PathBuf) -> Result<OperationResult, String> {
        let archive_manager = self.archive_manager.clone();
        let state_manager = self.state_manager.clone();
        let operation = Operation::CalculateHash { file: file.clone() };

        let result = self
            .run_cancellable(move |cancel_flag| {
                // Simulate progress updates
                for i in 0..=100 {
                    if cancel_flag.load(Ordering::Relaxed) {
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }

                    let progress = i as f64 / 100.0;
                    state_manager
                        .emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                    if i < 100 {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                }

                if cancel_flag.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Operation cancelled"));
                }

                // Perform actual hash calculation
                archive_manager.calculate_file_hash(&file)
            })
            .await?;

        result
            .map(OperationResult::HashCalculated)
            .map_err(|e| e.to_string())
    }

    pub async fn cancel_all_operations(&self) {
        let operations = self.active_operations.lock().await;
        for flag in operations.values() {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub async fn get_active_operation_count(&self) -> usize {
        let operations = self.active_operations.lock().await;
        operations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppStateManager;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_operation_manager_creation() {
        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let op_manager = OperationManager::new(archive_manager, state_manager);

        assert_eq!(op_manager.get_active_operation_count().await, 0);
    }

    #[tokio::test]
    async fn test_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, World!").unwrap();

        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let op_manager = OperationManager::new(archive_manager, state_manager);

        let operation = Operation::CalculateHash {
            file: test_file.clone(),
        };

        let result = op_manager.execute_operation(operation).await;
        assert!(result.is_ok());

        if let Ok(OperationResult::HashCalculated(hash)) = result {
            assert!(!hash.is_empty());
            assert_eq!(hash.len(), 64); // SHA256 hash length
        } else {
            panic!("Expected HashCalculated result");
        }
    }

    #[tokio::test]
    async fn test_cancel_operation() {
        // Setup
        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let op_manager = Arc::new(OperationManager::new(archive_manager, state_manager));

        // Create a dummy file for hash calculation
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, World!").unwrap();

        let operation = Operation::CalculateHash {
            file: test_file.clone(),
        };

        // Clone op_manager for the task
        let op_manager_clone = op_manager.clone();

        // Spawn operation in a separate task
        let handle = tokio::spawn(async move { op_manager_clone.execute_operation(operation).await });

        // Wait a bit to let the operation start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify operation is active
        assert_eq!(op_manager.get_active_operation_count().await, 1);

        // Cancel operations
        op_manager.cancel_all_operations().await;

        // Await result
        let result = handle.await.unwrap();

        // Check if it was cancelled
        match result {
            Ok(_) => panic!("Operation completed successfully but should have been cancelled"),
            Err(e) => {
                // Expected error message should indicate cancellation
                assert!(e.contains("cancelled") || e.contains("Cancelled"), "Error message was: {}", e);
            }
        }
    }
}
