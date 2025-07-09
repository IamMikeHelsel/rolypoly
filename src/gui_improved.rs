use crate::archive::ArchiveManager;
use slint::{Model, VecModel};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use tokio::sync::mpsc;

slint::include_modules!();

#[derive(Debug, Clone)]
pub enum GuiOperation {
    CreateArchive { output: PathBuf, files: Vec<PathBuf> },
    ExtractArchive { archive: PathBuf, output: PathBuf },
    ValidateArchive { archive: PathBuf },
    CalculateHash { file: PathBuf },
}

#[derive(Debug, Clone)]
pub enum UiUpdate {
    StatusText(String),
    ClearFiles,
    SetPrimaryButtonEnabled(bool),
}

pub struct GuiController {
    app_window: AppWindow,
    archive_manager: Arc<ArchiveManager>,
    current_files: Rc<VecModel<FileEntry>>,
    current_archive_path: Arc<Mutex<Option<PathBuf>>>,
    operation_tx: mpsc::UnboundedSender<GuiOperation>,
    operation_rx: Mutex<Option<mpsc::UnboundedReceiver<GuiOperation>>>,
}

impl GuiController {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let app_window = AppWindow::new()?;
        let archive_manager = Arc::new(ArchiveManager::new());
        let current_files = Rc::new(VecModel::default());
        let current_archive_path = Arc::new(Mutex::new(None::<PathBuf>));
        let (operation_tx, operation_rx) = mpsc::unbounded_channel();

        Ok(Self {
            app_window,
            archive_manager,
            current_files,
            current_archive_path,
            operation_tx,
            operation_rx: Mutex::new(Some(operation_rx)),
        })
    }

    pub fn setup(&self) {
        self.setup_ui();
        self.setup_callbacks();
        self.setup_operation_handler();
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
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_add_files(move || {
            let app_window = app_window_weak.upgrade().unwrap();
            app_window.set_status_text("Opening file dialog...".into());
            
            if let Some(files) = rfd::FileDialog::new().pick_files() {
                let count = files.len();
                
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
                    }
                }
                
                app_window.set_primary_button_enabled(current_files.row_count() > 0);
                app_window.set_app_state(AppState::Empty);
                app_window.set_status_text(format!("Added {} files.", count).into());
            } else {
                app_window.set_status_text("File dialog cancelled.".into());
            }
        });
    }

    fn setup_files_dropped_callback(&self) {
        let current_files = self.current_files.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_files_dropped(move |urls| {
            let app_window = app_window_weak.upgrade().unwrap();
            let count = urls.row_count();
            
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
                        }
                    }
                }
            }
            
            app_window.set_primary_button_enabled(current_files.row_count() > 0);
            app_window.set_app_state(AppState::Empty);
            app_window.set_status_text(format!("Dropped {} files.", count).into());
        });
    }

    fn setup_open_archive_callback(&self) {
        let archive_manager = self.archive_manager.clone();
        let current_files = self.current_files.clone();
        let current_archive_path = self.current_archive_path.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_open_archive(move || {
            let app_window = app_window_weak.upgrade().unwrap();
            app_window.set_status_text("Opening archive...".into());
            
            if let Some(archive_path) = rfd::FileDialog::new()
                .add_filter("Archives", &["zip", "tar", "gz", "7z"])
                .pick_file()
            {
                let manager = archive_manager.clone();
                
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
                        
                        *current_archive_path.lock().unwrap() = Some(archive_path.clone());
                        app_window.set_app_state(AppState::ReadyArchive);
                        app_window.set_primary_button_text("Extract".into());
                        app_window.set_primary_button_enabled(true);
                        app_window.set_archive_name(archive_name.into());
                    }
                    Err(e) => {
                        app_window.set_status_text(format!("Error: {}", e).into());
                    }
                }
            } else {
                app_window.set_status_text("Open archive cancelled.".into());
            }
        });
    }

    fn setup_primary_action_callback(&self) {
        let current_files = self.current_files.clone();
        let current_archive_path = self.current_archive_path.clone();
        let operation_tx = self.operation_tx.clone();
        let app_window_weak = self.app_window.as_weak();

        self.app_window.on_primary_action(move || {
            if let Some(app_window) = app_window_weak.upgrade() {
                match app_window.get_app_state() {
                    AppState::Empty => {
                        // Compress operation
                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("ZIP file", &["zip"])
                            .save_file()
                        {
                            let files: Vec<PathBuf> = current_files
                                .iter()
                                .map(|f| PathBuf::from(f.path.as_str()))
                                .collect();
                            
                            if !files.is_empty() {
                                let operation = GuiOperation::CreateArchive {
                                    output: save_path,
                                    files,
                                };
                                let _ = operation_tx.send(operation);
                                app_window.set_status_text("Compressing...".into());
                                app_window.set_primary_button_enabled(false);
                            }
                        }
                    }
                    AppState::ReadyArchive => {
                        // Extract operation
                        if let Some(extract_path) = rfd::FileDialog::new().pick_folder() {
                            if let Some(archive_path) = current_archive_path.lock().unwrap().as_ref() {
                                let operation = GuiOperation::ExtractArchive {
                                    archive: archive_path.clone(),
                                    output: extract_path,
                                };
                                let _ = operation_tx.send(operation);
                                app_window.set_status_text("Extracting...".into());
                                app_window.set_primary_button_enabled(false);
                            }
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

    fn setup_operation_handler(&self) {
        let archive_manager = self.archive_manager.clone();
        let app_window_weak = self.app_window.as_weak();
        let mut operation_rx = self.operation_rx.lock().unwrap().take().unwrap();

        tokio::spawn(async move {
            while let Some(operation) = operation_rx.recv().await {
                let archive_manager = archive_manager.clone();
                let app_window_weak = app_window_weak.clone();
                let _operation_clone = operation.clone();

                tokio::spawn(async move {
                    let result = match operation {
                        GuiOperation::CreateArchive { output, files } => {
                            let file_refs: Vec<&PathBuf> = files.iter().collect();
                            archive_manager.create_archive(&output, &file_refs)
                                .map(|_| format!("Archive created: {}", output.display()))
                        }
                        GuiOperation::ExtractArchive { archive, output } => {
                            archive_manager.extract_archive(&archive, &output)
                                .map(|_| format!("Archive extracted to: {}", output.display()))
                        }
                        GuiOperation::ValidateArchive { archive } => {
                            archive_manager.validate_archive(&archive)
                                .map(|valid| format!("Archive is {}", if valid { "valid" } else { "invalid" }))
                        }
                        GuiOperation::CalculateHash { file } => {
                            archive_manager.calculate_file_hash(&file)
                                .map(|hash| format!("Hash: {}", hash))
                        }
                    };

                    // Update UI using invoke_from_event_loop
                    let result_msg = match result {
                        Ok(success_msg) => success_msg,
                        Err(e) => format!("Error: {}", e),
                    };
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(app_window) = app_window_weak.upgrade() {
                            app_window.set_status_text(result_msg.into());
                            app_window.set_primary_button_enabled(true);
                        }
                    }).unwrap();
                    
                    // TODO: Clear files after successful archive creation
                    // This requires access to the files model from main thread
                });
            }
        });
    }


    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.app_window.run()
    }
}

pub fn run_gui_improved() -> Result<(), slint::PlatformError> {
    let controller = GuiController::new()?;
    controller.setup();
    controller.run()
}

// Helper functions
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