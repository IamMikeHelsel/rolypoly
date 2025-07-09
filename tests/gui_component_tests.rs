use anyhow::Result;
use rusty::archive::ArchiveManager;
use rusty::gui::{create_archive, CommandResponse};
use std::fs;
use tempfile::TempDir;

mod test_helpers;

#[tokio::test]
async fn test_create_archive_component() -> Result<()> {
    let (_app, app_handle) = test_helpers::mock_app();
    let temp_dir = TempDir::new()?;
    let file_to_compress = temp_dir.path().join("test_file.txt");
    fs::write(&file_to_compress, "Hello from GUI test!")?;

    let archive_path = temp_dir.path().join("gui_test.zip");

    let response = create_archive(
        app_handle,
        archive_path.to_str().unwrap().to_string(),
        vec![file_to_compress.to_str().unwrap().to_string()],
    )
    .await;

    assert!(response.success);
    assert!(archive_path.exists());
    Ok(())
}