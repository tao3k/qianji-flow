use crate::contracts::QianjiMechanism;
use crate::error::QianjiError;
use std::sync::Arc;

use crate::engine::compiler::mechanism_dispatch::resolver_chain;
use crate::engine::compiler::stateful_mechanisms;
#[cfg(feature = "llm")]
use crate::engine::compiler::{formal_audit as formal_audit_cfg, llm_client};

#[cfg(feature = "llm")]
pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let resolver_chain::DispatchContext {
        compiler, node_def, ..
    } = context;
    if formal_audit_cfg::uses_llm_controller(node_def) {
        let client = llm_client::resolve_for_node(node_def, compiler.llm_client.clone())?;
        return stateful_mechanisms::formal_audit_with_llm(
            &compiler.orchestrator,
            &compiler.registry,
            node_def,
            client,
        );
    }
    Ok(stateful_mechanisms::formal_audit_native(node_def))
}

#[cfg(not(feature = "llm"))]
pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let resolver_chain::DispatchContext { node_def, .. } = context;
    stateful_mechanisms::formal_audit_requires_llm_guard(node_def)?;
    Ok(stateful_mechanisms::formal_audit_native(node_def))
}
