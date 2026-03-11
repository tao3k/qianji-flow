use pyo3::prelude::*;

use crate::engine::QianjiEngine;
use crate::scheduler::QianjiScheduler;

use super::engine::PyQianjiEngine;
use super::runtime::{create_tokio_runtime, parse_context_json, serialize_json_result};

/// Python wrapper exposing `QianjiScheduler`.
#[pyclass(name = "QianjiScheduler")]
pub struct PyQianjiScheduler {
    /// Inner Rust scheduler instance.
    pub inner: QianjiScheduler,
}

#[pymethods]
impl PyQianjiScheduler {
    /// Creates a scheduler from an existing engine.
    #[new]
    #[must_use]
    pub fn new(engine: &PyQianjiEngine) -> Self {
        // Cloning the engine into the scheduler.
        // In a real scenario, we might want to share ownership better.
        Self {
            inner: QianjiScheduler::new(QianjiEngine {
                graph: engine.inner.graph.clone(),
            }),
        }
    }

    /// Runs the scheduler asynchronously from Python.
    ///
    /// # Errors
    ///
    /// Returns `PyValueError` for invalid JSON payload and `PyRuntimeError`
    /// for runtime creation or scheduler execution failures.
    pub fn run(&self, py: Python<'_>, context_json: &str) -> PyResult<String> {
        let context = parse_context_json(context_json)?;
        py.detach(|| {
            let runtime = create_tokio_runtime()?;
            let result = runtime
                .block_on(self.inner.run(context))
                .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
            serialize_json_result(&result)
        })
    }
}

impl Default for PyQianjiScheduler {
    fn default() -> Self {
        Self {
            inner: QianjiScheduler::new(QianjiEngine::new()),
        }
    }
}
