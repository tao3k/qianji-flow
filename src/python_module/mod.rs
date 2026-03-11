//! PyO3 module surface for `xiuxian-qianji`.

use pyo3::prelude::*;

mod engine;
#[cfg(feature = "llm")]
mod llm_bridge;
mod runtime;
mod scheduler;

use engine::PyQianjiEngine;
#[cfg(feature = "llm")]
use llm_bridge::run_master_research_array;
use scheduler::PyQianjiScheduler;

#[pymodule]
fn _xiuxian_qianji(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyQianjiEngine>()?;
    module.add_class::<PyQianjiScheduler>()?;
    #[cfg(feature = "llm")]
    module.add_function(wrap_pyfunction!(run_master_research_array, module)?)?;
    Ok(())
}
