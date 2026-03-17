use crate::contracts::QianjiMechanism;
use crate::error::QianjiError;
use std::sync::Arc;

#[cfg(feature = "llm")]
use crate::engine::compiler::llm_client;
use crate::engine::compiler::mechanism_dispatch::resolver_chain;
#[cfg(feature = "llm")]
use crate::engine::compiler::stateful_mechanisms;

#[cfg(feature = "llm")]
pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let resolver_chain::DispatchContext {
        compiler, node_def, ..
    } = context;
    let client = llm_client::resolve_for_node(node_def, compiler.llm_client.clone())?;
    Ok(stateful_mechanisms::llm(
        &compiler.orchestrator,
        &compiler.registry,
        node_def,
        client,
    ))
}

#[cfg(not(feature = "llm"))]
pub(super) fn build(
    _context: resolver_chain::DispatchContext<'_>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    Err(QianjiError::Topology(
        "Task type 'llm' requires enabling feature 'llm' for xiuxian-qianji".to_string(),
    ))
}
