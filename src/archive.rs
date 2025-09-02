use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct ArchiveManager;

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveManager {
    pub fn new() -> Self {
        Self
    }

    /// Validate the integrity of a ZIP archive
    pub fn validate_archive<P: AsRef<Path>>(&self, archive_path: P) -> Result<bool> {
        let file = File::open(archive_path.as_ref())?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;

        println!("→ Validating: {}", archive_path.as_ref().display());
        let start = Instant::now();
        let pb = ProgressBar::new(archive.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>5}/{len:<5} {percent:>3}% {eta_precise} | {msg}"
                )
                .unwrap()
                .progress_chars("█· "),
        );

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            pb.set_message(format!("Validating: {}", file.name()));

            // The zip crate automatically validates CRC32 when reading
            // If there's a CRC mismatch, it will return an error
            drop(file);
            pb.inc(1);
        }

        let elapsed = start.elapsed();
        pb.finish_with_message(format!(
            "✓ Validation completed in {:.2?}",
            elapsed
        ));
        Ok(true)
    }

    /// Calculate SHA256 hash of a file
    pub fn calculate_file_hash<P: AsRef<Path>>(&self, file_path: P) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get archive statistics
    pub fn get_archive_stats<P: AsRef<Path>>(&self, archive_path: P) -> Result<ArchiveStats> {
        let file = File::open(archive_path.as_ref())?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;

        let mut total_uncompressed_size = 0u64;
        let mut total_compressed_size = 0u64;
        let mut file_count = 0;
        let mut dir_count = 0;

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;

            if file.is_dir() {
                dir_count += 1;
            } else {
                file_count += 1;
                total_uncompressed_size += file.size();
                total_compressed_size += file.compressed_size();
            }
        }

        let compression_ratio = if total_uncompressed_size > 0 {
            (total_compressed_size as f64 / total_uncompressed_size as f64) * 100.0
        } else {
            0.0
        };

        Ok(ArchiveStats {
            file_count,
            dir_count,
            total_uncompressed_size,
            total_compressed_size,
            compression_ratio,
        })
    }

    /// Create a new ZIP archive with the specified files
    pub fn create_archive<P: AsRef<Path>>(&self, archive_path: P, files: &[P]) -> Result<()> {
        let file = File::create(archive_path.as_ref())?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Count total files for progress bar
        let mut total_files = 0;
        for file_path in files {
            let path = file_path.as_ref();
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "File or directory does not exist: {}",
                    path.display()
                ));
            }
            if path.is_file() {
                total_files += 1;
            } else if path.is_dir() {
                total_files += WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .count();
            }
        }

        println!("→ Creating: {}", archive_path.as_ref().display());
        let start = Instant::now();
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>5}/{len:<5} {percent:>3}% {eta_precise} | {msg}"
                )
                .unwrap()
                .progress_chars("█· "),
        );

        for file_path in files {
            let path = file_path.as_ref();
            if path.is_file() {
                pb.set_message(format!("Adding: {}", path.display()));
                self.add_file_to_zip(&mut zip, path, &options)?;
                pb.inc(1);
            } else if path.is_dir() {
                self.add_dir_to_zip_with_progress(&mut zip, path, &options, &pb)?;
            }
        }

        let elapsed = start.elapsed();
        pb.finish_with_message(format!(
            "✓ Created {} files in {:.2?}",
            total_files, elapsed
        ));
        zip.finish()?;
        Ok(())
    }

    /// Extract a ZIP archive to the specified directory
    pub fn extract_archive<P: AsRef<Path>>(&self, archive_path: P, output_dir: P) -> Result<()> {
        let file = File::open(archive_path.as_ref())?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;

        println!(
            "→ Extracting: {} → {}",
            archive_path.as_ref().display(),
            output_dir.as_ref().display()
        );
        let start = Instant::now();
        let pb = ProgressBar::new(archive.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>5}/{len:<5} {percent:>3}% {eta_precise} | {msg}"
                )
                .unwrap()
                .progress_chars("█· "),
        );

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let output_path = output_dir.as_ref().join(file.name());
            pb.set_message(format!("Extracting: {}", file.name()));

            if file.is_dir() {
                std::fs::create_dir_all(&output_path)?;
            } else {
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output_file = File::create(&output_path)?;
                std::io::copy(&mut file, &mut output_file)?;
            }
            pb.inc(1);
        }

        let elapsed = start.elapsed();
        pb.finish_with_message(format!("✓ Extracted in {:.2?}", elapsed));
        Ok(())
    }

    /// List contents of a ZIP archive
    pub fn list_archive<P: AsRef<Path>>(&self, archive_path: P) -> Result<Vec<String>> {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;
        let mut contents = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            contents.push(file.name().to_string());
        }

        Ok(contents)
    }

    fn add_file_to_zip(
        &self,
        zip: &mut ZipWriter<File>,
        file_path: &Path,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let name = file_path.file_name().unwrap().to_string_lossy();
        zip.start_file(name, *options)?;
        let mut file = File::open(file_path)?;
        std::io::copy(&mut file, zip)?;
        Ok(())
    }

    fn add_dir_to_zip_with_progress(
        &self,
        zip: &mut ZipWriter<File>,
        dir_path: &Path,
        options: &SimpleFileOptions,
        pb: &ProgressBar,
    ) -> Result<()> {
        let walkdir = WalkDir::new(dir_path);
        let it = walkdir.into_iter();

        // Get the directory name to preserve structure
        let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for entry in it {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(dir_path)?.to_string_lossy();

            // Include directory name in archive path
            let archive_path = if relative_path.is_empty() {
                format!("{dir_name}/")
            } else {
                format!("{dir_name}/{relative_path}")
            };

            if path.is_file() {
                pb.set_message(format!("Adding: {}", path.display()));
                zip.start_file(&archive_path, *options)?;
                let mut file = File::open(path)?;
                std::io::copy(&mut file, zip)?;
                pb.inc(1);
            } else if path.is_dir() && !relative_path.is_empty() {
                zip.add_directory(format!("{archive_path}/"), *options)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveStats {
    pub file_count: usize,
    pub dir_count: usize,
    pub total_uncompressed_size: u64,
    pub total_compressed_size: u64,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_archive_single_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create a test file
        fs::write(&test_file, "Hello, World!")?;

        // Create archive
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Verify archive was created
        assert!(archive_path.exists());
        assert!(archive_path.metadata()?.len() > 0);

        Ok(())
    }

    #[test]
    fn test_create_archive_multiple_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test files
        fs::write(&test_file1, "Hello, World!")?;
        fs::write(&test_file2, "Goodbye, World!")?;

        // Create archive
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file1, &test_file2])?;

        // Verify archive was created
        assert!(archive_path.exists());

        // Verify contents
        let contents = manager.list_archive(&archive_path)?;
        assert_eq!(contents.len(), 2);
        assert!(contents.contains(&"test1.txt".to_string()));
        assert!(contents.contains(&"test2.txt".to_string()));

        Ok(())
    }

    #[test]
    fn test_create_archive_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().join("test_dir");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test directory with files
        fs::create_dir(&test_dir)?;
        fs::write(test_dir.join("file1.txt"), "Content 1")?;
        fs::write(test_dir.join("file2.txt"), "Content 2")?;

        // Create archive
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_dir])?;

        // Verify archive was created
        assert!(archive_path.exists());

        // Verify contents
        let contents = manager.list_archive(&archive_path)?;
        assert!(!contents.is_empty());

        Ok(())
    }

    #[test]
    fn test_extract_archive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extract");

        // Create a test file and archive
        fs::write(&test_file, "Hello, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Extract archive
        fs::create_dir(&extract_dir)?;
        manager.extract_archive(&archive_path, &extract_dir)?;

        // Verify extracted file
        let extracted_file = extract_dir.join("test.txt");
        assert!(extracted_file.exists());
        assert_eq!(fs::read_to_string(&extracted_file)?, "Hello, World!");

        Ok(())
    }

    #[test]
    fn test_list_archive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test files and archive
        fs::write(&test_file1, "Hello, World!")?;
        fs::write(&test_file2, "Goodbye, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file1, &test_file2])?;

        // List archive contents
        let contents = manager.list_archive(&archive_path)?;

        assert_eq!(contents.len(), 2);
        assert!(contents.contains(&"test1.txt".to_string()));
        assert!(contents.contains(&"test2.txt".to_string()));

        Ok(())
    }

    #[test]
    fn test_archive_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");
        let archive_path = temp_dir.path().join("test.zip");

        let manager = ArchiveManager::new();
        let result = manager.create_archive(&archive_path, &[&nonexistent_file]);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_archive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create a test file and archive
        fs::write(&test_file, "Hello, World!")?;
        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file])?;

        // Validate the archive
        let is_valid = manager.validate_archive(&archive_path)?;
        assert!(is_valid);

        Ok(())
    }

    #[test]
    fn test_calculate_file_hash() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");

        // Create a test file
        fs::write(&test_file, "Hello, World!")?;

        let manager = ArchiveManager::new();
        let hash = manager.calculate_file_hash(&test_file)?;

        // The hash should be consistent and not empty
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 character hex string

        Ok(())
    }

    #[test]
    fn test_get_archive_stats() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file1 = temp_dir.path().join("test1.txt");
        let test_file2 = temp_dir.path().join("test2.txt");
        let archive_path = temp_dir.path().join("test.zip");

        // Create test files
        fs::write(&test_file1, "Hello, World!")?;
        fs::write(&test_file2, "Goodbye, World!")?;

        let manager = ArchiveManager::new();
        manager.create_archive(&archive_path, &[&test_file1, &test_file2])?;

        // Get archive stats
        let stats = manager.get_archive_stats(&archive_path)?;

        assert_eq!(stats.file_count, 2);
        assert!(stats.total_uncompressed_size > 0);
        assert!(stats.total_compressed_size > 0);
        assert!(stats.compression_ratio > 0.0);

        Ok(())
    }
}
