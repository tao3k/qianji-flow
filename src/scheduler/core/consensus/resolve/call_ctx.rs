pub(super) struct ConsensusCallCtx<'a> {
    pub(super) manager: &'a crate::consensus::ConsensusManager,
    pub(super) session_id: &'a str,
    pub(super) node_id: &'a str,
    pub(super) output_hash: &'a str,
    pub(super) output_data: &'a serde_json::Value,
    pub(super) telemetry_target: Option<f32>,
}
