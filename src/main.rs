mod archive;
mod cli;
mod gui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use std::env;

pub fn run_gui() -> Result<()> {
    // Set up panic handler for GUI mode
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("GUI panic occurred: {panic_info:?}");
        // Log but don't crash the GUI
    }));

    gui::run_gui()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check for GUI mode
    if args.len() <= 1 || args.contains(&"--gui".to_string()) {
        return run_gui();
    }

    // Otherwise, run CLI
    let cli_args = Cli::parse();
    cli_args.run()
}
