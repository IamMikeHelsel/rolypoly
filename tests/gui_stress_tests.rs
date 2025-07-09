use anyhow::Result;
use rusty::gui::{create_archive, list_archive, CommandResponse};
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_large_number_of_files_stress() -> Result<()> {
    let (_app, app_handle) = test_helpers::mock_app();
    let temp_dir = TempDir::new()?;
    let work_dir = temp_dir.path();
    let archive_path = work_dir.join("stress_test.zip");

    let mut files_to_add = Vec::new();
    for i in 0..1000 {
        let file_path = work_dir.join(format!("file_{}.txt", i));
        fs::write(&file_path, format!("content {}", i))?;
        files_to_add.push(file_path.to_str().unwrap().to_string());
    }

    let response = create_archive(
        app_handle.clone(),
        archive_path.to_str().unwrap().to_string(),
        files_to_add,
    )
    .await;

    assert!(response.success);

    let list_response = list_archive(app_handle, archive_path.to_str().unwrap().to_string()).await;
    assert!(list_response.success);
    assert_eq!(list_response.data.unwrap().len(), 1000);

    Ok(())
}
