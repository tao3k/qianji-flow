pub(in crate::scheduler::core) enum RemoteDelegationOutcome {
    Noop,
    Progressed,
    Suspend(serde_json::Value),
}
