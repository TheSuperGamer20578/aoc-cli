use pyo3::{pymodule, PyResult, Python};
use pyo3::prelude::PyModule;

pub mod solutions;

macro_rules! submodules {
    {
        module: $m:ident;
        $(
        functions {
            $($function:path),*$(,)?
        }
        )?
        $(
        submodules {
            $($module:path),*$(,)?
        }
        )?
    } => {
        $(
            $(
                $m.add_wrapped(pyo3::wrap_pyfunction!($function))?;
            )*
        )?
        $(
            $(
                let submodule = pyo3::wrap_pymodule!($module);
                $m.add_wrapped(submodule)?;
                // TODO: Make importing work properly
            )*
        )?
    };
}

#[pymodule]
pub fn aoc(_py: Python, m: &PyModule) -> PyResult<()> {
    submodules! {
        module: m;
        functions {
            solutions::solution,
        }
    }

    Ok(())
}
