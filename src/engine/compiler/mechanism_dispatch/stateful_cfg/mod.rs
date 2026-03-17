use super::resolver_chain;
use crate::engine::compiler::task_type;

mod formal_audit;
mod llm;

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    match context.task_type {
        task_type::TaskType::FormalAudit => Some(formal_audit::build(context)),
        task_type::TaskType::Llm => Some(llm::build(context)),
        _ => None,
    }
}
