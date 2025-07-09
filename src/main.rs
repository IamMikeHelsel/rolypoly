mod archive;
mod cli;
mod gui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use std::env;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_gui() {
    // Set up panic handler for GUI mode
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("GUI panic occurred: {panic_info:?}");
        // Log but don't crash the GUI
    }));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gui::create_archive,
            gui::extract_archive,
            gui::list_archive,
            gui::validate_archive,
            gui::get_archive_stats,
            gui::calculate_file_hash,
            gui::get_app_info,
            gui::health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check for GUI mode
    if args.len() <= 1 || args.contains(&"--gui".to_string()) {
        run_gui();
        return Ok(());
    }
    
    // Otherwise, run CLI
    let cli_args = Cli::parse();
    cli_args.run()
}
