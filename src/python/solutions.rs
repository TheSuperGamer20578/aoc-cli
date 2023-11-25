use std::sync::RwLock;
use pyo3::{Py, PyAny, pyfunction, pymodule, PyResult, Python, wrap_pyfunction};
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyCFunction, PyModule};
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Solution {
    pub(crate) year: u16,
    pub(crate) day: u8,
    pub(crate) part: u8,
    pub(crate) function: Py<PyAny>,
}

lazy_static! {
    pub static ref SOLUTIONS: RwLock<Vec<Solution>> = RwLock::new(Vec::new());
}

#[pyfunction]
pub(super) fn solution(py: Python, year: u16, day: u8, part: u8) -> PyResult<&PyCFunction> {
    if ![1, 2].contains(&part) {
        return Err(PyTypeError::new_err("Invalid part number: {part}, expected 1 or 2"));
    }

   PyCFunction::new_closure(py, None, None, move |args, kwargs| {
       if kwargs.is_some() {
           return Err(PyTypeError::new_err("Keyword arguments are not supported"));
       }
       if args.len() != 1 {
           return Err(PyTypeError::new_err("Expected 1 positional argument"));
       }

       let function = Py::from(args.get_item(0)?);
       let mut solutions = SOLUTIONS.write().map_err(|_| PyTypeError::new_err("Failed to acquire write lock on SOLUTIONS"))?;
       solutions.push(Solution {
           year,
           day,
           part,
           function,
       });

       Ok(())
   })
}
