use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli_args = rolypoly::cli::Cli::parse();
    cli_args.run()
}

