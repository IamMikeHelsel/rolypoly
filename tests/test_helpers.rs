use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).unwrap();
    file_path
}

pub fn create_test_archive(dir: &TempDir, files: &[(&str, &str)]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.zip");
    
    // Create test files
    let mut file_paths = Vec::new();
    for (name, content) in files {
        let file_path = create_test_file(dir, name, content);
        file_paths.push(file_path);
    }
    
    // Create archive using our CLI
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "create", archive_path.to_str().unwrap()])
        .args(file_paths.iter().map(|p| p.to_str().unwrap()))
        .output()
        .expect("Failed to create test archive");
    
    if !output.status.success() {
        panic!("Failed to create test archive: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    archive_path
}

pub fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<(), String> {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "extract", 
               archive_path.to_str().unwrap(), 
               "-o", output_dir.to_str().unwrap()])
        .output()
        .expect("Failed to extract archive");
    
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn list_archive_contents(archive_path: &Path) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "list", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to list archive");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn validate_archive(archive_path: &Path) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "validate", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to validate archive");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[allow(dead_code)]
pub fn get_archive_stats(archive_path: &Path) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "stats", archive_path.to_str().unwrap()])
        .output()
        .expect("Failed to get archive stats");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn calculate_file_hash(file_path: &Path) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--bin", "rusty", "--", "hash", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to calculate hash");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}