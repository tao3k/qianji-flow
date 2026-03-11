//! Probabilistic MDP routing mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use rand::Rng;
use serde_json::json;

/// Mechanism responsible for dynamic probabilistic path selection.
pub struct ProbabilisticRouter {
    /// List of available branches and their relative weights.
    pub branches: Vec<(String, f32)>, // (BranchName, StaticWeight)
}

#[async_trait]
impl QianjiMechanism for ProbabilisticRouter {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let confidence_bias = to_f32(
            context
                .get("omega_confidence")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0),
        );

        let total_weight: f32 = self
            .branches
            .iter()
            .map(|(_, w)| *w * confidence_bias)
            .sum();
        let mut rng = rand::thread_rng();
        let mut pick = rng.gen_range(0.0..total_weight);

        let mut selected_branch = self
            .branches
            .first()
            .map(|(n, _)| n.clone())
            .unwrap_or_default();
        for (name, weight) in &self.branches {
            pick -= *weight * confidence_bias;
            if pick <= 0.0 {
                selected_branch.clone_from(name);
                break;
            }
        }

        Ok(QianjiOutput {
            data: json!({ "selected_route": selected_branch }),
            instruction: FlowInstruction::SelectBranch(selected_branch),
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

#[allow(clippy::cast_possible_truncation)]
fn to_f32(value: f64) -> f32 {
    value as f32
}
