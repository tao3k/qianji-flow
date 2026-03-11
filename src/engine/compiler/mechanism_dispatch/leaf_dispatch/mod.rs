use crate::error::QianjiError;

use super::resolver_chain;

mod io_control;
mod quality_guard;
mod wendao_router;

const LEAF_RESOLVERS: [resolver_chain::ResolverFn; 3] = [
    io_control::build,
    quality_guard::build,
    wendao_router::build,
];

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    resolver_chain::run(&LEAF_RESOLVERS, context).or_else(|| {
        let task_type = context.task_type;
        Some(Err(QianjiError::Topology(format!(
            "Internal dispatch mismatch for leaf task routing: {task_type:?}"
        ))))
    })
}
