use crate::archive::ArchiveManager;
use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rolypoly")]
#[command(about = "A modern ZIP archiver written in Rust")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Emit machine-readable JSON to stdout
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Emit progress updates (JSON if --json, otherwise human)
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub progress: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new ZIP archive
    Create {
        /// Name of the archive to create
        archive: PathBuf,
        /// Files and directories to add to the archive
        files: Vec<PathBuf>,
    },
    /// Extract a ZIP archive
    Extract {
        /// Path to the archive to extract
        archive: PathBuf,
        /// Directory to extract to (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// List contents of a ZIP archive
    List {
        /// Path to the archive to list
        archive: PathBuf,
    },
    /// Validate the integrity of a ZIP archive
    Validate {
        /// Path to the archive to validate
        archive: PathBuf,
    },
    /// Show statistics about a ZIP archive
    Stats {
        /// Path to the archive to analyze
        archive: PathBuf,
    },
    /// Calculate SHA256 hash of a file
    Hash {
        /// Path to the file to hash
        file: PathBuf,
    },
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let manager = ArchiveManager::new();

        match self.command {
            Commands::Create { archive, files } => {
                if files.is_empty() {
                    return Err(anyhow::anyhow!("No files specified to add to archive"));
                }
                let file_refs: Vec<&PathBuf> = files.iter().collect();
                manager.create_archive(&archive, &file_refs)?;
                if self.json {
                    #[derive(Serialize)]
                    struct Out<'a> { event: &'a str, archive: String }
                    println!("{}", serde_json::to_string(&Out { event: "created", archive: archive.display().to_string() })?);
                }
                // Otherwise progress and completion messages are handled by the archiver
            }
            Commands::Extract { archive, output } => {
                manager.extract_archive(&archive, &output)?;
                if self.json {
                    #[derive(Serialize)]
                    struct Out<'a> { event: &'a str, archive: String, output: String }
                    println!("{}", serde_json::to_string(&Out { event: "extracted", archive: archive.display().to_string(), output: output.display().to_string() })?);
                }
                // Otherwise progress and completion messages are handled by the archiver
            }
            Commands::List { archive } => {
                let contents = manager.list_archive(&archive)?;
                if self.json {
                    #[derive(Serialize)]
                    struct Out { archive: String, files: Vec<String> }
                    println!("{}", serde_json::to_string(&Out { archive: archive.display().to_string(), files: contents })?);
                } else {
                    println!("Archive: {}", archive.display());
                    if contents.is_empty() {
                        println!("Archive is empty");
                    } else {
                        for item in contents { println!("  {item}"); }
                    }
                }
            }
            Commands::Validate { archive } => {
                let is_valid = manager.validate_archive(&archive)?;
                if self.json {
                    #[derive(Serialize)]
                    struct Out { archive: String, valid: bool }
                    println!("{}", serde_json::to_string(&Out { archive: archive.display().to_string(), valid: is_valid })?);
                } else {
                    if is_valid {
                        println!("✓ Archive is valid and all files passed integrity checks");
                    } else {
                        println!("✗ Archive validation failed");
                    }
                }
            }
            Commands::Stats { archive } => {
                let stats = manager.get_archive_stats(&archive)?;
                if self.json {
                    println!("{}", serde_json::to_string(&stats)?);
                } else {
                    println!("Archive Statistics:");
                    println!("  Files: {}", stats.file_count);
                    println!("  Directories: {}", stats.dir_count);
                    println!("  Uncompressed size: {} bytes", stats.total_uncompressed_size);
                    println!("  Compressed size: {} bytes", stats.total_compressed_size);
                    println!("  Compression ratio: {:.1}%", stats.compression_ratio);
                    if stats.total_uncompressed_size > 0 {
                        if stats.total_uncompressed_size > stats.total_compressed_size {
                            let space_saved = stats.total_uncompressed_size - stats.total_compressed_size;
                            println!("  Space saved: {space_saved} bytes");
                        } else {
                            let space_increased = stats.total_compressed_size - stats.total_uncompressed_size;
                            println!("  Space increased: {space_increased} bytes (due to compression overhead)");
                        }
                    }
                }
            }
            Commands::Hash { file } => {
                let hash = manager.calculate_file_hash(&file)?;
                if self.json {
                    #[derive(Serialize)]
                    struct Out { file: String, algo: &'static str, hash: String }
                    println!("{}", serde_json::to_string(&Out { file: file.display().to_string(), algo: "sha256", hash })?);
                } else {
                    println!("SHA256: {hash}");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cli_create_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create a test file
        fs::write(&test_file, "Hello, World!")?;

        // Test create command
        let cli = Cli {
            command: Commands::Create {
                archive: archive_path.clone(),
                files: vec![test_file],
            },
        };

        cli.run()?;

        // Verify archive was created
        assert!(archive_path.exists());
        assert!(archive_path.metadata()?.len() > 0);

        Ok(())
    }

    #[test]
    fn test_cli_extract_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extract");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Create extract directory
        fs::create_dir(&extract_dir)?;

        // Test extract command
        let cli = Cli {
            command: Commands::Extract {
                archive: archive_path,
                output: extract_dir.clone(),
            },
        };

        cli.run()?;

        // Verify extracted file
        let extracted_file = extract_dir.join("test.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(&extracted_file)?, "Hello, World!");

        Ok(())
    }

    #[test]
    fn test_cli_list_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test files and archive
        fs::write(&test_file1, "Hello, World!")?;
        fs::write(&test_file2, "Goodbye, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file1, &test_file2])?;

        // Test list command
        let cli = Cli {
            command: Commands::List {
                archive: archive_path,
            },
        };

        // This will print to stdout, but we can't easily capture it in tests
        // The actual functionality is tested in the archive module
        cli.run()?;

        Ok(())
    }

    #[test]
    fn test_cli_create_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");

        let cli = Cli {
            command: Commands::Create {
                archive: archive_path,
                files: vec![],
            },
        };

        // This should return an error
        let result = cli.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files specified"));
    }

    #[test]
    fn test_cli_validate_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Test validate command
        let cli = Cli {
            command: Commands::Validate {
                archive: archive_path,
            },
        };

        cli.run()?;

        Ok(())
    }

    #[test]
    fn test_cli_stats_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test file and archive
        fs::write(&test_file, "Hello, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Test stats command
        let cli = Cli {
            command: Commands::Stats {
                archive: archive_path,
            },
        };

        cli.run()?;

        Ok(())
    }

    #[test]
    fn test_cli_hash_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");

        // Create test file
        fs::write(&test_file, "Hello, World!")?;

        // Test hash command
        let cli = Cli {
            command: Commands::Hash { file: test_file },
        };

        cli.run()?;

        Ok(())
    }
}
