use crate::archive::ArchiveManager;
use slint::{Model, VecModel};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

slint::include_modules!();

pub fn run_gui() -> Result<(), slint::PlatformError> {
    let app_window = AppWindow::new()?;
    let archive_manager = Rc::new(RefCell::new(ArchiveManager::new()));
    let current_files = Rc::new(VecModel::default());
    let current_archive_path = Rc::new(RefCell::new(None::<PathBuf>));

    // Set initial state with sample files
    let sample_files = vec![
        FileEntry {
            name: "Documents".into(),
            size: "14.5 KB".into(),
            r#type: "Folder".into(),
            modified: "01/13/2024".into(),
            selected: false,
        },
        FileEntry {
            name: "image.png".into(),
            size: "75.2 KB".into(),
            r#type: "MPEG".into(),
            modified: "01/23/2024".into(),
            selected: false,
        },
        FileEntry {
            name: "Video.mp4".into(),
            size: "74.2 MB".into(),
            r#type: "MPEG".into(),
            modified: "11/7/2024".into(),
            selected: false,
        },
        FileEntry {
            name: "Presentation.pptx".into(),
            size: "5.52 MB".into(),
            r#type: "PDF".into(),
            modified: "10/20/2024".into(),
            selected: false,
        },
    ];
    
    current_files.set_vec(sample_files);
    app_window.set_files(current_files.clone().into());
    app_window.set_app_state(AppState::Empty);
    app_window.set_primary_button_text("Compress".into());
    app_window.set_primary_button_enabled(true);

    // Set up callbacks
    {
        let current_files = current_files.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_add_files(move || {
            println!("Add files button clicked - opening file dialog...");
            
            match rfd::FileDialog::new()
                .set_title("Select files to add to archive")
                .pick_files()
            {
                Some(files) => {
                    println!("Selected {} files", files.len());
                    for file_path in files {
                        println!("Adding file: {:?}", file_path);
                        if let Ok(metadata) = std::fs::metadata(&file_path) {
                            let file_entry = FileEntry {
                                name: file_path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    .into(),
                                size: format_file_size(metadata.len()).into(),
                                r#type: get_file_type(&file_path).into(),
                                modified: format_modified_time(&metadata).into(),
                                selected: false,
                            };
                            current_files.push(file_entry);
                        }
                    }
                    
                    if let Some(app_window) = app_window_weak.upgrade() {
                        app_window.set_primary_button_enabled(current_files.row_count() > 0);
                        app_window.set_app_state(AppState::Empty);
                    }
                },
                None => {
                    println!("File dialog was cancelled");
                }
            }
        });
    }

    {
        let archive_manager = archive_manager.clone();
        let current_files = current_files.clone();
        let current_archive_path = current_archive_path.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_open_archive(move || {
            println!("Open archive button clicked - opening file dialog...");
            
            match rfd::FileDialog::new()
                .set_title("Open archive")
                .add_filter("Archives", &["zip", "tar", "gz", "7z"])
                .pick_file()
            {
                Some(archive_path) => {
                    println!("Selected archive: {:?}", archive_path);
                let manager = archive_manager.borrow();
                match manager.list_archive(&archive_path) {
                    Ok(contents) => {
                        current_files.set_vec(
                            contents.into_iter().map(|name| FileEntry {
                                name: name.into(),
                                size: "Unknown".into(),
                                r#type: "File".into(),
                                modified: "Unknown".into(),
                                selected: false,
                            }).collect::<Vec<_>>()
                        );
                        
                        *current_archive_path.borrow_mut() = Some(archive_path.clone());
                        
                        if let Some(app_window) = app_window_weak.upgrade() {
                            app_window.set_app_state(AppState::ReadyArchive);
                            app_window.set_primary_button_text("Extract".into());
                            app_window.set_primary_button_enabled(true);
                            app_window.set_archive_name(
                                archive_path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    .into()
                            );
                        }
                    },
                    Err(e) => {
                        eprintln!("Error opening archive: {}", e);
                    }
                }
                },
                None => {
                    println!("Archive dialog was cancelled");
                }
            }
        });
    }

    {
        let archive_manager = archive_manager.clone();
        let current_files = current_files.clone();
        let current_archive_path = current_archive_path.clone();
        let app_window_weak = app_window.as_weak();
        app_window.on_primary_action(move || {
            if let Some(app_window) = app_window_weak.upgrade() {
                let state = app_window.get_app_state();
                match state {
                    AppState::Empty => {
                        // Compress files
                        if current_files.row_count() > 0 {
                            if let Some(save_path) = rfd::FileDialog::new()
                                .set_title("Save archive as...")
                                .add_filter("ZIP Archive", &["zip"])
                                .save_file()
                            {
                                app_window.set_app_state(AppState::Building);
                                app_window.set_primary_button_enabled(false);
                                
                                // For demo purposes, create a simple archive with test files
                                println!("Archive creation requested - saving to: {:?}", save_path);
                                println!("Files to include:");
                                for i in 0..current_files.row_count() {
                                    if let Some(file) = current_files.row_data(i) {
                                        println!("  - {}", file.name);
                                    }
                                }
                                
                                // In a real implementation, we would use the actual file paths
                                // For now, just simulate success
                                app_window.set_app_state(AppState::Empty);
                                app_window.set_primary_button_enabled(true);
                            }
                        }
                    },
                    AppState::ReadyArchive => {
                        // Extract archive
                        if let Some(extract_path) = rfd::FileDialog::new()
                            .set_title("Extract to folder...")
                            .pick_folder()
                        {
                            app_window.set_app_state(AppState::Extracting);
                            app_window.set_primary_button_enabled(false);
                            
                            if let Some(archive_path) = current_archive_path.borrow().as_ref() {
                                let manager = archive_manager.borrow();
                                match manager.extract_archive(archive_path, &extract_path) {
                                    Ok(_) => {
                                        println!("Archive extracted successfully to: {:?}", extract_path);
                                        app_window.set_app_state(AppState::ReadyArchive);
                                        app_window.set_primary_button_enabled(true);
                                    },
                                    Err(e) => {
                                        eprintln!("Error extracting archive: {}", e);
                                        app_window.set_app_state(AppState::ReadyArchive);
                                        app_window.set_primary_button_enabled(true);
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        });
    }

    {
        let current_files = current_files.clone();
        app_window.on_file_selected(move |index| {
            if let Some(mut file) = current_files.row_data(index as usize) {
                file.selected = !file.selected;
                current_files.set_row_data(index as usize, file);
            }
        });
    }

    // B3: Copy Path - Copies archive or selected file paths to clipboard
    app_window.on_copy_path(|| {
        println!("Copy Path: Would copy selected file paths to clipboard");
        // In a real implementation: use clipboard crate to copy paths
    });

    // B4: Share - OS share sheet (macOS Services / Windows Share)
    app_window.on_share(|| {
        println!("Share: Would open OS share sheet for current archive");
        // In a real implementation: use OS-specific share APIs
    });

    // B5: Settings - Global prefs: theme, default compression level, language
    app_window.on_show_settings(|| {
        println!("Settings: Would show preferences dialog for:");
        println!("  - Theme (dark/light/auto)");
        println!("  - Default compression level");
        println!("  - Language selection");
        println!("  - Archive format preferences");
    });

    // B6: Tools - Power-user tools (test archive, repair, split, checksum)
    app_window.on_show_tools(|| {
        println!("Tools: Would show power-user tools dialog:");
        println!("  - Test archive integrity");
        println!("  - Repair corrupted archives");
        println!("  - Split large archives");
        println!("  - Calculate checksums");
        println!("  - Archive optimization");
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
    use std::time::SystemTime;
    
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
            // Simple date formatting
            return format!("{}", duration.as_secs());
        }
    }
    "Unknown".to_string()
}