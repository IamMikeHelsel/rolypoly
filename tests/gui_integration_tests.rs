use anyhow::Result;
use rusty::gui::{create_archive, extract_archive, list_archive, validate_archive, get_archive_stats, calculate_file_hash, CommandResponse};
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_full_archive_workflow_gui() -> Result<()> {
    let (_app, app_handle) = test_helpers::mock_app();
    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    fs::write(&test_file, "GUI integration test")?;

    let archive_path = work_dir.join("gui_archive.zip");

    // 1. Create archive
    let create_response = create_archive(
        app_handle.clone(),
        archive_path.to_str().unwrap().to_string(),
        vec![test_file.to_str().unwrap().to_string()],
    )
    .await;
    assert!(create_response.success);

    // 2. List archive
    let list_response = list_archive(app_handle.clone(), archive_path.to_str().unwrap().to_string()).await;
    assert!(list_response.success);
    assert_eq!(list_response.data.unwrap().len(), 1);

    // 3. Validate archive
    let validate_response = validate_archive(app_handle.clone(), archive_path.to_str().unwrap().to_string()).await;
    assert!(validate_response.success);
    assert!(validate_response.data.unwrap());

    // 4. Get stats
    let stats_response = get_archive_stats(app_handle.clone(), archive_path.to_str().unwrap().to_string()).await;
    assert!(stats_response.success);
    assert_eq!(stats_response.data.unwrap().file_count, 1);

    // 5. Extract archive
    let extract_dir = work_dir.join("extracted");
    fs::create_dir(&extract_dir)?;
    let extract_response = extract_archive(
        app_handle.clone(),
        archive_path.to_str().unwrap().to_string(),
        extract_dir.to_str().unwrap().to_string(),
    )
    .await;
    assert!(extract_response.success);
    assert!(extract_dir.join("test.txt").exists());

    // 6. Calculate hash
    let hash_response = calculate_file_hash(app_handle.clone(), test_file.to_str().unwrap().to_string()).await;
    assert!(hash_response.success);
    assert!(!hash_response.data.unwrap().is_empty());

    Ok(())
}
