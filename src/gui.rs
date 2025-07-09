use crate::archive::{ArchiveManager, ArchiveStats};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
    pub code: String,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for ErrorResponse {
    fn description(&self) -> &str {
        &self.error
    }
    
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> From<T> for SuccessResponse<T> {
    fn from(data: T) -> Self {
        SuccessResponse {
            success: true,
            data,
        }
    }
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(error: anyhow::Error) -> Self {
        let error_msg = error.to_string();
        let (code, fun_message) = if error_msg.contains("No such file") || error_msg.contains("does not exist") {
            ("FILE_NOT_FOUND", get_file_not_found_message())
        } else if error_msg.contains("Permission denied") {
            ("PERMISSION_DENIED", get_permission_denied_message())
        } else if error_msg.contains("Invalid ZIP") || error_msg.contains("corrupt") || error_msg.contains("invalid Zip archive") || error_msg.contains("Could not find EOCD") {
            ("INVALID_ARCHIVE", get_invalid_archive_message())
        } else if error_msg.contains("No space left") {
            ("DISK_FULL", get_disk_full_message())
        } else {
            ("UNKNOWN_ERROR", get_unknown_error_message())
        };

        ErrorResponse {
            error: fun_message.to_string(),
            details: Some(format!("Technical details: {error_msg}")),
            code: code.to_string(),
        }
    }
}

// Safe wrapper for all GUI operations
fn safe_execute<T, F>(operation: F) -> std::result::Result<SuccessResponse<T>, ErrorResponse>
where
    F: FnOnce() -> Result<T>,
    T: serde::Serialize,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)) {
        Ok(Ok(result)) => Ok(result.into()),
        Ok(Err(e)) => {
            eprintln!("Operation failed: {e}");
            Err(e.into())
        }
        Err(panic) => {
            let panic_msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };
            
            eprintln!("Panic caught: {panic_msg}");
            Err(ErrorResponse {
                error: "Internal error occurred".to_string(),
                details: Some(panic_msg),
                code: "PANIC".to_string(),
            })
        }
    }
}

#[command]
pub async fn create_archive(
    archive_path: String,
    files: Vec<String>,
) -> std::result::Result<SuccessResponse<String>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if archive_path.trim().is_empty() {
            return Err(anyhow::anyhow!("Archive path cannot be empty"));
        }
        
        if files.is_empty() {
            return Err(anyhow::anyhow!("No files provided for archiving"));
        }
        
        // Validate file paths
        for file in &files {
            if file.trim().is_empty() {
                return Err(anyhow::anyhow!("Invalid file path provided"));
            }
        }
        
        let manager = ArchiveManager::new();
        let archive_path = PathBuf::from(archive_path);
        let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
        let file_refs: Vec<&PathBuf> = file_paths.iter().collect();
        
        manager.create_archive(&archive_path, &file_refs)?;
        
        Ok(format!("{} Archive created: {}", get_create_success_message(), archive_path.display()))
    })
}

#[command]
pub async fn extract_archive(
    archive_path: String,
    output_dir: String,
) -> std::result::Result<SuccessResponse<String>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if archive_path.trim().is_empty() {
            return Err(anyhow::anyhow!("Archive path cannot be empty"));
        }
        
        if output_dir.trim().is_empty() {
            return Err(anyhow::anyhow!("Output directory cannot be empty"));
        }
        
        let manager = ArchiveManager::new();
        let archive_path = PathBuf::from(archive_path);
        let output_dir = PathBuf::from(output_dir);
        
        // Check if archive exists
        if !archive_path.exists() {
            return Err(anyhow::anyhow!("Archive file does not exist: {}", archive_path.display()));
        }
        
        manager.extract_archive(&archive_path, &output_dir)?;
        
        Ok(format!("{} Files extracted to: {}", get_extract_success_message(), output_dir.display()))
    })
}

#[command]
pub async fn list_archive(
    archive_path: String
) -> std::result::Result<SuccessResponse<Vec<String>>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if archive_path.trim().is_empty() {
            return Err(anyhow::anyhow!("Archive path cannot be empty"));
        }
        
        let manager = ArchiveManager::new();
        let archive_path = PathBuf::from(archive_path);
        
        // Check if archive exists
        if !archive_path.exists() {
            return Err(anyhow::anyhow!("Archive file does not exist: {}", archive_path.display()));
        }
        
        let contents = manager.list_archive(&archive_path)?;
        Ok(contents)
    })
}

