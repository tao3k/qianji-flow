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
        task_type::TaskType::Command => Some(Ok(task_mechanisms::command(node_def))),
        task_type::TaskType::WriteFile => Some(Ok(task_mechanisms::write_file(node_def))),
        task_type::TaskType::Suspend => Some(Ok(task_mechanisms::suspend(node_def))),
        _ => None,
    }
}
