use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn run_command(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Command failed: cargo run -- {}", args.join(" "));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command failed");
    }

    String::from_utf8(output.stdout).expect("Invalid UTF-8 output")
}

#[test]
fn test_json_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");
    let archive_path = temp_dir.path().join("test.zip");
    let extract_dir = temp_dir.path().join("extract");

    // Create a test file
    fs::write(&test_file, "Hello, World!").expect("Failed to write test file");

    // Test Create
    let args = [
        "create",
        archive_path.to_str().unwrap(),
        test_file.to_str().unwrap(),
        "--json"
    ];
    let output = run_command(&args);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let created_line = lines.iter()
        .find(|line| line.contains(r#""event":"created""#))
        .expect("Failed to find created event");

    let json: serde_json::Value = serde_json::from_str(created_line).expect("Failed to parse JSON");
    assert_eq!(json["event"], "created");
    assert!(json["archive"].as_str().unwrap().contains("test.zip"));

    // Test List
    let args = ["list", archive_path.to_str().unwrap(), "--json"];
    let output = run_command(&args);
    // Find line with "files"
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let list_line = lines.iter()
        .find(|line| line.contains(r#""files":["#))
        .expect("Failed to find list output");

    let json: serde_json::Value = serde_json::from_str(list_line).expect("Failed to parse JSON");
    assert!(json["archive"].as_str().unwrap().contains("test.zip"));
    let files = json["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f.as_str().unwrap() == "test.txt"));

    // Test Validate
    let args = ["validate", archive_path.to_str().unwrap(), "--json"];
    let output = run_command(&args);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let valid_line = lines.iter()
        .find(|line| line.contains(r#""valid":true"#))
        .expect("Failed to find validate result");

    let json: serde_json::Value = serde_json::from_str(valid_line).expect("Failed to parse JSON");
    assert_eq!(json["valid"], true);

    // Test Extract
    let args = [
        "extract",
        archive_path.to_str().unwrap(),
        "--output",
        extract_dir.to_str().unwrap(),
        "--json"
    ];
    let output = run_command(&args);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let extracted_line = lines.iter()
        .find(|line| line.contains(r#""event":"extracted""#))
        .expect("Failed to find extracted event");

    let json: serde_json::Value = serde_json::from_str(extracted_line).expect("Failed to parse JSON");
    assert_eq!(json["event"], "extracted");
    assert!(json["output"].as_str().unwrap().contains("extract"));

    // Test Hash
    let args = ["hash", test_file.to_str().unwrap(), "--json"];
    let output = run_command(&args);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let hash_line = lines.iter()
        .find(|line| line.contains(r#""hash":""#))
        .expect("Failed to find hash result");

    let json: serde_json::Value = serde_json::from_str(hash_line).expect("Failed to parse JSON");
    assert_eq!(json["algo"], "sha256");
    assert!(json["file"].as_str().unwrap().contains("test.txt"));
    assert_eq!(json["hash"].as_str().unwrap().len(), 64);
}
