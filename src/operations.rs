use crate::archive::ArchiveManager;
use crate::state::{AppEvent, AppStateManager, Operation, OperationResult};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

pub struct OperationManager {
    archive_manager: Arc<ArchiveManager>,
    state_manager: Arc<AppStateManager>,
    operation_semaphore: Arc<Semaphore>,
    active_operations: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>>,
}

impl OperationManager {
    pub fn new(archive_manager: Arc<ArchiveManager>, state_manager: Arc<AppStateManager>) -> Self {
        Self {
            archive_manager,
            state_manager,
            operation_semaphore: Arc::new(Semaphore::new(3)), // Max 3 concurrent operations
            active_operations: Arc::new(tokio::sync::Mutex::new(Vec::new())),
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

        // Run in blocking task to avoid blocking the async runtime
        let result = tokio::task::spawn_blocking(move || {
            // Simulate progress updates
            for i in 0..=100 {
                let progress = i as f64 / 100.0;
                state_manager.emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                if i < 100 {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }

            // Perform actual archive creation
            let file_refs: Vec<&PathBuf> = files.iter().collect();
            archive_manager.create_archive(&output, &file_refs)
        })
        .await
        .map_err(|e| e.to_string())?;

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

        let result = tokio::task::spawn_blocking(move || {
            // Simulate progress updates
            for i in 0..=100 {
                let progress = i as f64 / 100.0;
                state_manager.emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                if i < 100 {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }

            // Perform actual extraction
            archive_manager.extract_archive(&archive, &output)
        })
        .await
        .map_err(|e| e.to_string())?;

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

        let result = tokio::task::spawn_blocking(move || {
            // Simulate progress updates
            for i in 0..=100 {
                let progress = i as f64 / 100.0;
                state_manager.emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                if i < 100 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }

            // Perform actual validation
            archive_manager.validate_archive(&archive)
        })
        .await
        .map_err(|e| e.to_string())?;

        result.map(OperationResult::ArchiveValidated).map_err(|e| e.to_string())
    }

    async fn calculate_hash_with_progress(&self, file: PathBuf) -> Result<OperationResult, String> {
        let archive_manager = self.archive_manager.clone();
        let state_manager = self.state_manager.clone();
        let operation = Operation::CalculateHash { file: file.clone() };

        let result = tokio::task::spawn_blocking(move || {
            // Simulate progress updates
            for i in 0..=100 {
                let progress = i as f64 / 100.0;
                state_manager.emit_event(AppEvent::OperationProgress(operation.clone(), progress));

                if i < 100 {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }

            // Perform actual hash calculation
            archive_manager.calculate_file_hash(&file)
        })
        .await
        .map_err(|e| e.to_string())?;

        result.map(OperationResult::HashCalculated).map_err(|e| e.to_string())
    }

    pub async fn cancel_all_operations(&self) {
        let mut operations = self.active_operations.lock().await;
        for handle in operations.drain(..) {
            handle.abort();
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
    async fn test_create_archive_success() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");
        std::fs::write(&test_file, "Hello, World!").unwrap();

        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let op_manager = OperationManager::new(archive_manager, state_manager.clone());

        let operation = Operation::CreateArchive {
            output: archive_path.clone(),
            files: vec![test_file.clone()],
        };

        let mut receiver = state_manager.subscribe();

        let result = op_manager.execute_operation(operation.clone()).await;
        assert!(result.is_ok());

        if let Ok(OperationResult::ArchiveCreated(path)) = result {
            assert_eq!(path, archive_path);
        } else {
            panic!("Expected ArchiveCreated result");
        }

        assert!(archive_path.exists());

        // Check for events
        let mut completed = false;
        loop {
            match receiver.try_recv() {
                Ok(AppEvent::OperationCompleted(op, _)) if op == operation => {
                    completed = true;
                    break;
                }
                Ok(_) => continue,
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
            }
        }
        assert!(completed, "OperationCompleted event not received");
    }

    #[tokio::test]
    async fn test_create_archive_failure() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        // Use a directory as output path to force failure
        let bad_output_path = temp_dir.path().join("bad_output");
        std::fs::create_dir(&bad_output_path).unwrap();

        std::fs::write(&test_file, "Hello, World!").unwrap();

        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let op_manager = OperationManager::new(archive_manager, state_manager.clone());

        let operation = Operation::CreateArchive {
            output: bad_output_path.clone(),
            files: vec![test_file.clone()],
        };

        let mut receiver = state_manager.subscribe();

        let result = op_manager.execute_operation(operation.clone()).await;
        assert!(result.is_err());

        // Check for events
        let mut failed = false;
        loop {
            match receiver.try_recv() {
                Ok(AppEvent::OperationFailed(op, _)) if op == operation => {
                    failed = true;
                    break;
                }
                Ok(_) => continue,
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
            }
        }
        assert!(failed, "OperationFailed event not received");
    }
}
