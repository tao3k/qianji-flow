use super::super::task_type;
use super::resolver_chain;

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