#[command]
pub async fn validate_archive(
    archive_path: String
) -> std::result::Result<SuccessResponse<bool>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if archive_path.trim().is_empty() {
            return Err(anyhow::anyhow!("Archive path cannot be empty"));
        }
        
        let manager = ArchiveManager::new();
        let archive_path = PathBuf::from(archive_path);
        
        // Check if archive exists
        if !archive_path.exists() {
            return Err(anyhow::anyhow!("Archive file does not exist: {}", archive_path.display()));
        }
        
        let is_valid = manager.validate_archive(&archive_path)?;
        Ok(is_valid)
    })
}

#[command]
pub async fn get_archive_stats(
    archive_path: String
) -> std::result::Result<SuccessResponse<ArchiveStats>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if archive_path.trim().is_empty() {
            return Err(anyhow::anyhow!("Archive path cannot be empty"));
        }
        
        let manager = ArchiveManager::new();
        let archive_path = PathBuf::from(archive_path);
        
        // Check if archive exists
        if !archive_path.exists() {
            return Err(anyhow::anyhow!("Archive file does not exist: {}", archive_path.display()));
        }
        
        let stats = manager.get_archive_stats(&archive_path)?;
        Ok(stats)
    })
}

#[command]
pub async fn calculate_file_hash(
    file_path: String
) -> std::result::Result<SuccessResponse<String>, ErrorResponse> {
    safe_execute(|| {
        // Input validation
        if file_path.trim().is_empty() {
            return Err(anyhow::anyhow!("File path cannot be empty"));
        }
        
        let manager = ArchiveManager::new();
        let file_path = PathBuf::from(file_path);
        
        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", file_path.display()));
        }
        
        if !file_path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {}", file_path.display()));
        }
        
        let hash = manager.calculate_file_hash(&file_path)?;
        Ok(hash)
    })
}

#[command]
pub async fn get_app_info() -> std::result::Result<SuccessResponse<serde_json::Value>, ErrorResponse> {
    safe_execute(|| {
        let info = serde_json::json!({
            "name": "Rusty",
            "version": "0.1.0",
            "description": "Modern ZIP Archiver",
            "author": "Claude Code Assistant",
            "build_time": "unknown",
            "commit": "unknown",
        });
        Ok(info)
    })
}

// Health check command
#[command]
pub async fn health_check() -> std::result::Result<SuccessResponse<String>, ErrorResponse> {
    safe_execute(|| {
        // Test basic functionality
        let manager = ArchiveManager::new();
        
        // Test that we can create a manager instance
        let _ = manager; // Ensure manager is used
        
        Ok(get_health_check_message())
    })
}

