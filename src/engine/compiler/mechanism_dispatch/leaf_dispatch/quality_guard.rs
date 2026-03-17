use crate::engine::compiler::mechanism_dispatch::resolver_chain;
use crate::engine::compiler::{task_mechanisms, task_type};

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    let resolver_chain::DispatchContext {
        task_type,
        node_def,
        ..
    } = context;
    match task_type {
        task_type::TaskType::Calibration => Some(Ok(task_mechanisms::calibration(node_def))),
        task_type::TaskType::Mock => Some(Ok(task_mechanisms::mock(node_def))),
        task_type::TaskType::SecurityScan => Some(Ok(task_mechanisms::security_scan(node_def))),
        _ => None,
    }
}
