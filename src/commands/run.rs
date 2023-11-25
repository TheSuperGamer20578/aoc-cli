use std::fs::read_to_string;
use std::sync::RwLock;
use anyhow::Result;
use futures::future::try_join_all;
use glob::glob;
use pyo3::{append_to_inittab, prepare_freethreaded_python, Python};
use pyo3::types::PyModule;
use crate::api;
use crate::Config;
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

pub async fn run(config: &mut Config, year: Option<u16>, day: Option<u8>, part: Option<u8>) -> Result<()> {
    append_to_inittab!(aoc);
    prepare_freethreaded_python();
    let files = glob("./**/*.py")?;
    Python::with_gil(|py| -> Result<()> {
        for file in files {
            let file = file?;
            PyModule::from_code(py, &read_to_string(&file)?, &file.display().to_string(), "__aoc__")?;
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
        config.day_mut(year, day).input = Some(input);
    }

    Python::with_gil(|py| -> Result<()> {
        for (solution, input) in solutions {
            let result = solution.function.call1(py, (input,))?;
            println!("{} day {} part {}: {result}", solution.year, solution.day, solution.part);
        }
        Ok(())
    })?;

    Ok(())
}
