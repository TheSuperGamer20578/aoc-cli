#![warn(clippy::pedantic)]

mod api;
mod commands;
mod python;
mod display;
mod value_enum;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::exit;
use clap::{Parser, Subcommand};
use anyhow::{anyhow, bail, Result};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};
use tracing_log::AsTrace;
use crate::api::Submission;
use crate::display::Logger;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    token: Option<String>,
    trusted_dirs: Vec<PathBuf>,
    days: HashMap<u16, HashMap<u8, Day>>,
}

impl Config {
    pub fn get_input(&self, year: u16, day: u8) -> Option<String> {
        self.days.get(&year)?.get(&day)?.input.clone()
    }

    pub fn day(&mut self, year: u16, day: u8) -> &Day {
        self.days
            .entry(year).or_default()
            .entry(day).or_insert(Day::new(year, day))
    }

    pub fn day_mut(&mut self, year: u16, day: u8) -> &mut Day {
        self.days
            .entry(year).or_default()
            .entry(day).or_insert(Day::new(year, day))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Day {
    year: u16,
    day: u8,
    input: Option<String>,
    part1: Part,
    part2: Part,
}

impl Day {
    pub fn new(year: u16, day: u8) -> Self {
        Self {
            year,
            day,
            input: None,
            part1: Part::default(),
            part2: Part::default(),
        }
    }

    pub fn part(&mut self, part: u8) -> &mut Part {
        match part {
            1 => &mut self.part1,
            2 => &mut self.part2,
            _ => panic!("Invalid part number: {part}"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Part {
    status: PartStatus,
    submissions: Vec<Submission>,
}

#[derive(Debug, Serialize, Deserialize)]
enum PartStatus {
    Active {
        min: Option<u128>,
        max: Option<u128>,
        incorrect: Vec<String>,
    },
    Solved(Submission),
}

impl Default for PartStatus {
    fn default() -> Self {
        Self::Active {
            min: None,
            max: None,
            incorrect: Vec::new(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Trusts a directory to contain solutions
    Trust {
        /// The directory to trust
        dir: PathBuf,
    },
    /// Sets the session token to use for submitting solutions and fetching inputs
    Token,
    /// Runs and benchmarks all solutions
    Run {
        /// Only run solutions for the given year
        year: Option<u16>,
        /// Only run solutions for the given day
        day: Option<u8>,
        /// Only run the given part of the solution
        part: Option<u8>,
        /// Submit solutions
        #[clap(long)]
        submit: bool,
        /// Submit known incorrect solutions
        #[clap(long)]
        disable_submit_safety: bool,
    },
    /// Creates a new solution from a template
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
async fn main() {
    if let Err(err) = _main().await {
        error!(root_cause = err.root_cause(), "{err}");
        trace!("Error details:\n\n{err:?}");
        exit(1);
    }
}

async fn _main() -> Result<()> {
    let args = Args::parse();
    Logger::new(args.verbose.log_level_filter().as_trace()).init()?;
    let mut config: Config = confy::load(env!("CARGO_CRATE_NAME"), None)?;
    let cwd = current_dir()?;
    let Some(base_dir) = config.trusted_dirs.iter()
        .find(|dir| cwd.starts_with(dir))
    else {
        if let Command::Trust { dir } = args.command {
            commands::trust(&mut config, &dir)?;
            confy::store(env!("CARGO_CRATE_NAME"), None, config)?;
            return Ok(());
        }
        bail!("Current directory is not trusted. Use `aoc trust <dir>` to trust the current directory.");
    };
    if config.token.is_none() && !matches!(args.command, Command::Token) {
        bail!("No token set. Use `aoc token` to set your session token.");
    }
    match args.command {
        Command::Trust { dir } => commands::trust(&mut config, &dir)?,
        Command::Token => commands::token(&mut config)?,
        Command::Run {
            year,
            day,
            part,
            submit,
            disable_submit_safety
        } => commands::run(&mut config, year, day, part, submit, disable_submit_safety).await?,
        Command::New { .. } => todo!(),
    }
    confy::store(env!("CARGO_CRATE_NAME"), None, config)?;
    Ok(())
}
