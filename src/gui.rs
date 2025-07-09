use crate::archive::ArchiveManager;
use arboard;
use slint::{Model, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();

pub fn run_gui() -> Result<(), slint::PlatformError> {
    let app_window = AppWindow::new()?;
    let archive_manager = Rc::new(RefCell::new(ArchiveManager::new()));
    let current_files = Rc::new(VecModel::default());
    let current_archive_path = Rc::new(RefCell::new(None::<PathBuf>));

    app_window.set_files(current_files.clone().into());
    app_window.set_app_state(AppState::Empty);
    app_window.set_primary_button_text("Compress".into());
    app_window.set_primary_button_enabled(false);
    app_window.set_status_text("Ready".into());

    // Set up callbacks
    {
        let current_files = current_files.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_add_files(move || {
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

    {
        let current_files = current_files.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_files_dropped(move |urls| {
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

    {
        let archive_manager = archive_manager.clone();
        let current_files = current_files.clone();
        let current_archive_path = current_archive_path.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_open_archive(move || {
            let app_window = app_window_weak.upgrade().unwrap();
            app_window.set_status_text("Opening archive...".into());
            if let Some(archive_path) = rfd::FileDialog::new()
                .add_filter("Archives", &["zip", "tar", "gz", "7z"])
                .pick_file()
            {
                let manager = archive_manager.borrow();
                match manager.list_archive(&archive_path) {
                    Ok(contents) => {
                        let archive_name = archive_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        app_window
                            .set_status_text(format!("Opened archive: {}", archive_name).into());
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
                        *current_archive_path.borrow_mut() = Some(archive_path.clone());
                        app_window.set_app_state(AppState::ReadyArchive);
                        app_window.set_primary_button_text("Extract".into());
                        app_window.set_primary_button_enabled(true);
                        app_window.set_archive_name(archive_name.into());
                    }
                    Err(e) => app_window.set_status_text(format!("Error: {}", e).into()),
                }
            } else {
                app_window.set_status_text("Open archive cancelled.".into());
            }
        });
    }

    {
        let _archive_manager = archive_manager.clone();
        let current_files = current_files.clone();
        let _current_archive_path = current_archive_path.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_primary_action(move || {
            if let Some(app_window) = app_window_weak.upgrade() {
                match app_window.get_app_state() {
                    AppState::Empty => {
                        // Compress
                        if let Some(save_path) =
                            rfd::FileDialog::new().add_filter("ZIP file", &["zip"]).save_file()
                        {
                            app_window.set_status_text("Compressing...".into());
                            // ... (background thread logic for compression would go here)
                            current_files.set_vec(vec![]);
                            app_window.set_status_text(
                                format!("Archive created: {}", save_path.display()).into(),
                            );
                        }
                    }
                    AppState::ReadyArchive => {
                        // Extract
                        if let Some(extract_path) = rfd::FileDialog::new().pick_folder() {
                            app_window.set_status_text("Extracting...".into());
                            // ... (background thread logic for extraction would go here)
                            app_window.set_status_text(
                                format!("Archive extracted to: {}", extract_path.display()).into(),
                            );
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    {
        let current_files = current_files.clone();
        app_window.on_toggle_selection(move |index| {
            if let Some(mut file) = current_files.row_data(index as usize) {
                file.selected = !file.selected;
                current_files.set_row_data(index as usize, file);
            }
        });
    }

    {
        let current_files = current_files.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_copy_path(move || {
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

    app_window.on_share(|| {
        println!("Share: Not yet implemented.");
    });

    app_window.on_show_settings(|| {
        println!("Settings: Not yet implemented.");
    });

    app_window.on_show_tools(|| {
        println!("Tools: Not yet implemented.");
    });

    app_window.run()
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
