use std::fs::read_to_string;
use std::sync::RwLock;
use anyhow::Result;
use futures::future::try_join_all;
use glob::glob;
use indicatif::ProgressIterator;
use pyo3::{append_to_inittab, prepare_freethreaded_python, Python};
use pyo3::types::PyModule;
use tracing::{error, warn};
use crate::{api, PartStatus};
use crate::api::{SubmitResult, WrongAnswerReason};
use crate::Config;
use crate::display::{ActionType, confirm, FormatTraceBack, println, progress_bar};
use crate::python::aoc;
use crate::python::solutions::{Solution, SOLUTIONS};

async fn get_input<'s>(config: &Config, solution: &'s Solution, new_inputs: &RwLock<Vec<(u16, u8, String)>>) -> Result<(&'s Solution, String)> {
    let input = if let Some(input) = config.get_input(solution.year, solution.day) { input } else {
        let input = api::get_input(config, solution.year, solution.day).await?;
        new_inputs.write().unwrap().push((solution.year, solution.day, input.clone()));
        input
    };
    Ok((solution, input))
}

async fn handle_submit(config: &mut Config, solution: &Solution, identifier: &String, result: &String) -> Result<()> {
    let submit_result = api::submit(config, solution.year, solution.day, solution.part, result).await?;
    match &submit_result.result {
        SubmitResult::Accepted => {
            println("Solved", ActionType::Success, identifier);
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::TooHigh) => {
            println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, too high"));
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::TooLow) => {
            println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, too low"));
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::None) => {
            println("Incorrect", ActionType::Failure, format!("{identifier}: {result}"));
        }
        SubmitResult::TooSoon(retry_in) => {
            println("Too Soon", ActionType::Error, format!("{identifier}: You have submitted too recently, please retry in {retry_in}"));
        }
        SubmitResult::Invalid => {
            println("Invalid", ActionType::Error, format!("{identifier}: You don't seem to be solving the right level"));
        }
        SubmitResult::Unknown(message) => {
            println("Unknown", ActionType::Error, format!("{identifier}: {message}"));
        }
    }
    Ok(())
}

async fn handle_result(config: &mut Config, solution: &Solution, identifier: &String, result: &String, submit: bool, disable_submit_safety: bool) -> Result<()> {
    match &config.day(solution.year, solution.day).part(solution.part).status {
        PartStatus::Active { min, max, incorrect } => {
            if incorrect.contains(result) && !disable_submit_safety {
                println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, already tried"));
            } else if min.is_some() && !disable_submit_safety {
                let min = min.unwrap();
                if result.parse::<i64>()? < min {
                    println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, must be greater than {min}"));
                }
            } else if max.is_some() && !disable_submit_safety {
                let max = max.unwrap();
                if result.parse::<i64>()? > max {
                    println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, must be less than {max}"));
                }
            } else if submit && confirm(format!("Submit {result} for {identifier}?"))? {
                handle_submit(config, solution, identifier, result).await?;
            } else {
                println("Run", ActionType::Success, format!("{identifier}: {result}"));
            }
        }
        PartStatus::Solved(answer) => {
            if answer.answer == *result {
                println("Solved", ActionType::Success, identifier);
            } else {
                println("Incorrect", ActionType::Failure, format!("{identifier}: {result}, expected {}", answer.answer));
            }
        }
    }
    Ok(())
}

pub async fn run(config: &mut Config, year: Option<u16>, day: Option<u8>, part: Option<u8>, submit: bool, disable_submit_safety: bool) -> Result<()> {
    append_to_inittab!(aoc);
    prepare_freethreaded_python();
    let files: Vec<_> = glob("./**/*.py")?.collect();
    let mut import_failures = 0u16;
    let bar = progress_bar("Importing".to_string(), ActionType::Prepare, files.len() as u64)?;
    Python::with_gil(|py| -> Result<()> {
        for (i, file) in files.into_iter().enumerate().progress_with(bar.clone()) {
            let file = file?;
            bar.set_message(file.display().to_string());
            if let Err(error) = PyModule::from_code(py, &read_to_string(&file)?, &file.display().to_string(), &format!("aoc_{i}")).tb() {
                error!("Failed to import {}\n\n{error}", file.display());
                import_failures += 1;
            }
        }
        Ok(())
    })?;

    let new_inputs: RwLock<Vec<(u16, u8, String)>> = RwLock::new(Vec::new());
    let solutions: Vec<_> = {
        let solutions = SOLUTIONS.read().unwrap();
        solutions.iter()
            .filter(|solution| {
                if let Some(year) = year {
                    if solution.year != year {
                        return false;
                    }
                }
                if let Some(day) = day {
                    if solution.day != day {
                        return false;
                    }
                }
                if let Some(part) = part {
                    if solution.part != part {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    };
    let solutions = try_join_all(solutions.iter()
        .map(|solution| get_input(config, solution, &new_inputs))
    ).await?;
    for (year, day, input) in new_inputs.into_inner().unwrap() {
        config.day(year, day).input = Some(input);
    }

    let mut failures = 0u16;
    let mut skips = 0u16;
    let bar = progress_bar("Running".to_string(), ActionType::Progress, solutions.len() as u64)?;
    for (solution, input) in solutions {
        let (identifier, result) = Python::with_gil(|py| -> Result<_> {
            let identifier = format!("{} day {} part {} ({})", solution.year, solution.day, solution.part, solution.function.getattr(py, "__name__")?);
            bar.set_message(identifier.clone());
            let result = match solution.function.call1(py, (input, )).tb() {
                Ok(result) => result,
                Err(error) => {
                    error!("{identifier}: Failed to run solution:\n\n{error}");
                    failures += 1;
                    return Ok((identifier, None));
                }
            };
            let result: String = result.getattr(py, "__str__")?.call0(py)?.extract(py)?;
            Ok((identifier, Some(result)))
        })?;
        let Some(result) = result else {
            println(
                "Skipped",
                ActionType::Warning,
                identifier,
            );
            skips += 1;
            continue;
        };
        handle_result(config, solution, &identifier, &result, submit, disable_submit_safety).await?;
        bar.inc(1);
    }
    if import_failures > 0 {
        warn!("{import_failures} solution{} failed to import", if import_failures == 1 {""} else {"s"});
    }
    if failures > 0 {
        warn!("{failures} solution{} failed", if failures == 1 {""} else {"s"});
    }
    let skips = skips - failures;
    if skips > 0 {
        warn!("{skips} solution{} were skipped", if skips == 1 {""} else {"s"});
    }

    Ok(())
}