// Fun message generators for various operations
fn get_create_success_message() -> &'static str {
    let messages = [
        "🎉 Beep boop! Archive assembled!",
        "📦 All packed up like a digital suitcase!",
        "✨ Files bundled with love!",
        "🦀 Rustacean magic complete!",
        "🎯 Compression mission accomplished!",
        "📁 Your files are now mingling together!",
        "🔧 Engineered for awesomeness!",
        "🚀 Launched into the zip dimension!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_extract_success_message() -> &'static str {
    let messages = [
        "🎪 Herding llamas...",
        "🔄 Reticulating splines...", 
        "🗺️ Mapping distance from Ypsilanti...",
        "🧬 Decompressing digital DNA...",
        "🎭 Unleashing the file spirits...",
        "🔮 Reading the archive tea leaves...",
        "🎨 Painting pixels back to life...",
        "🌟 Sprinkling extraction fairy dust...",
        "🎲 Rolling for file recovery...",
        "⚡ Channeling extraction energy...",
        "🎯 Precision unpacking engaged...",
        "🔧 Reverse-engineering the file puzzle...",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_invalid_archive_message() -> &'static str {
    let messages = [
        "🗑️ Nothing to see here, just a box of digital trash!",
        "🤷 This archive is having an identity crisis!",
        "🚫 Not a valid ZIP file - perhaps it's pretending?",
        "📦 This box seems to be empty... or full of mysteries!",
        "🔍 File format detective says: 'This ain't it, chief!'",
        "💭 Archive.exe has stopped working (because it's not an archive)!",
        "🎭 This file is cosplaying as a ZIP but fooling nobody!",
        "🌪️ Archive integrity: somewhere between 'nope' and 'not happening'!",
        "🔮 The archive oracle says: 'Reply hazy, try again (with a real ZIP)'",
        "🤖 ERROR 404: Valid archive not found!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_file_not_found_message() -> &'static str {
    let messages = [
        "🕵️ File went on vacation without leaving a forwarding address!",
        "👻 Spooky! That file has vanished into the digital void!",
        "🎯 Target acquired... wait, no, target has escaped!",
        "🔍 File playing hide and seek (and winning)!",
        "🚀 Houston, we have a missing file problem!",
        "🎪 File has joined the circus and left town!",
        "📍 GPS can't locate this file - it's off the grid!",
        "🌌 File has transcended to another dimension!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_permission_denied_message() -> &'static str {
    let messages = [
        "🚪 Access denied! You shall not pass (without proper permissions)!",
        "🔐 File is locked tighter than Fort Knox!",
        "👮 File bouncer says: 'You're not on the list!'",
        "🎫 Ticket required! Please upgrade your permissions!",
        "🛡️ File is under witness protection!",
        "🏰 The file castle has raised its drawbridge!",
        "🔑 Wrong key! Try asking the file owner nicely!",
        "👑 This file belongs to the digital nobility!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_disk_full_message() -> &'static str {
    let messages = [
        "💾 Disk is fuller than a Thanksgiving turkey!",
        "📦 Storage overflow! Time for digital spring cleaning!",
        "🚀 Houston, we have a storage problem!",
        "🎪 No more room in the digital circus tent!",
        "🏠 File hotel: 'No Vacancy' sign is up!",
        "📚 Digital library is at maximum capacity!",
        "🎒 Your digital backpack is bursting at the seams!",
        "🌊 Drowning in a sea of ones and zeros!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_unknown_error_message() -> &'static str {
    let messages = [
        "🤖 Beep boop! Something unexpected happened!",
        "🎯 We've encountered a wild error! It's not very effective!",
        "🔧 Oops! A digital gremlin got into the machinery!",
        "🎪 The digital circus has gone off-script!",
        "🌪️ A glitch in the matrix has been detected!",
        "🎲 The dice of fate rolled a critical failure!",
        "🦄 A mythical error appeared! (Very rare!)",
        "⚡ Unexpected plot twist in the file story!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

fn get_health_check_message() -> String {
    let messages = [
        "💪 Feeling strong and ready to zip!",
        "🦀 Rustacean health: Excellent!",
        "⚡ All systems nominal, captain!",
        "🎯 Locked and loaded for file operations!",
        "🚀 Ready for launch into archive space!",
        "🔧 All gears properly oiled and spinning!",
        "🌟 Shining bright like a compressed diamond!",
        "🎪 The archive circus is ready for showtime!",
    ];
    
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_archive_gui() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create a test file
        fs::write(&test_file, "Hello, World!")?;

        // Test create command
        let result = create_archive(
            archive_path.to_string_lossy().to_string(),
            vec![test_file.to_string_lossy().to_string()],
        ).await;

        assert!(result.is_ok());
        assert!(archive_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test with invalid input
        let result = create_archive("".to_string(), vec![]).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error.code, "UNKNOWN_ERROR");
        assert!(!error.error.is_empty());
    }

    #[tokio::test]
    async fn test_safe_execute_panic() {
        let result = safe_execute::<String, _>(|| {
            panic!("Test panic");
        });
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "PANIC");
    }

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(!response.data.is_empty());
        // Accept any fun health message rather than expecting a specific one
        assert!(response.data.len() > 10); // Should be a meaningful message
    }

    #[tokio::test]
    async fn test_input_validation() {
        // Test empty archive path
        let result = list_archive("".to_string()).await;
        assert!(result.is_err());
        
        // Test empty file list
        let result = create_archive("test.zip".to_string(), vec![]).await;
        assert!(result.is_err());
        
        // Test nonexistent file
        let result = list_archive("/nonexistent/path.zip".to_string()).await;
        assert!(result.is_err());
    }
}