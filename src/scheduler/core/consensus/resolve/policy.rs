pub(super) fn consensus_target_progress(policy: &crate::consensus::ConsensusPolicy) -> f32 {
    use crate::consensus::ConsensusMode;
    match policy.mode {
        ConsensusMode::Majority => 0.5,
        ConsensusMode::Unanimous => 1.0,
        ConsensusMode::Weighted => policy.weight_threshold.clamp(0.0, 1.0),
    }
}
