use pyo3::prelude::*;

pub(super) fn parse_context_json(context_json: &str) -> PyResult<serde_json::Value> {
    serde_json::from_str(context_json)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn serialize_json_result(value: &serde_json::Value) -> PyResult<String> {
    serde_json::to_string(value)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn create_tokio_runtime() -> PyResult<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new().map_err(|error| {
        pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Failed to create Tokio runtime: {error}"
        ))
    })
}
