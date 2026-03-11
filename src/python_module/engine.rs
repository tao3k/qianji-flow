use std::sync::Arc;

use pyo3::prelude::*;

use crate::engine::QianjiEngine;
use crate::executors::MockMechanism;

/// Python wrapper exposing `QianjiEngine`.
#[pyclass(name = "QianjiEngine")]
pub struct PyQianjiEngine {
    /// Inner Rust engine instance.
    pub inner: QianjiEngine,
}

#[pymethods]
impl PyQianjiEngine {
    /// Creates an empty `QianjiEngine`.
    #[new]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: QianjiEngine::new(),
        }
    }

    /// Adds a mock node for testing from Python.
    /// In production, we'd add real mechanisms like Seeker/Annotator.
    pub fn add_mock_node(&mut self, id: &str, weight: f32) -> usize {
        let id_owned = id.to_string();
        let mech = Arc::new(MockMechanism {
            name: id_owned.clone(),
            weight,
        });
        self.inner.add_mechanism(&id_owned, mech).index()
    }

    /// Adds a directed edge between two node indices.
    pub fn add_link(&mut self, from: usize, to: usize, label: Option<&str>, weight: f32) {
        use petgraph::stable_graph::NodeIndex;
        self.inner
            .add_link(NodeIndex::new(from), NodeIndex::new(to), label, weight);
    }
}

impl Default for PyQianjiEngine {
    fn default() -> Self {
        Self::new()
    }
}
