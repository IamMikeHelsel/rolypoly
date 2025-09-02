use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppEvent {
    FilesAdded(Vec<PathBuf>),
    ArchiveOpened(PathBuf),
    OperationStarted(Operation),
    OperationProgress(Operation, f64),
    OperationCompleted(Operation, OperationResult),
    OperationFailed(Operation, String),
    StateChanged(AppState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    CreateArchive {
        output: PathBuf,
        files: Vec<PathBuf>,
    },
    ExtractArchive {
        archive: PathBuf,
        output: PathBuf,
    },
    ValidateArchive {
        archive: PathBuf,
    },
    CalculateHash {
        file: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub enum OperationResult {
    ArchiveCreated(PathBuf),
    ArchiveExtracted(PathBuf),
    ArchiveValidated(bool),
    HashCalculated(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Empty,
    FilesSelected(Vec<PathBuf>),
    ArchiveLoaded(PathBuf),
    Processing(Operation),
    Error(String),
}

pub struct AppStateManager {
    state: Arc<Mutex<AppState>>,
    event_sender: broadcast::Sender<AppEvent>,
    _event_receiver: broadcast::Receiver<AppEvent>,
}

impl AppStateManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(100);
        Self {
            state: Arc::new(Mutex::new(AppState::Empty)),
            event_sender,
            _event_receiver: event_receiver,
        }
    }

    pub fn get_state(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    pub fn set_state(&self, new_state: AppState) {
        {
            let mut state = self.state.lock().unwrap();
            *state = new_state.clone();
        }
        let _ = self.event_sender.send(AppEvent::StateChanged(new_state));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.event_sender.subscribe()
    }

    pub fn emit_event(&self, event: AppEvent) {
        let _ = self.event_sender.send(event);
    }

    pub fn transition_to(&self, new_state: AppState) -> Result<(), String> {
        let current_state = self.get_state();

        // Validate state transitions
        match (&current_state, &new_state) {
            (AppState::Empty, AppState::FilesSelected(_)) => Ok(()),
            (AppState::Empty, AppState::ArchiveLoaded(_)) => Ok(()),
            (AppState::FilesSelected(_), AppState::Processing(_)) => Ok(()),
            (AppState::ArchiveLoaded(_), AppState::Processing(_)) => Ok(()),
            (AppState::Processing(_), AppState::Empty) => Ok(()),
            (AppState::Processing(_), AppState::FilesSelected(_)) => Ok(()),
            (AppState::Processing(_), AppState::ArchiveLoaded(_)) => Ok(()),
            (AppState::Processing(_), AppState::Error(_)) => Ok(()),
            (AppState::Error(_), AppState::Empty) => Ok(()),
            (AppState::Error(_), AppState::FilesSelected(_)) => Ok(()),
            (AppState::Error(_), AppState::ArchiveLoaded(_)) => Ok(()),
            _ => {
                Err(format!("Invalid state transition from {:?} to {:?}", current_state, new_state))
            }
        }?;

        self.set_state(new_state);
        Ok(())
    }
}

impl Default for AppStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_state_transitions() {
        let state_manager = AppStateManager::new();

        // Test valid transitions
        assert_eq!(state_manager.get_state(), AppState::Empty);

        let files = vec![PathBuf::from("test.txt")];
        assert!(state_manager.transition_to(AppState::FilesSelected(files.clone())).is_ok());

        let operation = Operation::CreateArchive {
            output: PathBuf::from("test.zip"),
            files: files.clone(),
        };
        assert!(state_manager.transition_to(AppState::Processing(operation)).is_ok());

        assert!(state_manager.transition_to(AppState::Empty).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        let state_manager = AppStateManager::new();

        // Test invalid transition
        let operation = Operation::CreateArchive {
            output: PathBuf::from("test.zip"),
            files: vec![PathBuf::from("test.txt")],
        };
        assert!(state_manager.transition_to(AppState::Processing(operation)).is_err());
    }

    #[test]
    fn test_event_system() {
        let state_manager = AppStateManager::new();
        let mut receiver = state_manager.subscribe();

        // Test event emission
        let files = vec![PathBuf::from("test.txt")];
        state_manager.emit_event(AppEvent::FilesAdded(files.clone()));

        // Check that event was received
        if let Ok(event) = receiver.try_recv() {
            match event {
                AppEvent::FilesAdded(received_files) => {
                    assert_eq!(received_files, files);
                }
                _ => panic!("Wrong event type received"),
            }
        } else {
            panic!("No event received");
        }
    }
}
