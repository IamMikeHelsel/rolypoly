use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli_args = rusty::cli::Cli::parse();
    cli_args.run()
}

