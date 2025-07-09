use tauri::{App, AppHandle};

// This function requires the `tauri/test` feature.
pub fn mock_app() -> (App, AppHandle) {
    let app = tauri::test::mock_app();
    let handle = app.handle().clone();
    (app, handle)
}
