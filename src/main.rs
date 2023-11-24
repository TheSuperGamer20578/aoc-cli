use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs and benchmarks all solutions
    #[clap(name = "run")]
    Run {
        /// Only run solutions for the given year
        year: Option<u16>,
        /// Only run solutions for the given day
        day: Option<u8>,
        /// Only run the given part of the solution
        part: Option<u8>,
    },
    /// Creates a new solution from a template
    #[clap(name = "new")]
    New {
        /// The template to use for the new solution
        template: PathBuf,
        /// The year to create a new solution for
        year: u16,
        /// The day to create a new solution for
        day: u8,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    Ok(())
}
