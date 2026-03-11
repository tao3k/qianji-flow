use crate::engine::QianjiEngine;
use crate::error::QianjiError;

pub(super) fn ensure_static_acyclic(engine: &QianjiEngine) -> Result<(), QianjiError> {
    if petgraph::algo::is_cyclic_directed(&engine.graph) {
        return Err(QianjiError::Topology(
            "Manifest contains a static cycle".to_string(),
        ));
    }
    Ok(())
}
