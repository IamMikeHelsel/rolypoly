#![cfg(feature = "gui")]

use anyhow::Result;

fn main() -> Result<()> {
    // Run the GUI application when built with the `gui` feature.
    rusty::gui::run_gui().map_err(Into::into)
}
