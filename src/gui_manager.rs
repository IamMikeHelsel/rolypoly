use crate::archive::ArchiveManager;
use crate::operations::OperationManager;
use crate::state::{AppEvent, AppState, AppStateManager, Operation};
use slint::{Model, VecModel, Weak};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

slint::include_modules!();

pub struct GuiManager {
    app_window: AppWindow,
    archive_manager: Arc<ArchiveManager>,
    state_manager: Arc<AppStateManager>,
    operation_manager: Arc<OperationManager>,
    current_files: Arc<VecModel<FileEntry>>,
    runtime: tokio::runtime::Runtime,
}

impl GuiManager {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let app_window = AppWindow::new()?;
        let archive_manager = Arc::new(ArchiveManager::new());
        let state_manager = Arc::new(AppStateManager::new());
        let operation_manager = Arc::new(OperationManager::new(
            archive_manager.clone(),
            state_manager.clone(),
        ));
        let current_files = Arc::new(VecModel::default());
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut gui_manager = Self {
            app_window,
            archive_manager,
            state_manager,
            operation_manager,
            current_files,
            runtime,
        };

        gui_manager.setup_ui();
        gui_manager.setup_callbacks();
        gui_manager.setup_event_listeners();

        Ok(gui_manager)
    }

    fn setup_ui(&self) {
        self.app_window.set_files(self.current_files.clone().into());
        self.app_window.set_app_state(AppState::Empty);
        self.app_window.set_primary_button_text("Compress".into());
        self.app_window.set_primary_button_enabled(false);
        self.app_window.set_status_text("Ready".into());
    }

    fn setup_callbacks(&self) {
        self.setup_add_files_callback();
        self.setup_files_dropped_callback();
        self.setup_open_archive_callback();
        self.setup_primary_action_callback();
        self.setup_toggle_selection_callback();
        self.setup_copy_path_callback();
        self.setup_utility_callbacks();
    }

    fn setup_add_files_callback(&self) {
        let current_files = self.current_files.clone();
        let state_manager = self.state_manager.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_add_files(move || {
            let app_window = app_window_weak.upgrade().unwrap();
            app_window.set_status_text("Opening file dialog...".into());
            
            if let Some(files) = rfd::FileDialog::new().pick_files() {
                let count = files.len();
                let mut file_paths = Vec::new();
                
                for file_path in files {
                    if let Ok(metadata) = std::fs::metadata(&file_path) {
                        current_files.push(FileEntry {
                            name: file_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                                .into(),
                            path: file_path.to_string_lossy().to_string().into(),
                            size: format_file_size(metadata.len()).into(),
                            r#type: get_file_type(&file_path).into(),
                            modified: format_modified_time(&metadata).into(),
                            selected: false,
                        });
                        file_paths.push(file_path);
                    }
                }
                
                app_window.set_primary_button_enabled(current_files.row_count() > 0);
                state_manager.emit_event(AppEvent::FilesAdded(file_paths.clone()));
                let _ = state_manager.transition_to(AppState::FilesSelected(file_paths));
                app_window.set_status_text(format!("Added {} files.", count).into());
            } else {
                app_window.set_status_text("File dialog cancelled.".into());
            }
        });
    }

    fn setup_files_dropped_callback(&self) {
        let current_files = self.current_files.clone();
        let state_manager = self.state_manager.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_files_dropped(move |urls| {
            let app_window = app_window_weak.upgrade().unwrap();
            let count = urls.row_count();
            let mut file_paths = Vec::new();
            
            for i in 0..urls.row_count() {
                if let Some(url) = urls.row_data(i) {
                    if let Ok(path) = std::path::PathBuf::from(url.as_str()).canonicalize() {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            current_files.push(FileEntry {
                                name: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    .into(),
                                path: path.to_string_lossy().to_string().into(),
                                size: format_file_size(metadata.len()).into(),
                                r#type: get_file_type(&path).into(),
                                modified: format_modified_time(&metadata).into(),
                                selected: false,
                            });
                            file_paths.push(path);
                        }
                    }
                }
            }
            
            app_window.set_primary_button_enabled(current_files.row_count() > 0);
            state_manager.emit_event(AppEvent::FilesAdded(file_paths.clone()));
            let _ = state_manager.transition_to(AppState::FilesSelected(file_paths));
            app_window.set_status_text(format!("Dropped {} files.", count).into());
        });
    }

    fn setup_open_archive_callback(&self) {
        let archive_manager = self.archive_manager.clone();
        let current_files = self.current_files.clone();
        let state_manager = self.state_manager.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_open_archive(move || {
            let app_window = app_window_weak.upgrade().unwrap();
            app_window.set_status_text("Opening archive...".into());
            
            if let Some(archive_path) = rfd::FileDialog::new()
                .add_filter("Archives", &["zip", "tar", "gz", "7z"])
                .pick_file()
            {
                let manager = archive_manager.clone();
                let archive_path_clone = archive_path.clone();
                
                match manager.list_archive(&archive_path) {
                    Ok(contents) => {
                        let archive_name = archive_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                            
                        app_window.set_status_text(format!("Opened archive: {}", archive_name).into());
                        
                        current_files.set_vec(
                            contents
                                .into_iter()
                                .map(|name| FileEntry {
                                    name: name.clone().into(),
                                    path: name.into(),
                                    size: "N/A".into(),
                                    r#type: "File".into(),
                                    modified: "N/A".into(),
                                    selected: false,
                                })
                                .collect::<Vec<_>>(),
                        );
                        
                        state_manager.emit_event(AppEvent::ArchiveOpened(archive_path_clone.clone()));
                        let _ = state_manager.transition_to(AppState::ArchiveLoaded(archive_path_clone));
                        app_window.set_app_state(AppState::ReadyArchive);
                        app_window.set_primary_button_text("Extract".into());
                        app_window.set_primary_button_enabled(true);
                        app_window.set_archive_name(archive_name.into());
                    }
                    Err(e) => {
                        app_window.set_status_text(format!("Error: {}", e).into());
                        let _ = state_manager.transition_to(AppState::Error(e.to_string()));
                    }
                }
            } else {
                app_window.set_status_text("Open archive cancelled.".into());
            }
        });
    }

    fn setup_primary_action_callback(&self) {
        let operation_manager = self.operation_manager.clone();
        let current_files = self.current_files.clone();
        let state_manager = self.state_manager.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_primary_action(move || {
            if let Some(app_window) = app_window_weak.upgrade() {
                let current_state = state_manager.get_state();
                let operation_manager = operation_manager.clone();
                let current_files = current_files.clone();
                let state_manager = state_manager.clone();
                let app_window_weak = app_window.as_weak();

                match current_state {
                    AppState::FilesSelected(files) => {
                        // Compress operation
                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("ZIP file", &["zip"])
                            .save_file()
                        {
                            let operation = Operation::CreateArchive {
                                output: save_path.clone(),
                                files: files.clone(),
                            };

                            tokio::spawn(async move {
                                let _ = state_manager.transition_to(AppState::Processing(operation.clone()));
                                
                                if let Some(app_window) = app_window_weak.upgrade() {
                                    app_window.set_status_text("Compressing...".into());
                                    app_window.set_primary_button_enabled(false);
                                }

                                match operation_manager.execute_operation(operation).await {
                                    Ok(_) => {
                                        let _ = state_manager.transition_to(AppState::Empty);
                                        if let Some(app_window) = app_window_weak.upgrade() {
                                            current_files.set_vec(vec![]);
                                            app_window.set_status_text(
                                                format!("Archive created: {}", save_path.display()).into(),
                                            );
                                            app_window.set_primary_button_enabled(false);
                                        }
                                    }
                                    Err(e) => {
                                        let _ = state_manager.transition_to(AppState::Error(e.clone()));
                                        if let Some(app_window) = app_window_weak.upgrade() {
                                            app_window.set_status_text(format!("Error: {}", e).into());
                                            app_window.set_primary_button_enabled(true);
                                        }
                                    }
                                }
                            });
                        }
                    }
                    AppState::ArchiveLoaded(archive_path) => {
                        // Extract operation
                        if let Some(extract_path) = rfd::FileDialog::new().pick_folder() {
                            let operation = Operation::ExtractArchive {
                                archive: archive_path.clone(),
                                output: extract_path.clone(),
                            };

                            tokio::spawn(async move {
                                let _ = state_manager.transition_to(AppState::Processing(operation.clone()));
                                
                                if let Some(app_window) = app_window_weak.upgrade() {
                                    app_window.set_status_text("Extracting...".into());
                                    app_window.set_primary_button_enabled(false);
                                }

                                match operation_manager.execute_operation(operation).await {
                                    Ok(_) => {
                                        let _ = state_manager.transition_to(AppState::ArchiveLoaded(archive_path));
                                        if let Some(app_window) = app_window_weak.upgrade() {
                                            app_window.set_status_text(
                                                format!("Archive extracted to: {}", extract_path.display()).into(),
                                            );
                                            app_window.set_primary_button_enabled(true);
                                        }
                                    }
                                    Err(e) => {
                                        let _ = state_manager.transition_to(AppState::Error(e.clone()));
                                        if let Some(app_window) = app_window_weak.upgrade() {
                                            app_window.set_status_text(format!("Error: {}", e).into());
                                            app_window.set_primary_button_enabled(true);
                                        }
                                    }
                                }
                            });
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn setup_toggle_selection_callback(&self) {
        let current_files = self.current_files.clone();
        
        self.app_window.on_toggle_selection(move |index| {
            if let Some(mut file) = current_files.row_data(index as usize) {
                file.selected = !file.selected;
                current_files.set_row_data(index as usize, file);
            }
        });
    }

    fn setup_copy_path_callback(&self) {
        let current_files = self.current_files.clone();
        let app_window_weak = self.app_window.as_weak();
        
        self.app_window.on_copy_path(move || {
            if let Some(app_window) = app_window_weak.upgrade() {
                let selected_paths: Vec<String> = current_files
                    .iter()
                    .filter(|f| f.selected)
                    .map(|f| f.path.to_string())
                    .collect();

                if !selected_paths.is_empty() {
                    let paths_str = selected_paths.join("\n");
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(paths_str);
                        app_window.set_status_text(
                            format!("Copied {} paths to clipboard.", selected_paths.len()).into(),
                        );
                    } else {
                        app_window.set_status_text("Failed to access clipboard.".into());
                    }
                } else {
                    app_window.set_status_text("No files selected to copy.".into());
                }
            }
        });
    }

    fn setup_utility_callbacks(&self) {
        self.app_window.on_share(|| {
            println!("Share: Not yet implemented.");
        });

        self.app_window.on_show_settings(|| {
            println!("Settings: Not yet implemented.");
        });

        self.app_window.on_show_tools(|| {
            println!("Tools: Not yet implemented.");
        });
    }

    fn setup_event_listeners(&self) {
        let mut event_receiver = self.state_manager.subscribe();
        let app_window_weak = self.app_window.as_weak();

        // Spawn a task to listen for state changes
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                if let Some(app_window) = app_window_weak.upgrade() {
                    match event {
                        AppEvent::OperationProgress(operation, progress) => {
                            let progress_text = format!("{:.0}%", progress * 100.0);
                            match operation {
                                Operation::CreateArchive { .. } => {
                                    app_window.set_status_text(format!("Compressing... {}", progress_text).into());
                                }
                                Operation::ExtractArchive { .. } => {
                                    app_window.set_status_text(format!("Extracting... {}", progress_text).into());
                                }
                                Operation::ValidateArchive { .. } => {
                                    app_window.set_status_text(format!("Validating... {}", progress_text).into());
                                }
                                Operation::CalculateHash { .. } => {
                                    app_window.set_status_text(format!("Calculating hash... {}", progress_text).into());
                                }
                            }
                        }
                        AppEvent::StateChanged(new_state) => {
                            match new_state {
                                AppState::Empty => {
                                    app_window.set_app_state(AppState::Empty);
                                    app_window.set_primary_button_text("Compress".into());
                                    app_window.set_primary_button_enabled(false);
                                }
                                AppState::FilesSelected(_) => {
                                    app_window.set_app_state(AppState::Empty);
                                    app_window.set_primary_button_text("Compress".into());
                                    app_window.set_primary_button_enabled(true);
                                }
                                AppState::ArchiveLoaded(_) => {
                                    app_window.set_app_state(AppState::ReadyArchive);
                                    app_window.set_primary_button_text("Extract".into());
                                    app_window.set_primary_button_enabled(true);
                                }
                                AppState::Processing(_) => {
                                    app_window.set_app_state(AppState::Building);
                                    app_window.set_primary_button_enabled(false);
                                }
                                AppState::Error(error) => {
                                    app_window.set_status_text(format!("Error: {}", error).into());
                                    app_window.set_primary_button_enabled(true);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.app_window.run()
    }
}

// Helper functions (moved from gui.rs)
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

fn get_file_type(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_uppercase())
        .unwrap_or_else(|| "File".to_string())
}

fn format_modified_time(metadata: &std::fs::Metadata) -> String {
    use chrono::{DateTime, Utc};

    if let Ok(modified) = metadata.modified() {
        let datetime: DateTime<Utc> = modified.into();
        return datetime.format("%Y-%m-%d %H:%M").to_string();
    }
    "Unknown".to_string()
}