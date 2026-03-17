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
        if self.branches.is_empty() {
            return Err("Router has no branches configured".to_string());
        }

        let confidence_bias = confidence_bias(context)?;
        let mut eligible: Vec<(&String, f32)> = Vec::new();
        for (name, weight) in &self.branches {
            let scaled = *weight * confidence_bias;
            if !scaled.is_finite() {
                return Err("Router branch weight produced a non-finite score".to_string());
            }
            if scaled > 0.0 {
                eligible.push((name, scaled));
            }
        }
        if eligible.is_empty() {
            return Err("Router has no positive branch weights".to_string());
        }

        let total_weight: f32 = eligible.iter().map(|(_, w)| *w).sum();
        let mut rng = rand::thread_rng();
        let mut pick = rng.gen_range(0.0..total_weight);
        let mut selected_branch = eligible[0].0.clone();
        for (name, weight) in eligible {
            pick -= weight;
            if pick <= 0.0 {
                selected_branch = name.clone();
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

fn confidence_bias(context: &serde_json::Value) -> Result<f32, String> {
    let raw = context
        .get("omega_confidence")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(1.0);
    let bias = to_f32(raw, "omega_confidence")?;
    if bias <= 0.0 {
        return Err("omega_confidence must be positive".to_string());
    }
    Ok(bias)
}

fn to_f32(value: f64, field: &str) -> Result<f32, String> {
    if !value.is_finite() {
        return Err(format!("{field} must be finite"));
    }
    if value > f64::from(f32::MAX) || value < f64::from(f32::MIN) {
        return Err(format!("{field} must fit within f32 range"));
    }
    Ok(value as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_router_selects_single_branch() {
        let router = ProbabilisticRouter {
            branches: vec![("alpha".to_string(), 1.0)],
        };
        let output = router.execute(&json!({})).await.unwrap();
        assert_eq!(output.data["selected_route"], "alpha");
        match output.instruction {
            FlowInstruction::SelectBranch(branch) => assert_eq!(branch, "alpha"),
            _ => panic!("Expected SelectBranch instruction"),
        }
    }

    #[tokio::test]
    async fn test_router_empty_branches_error() {
        let router = ProbabilisticRouter { branches: vec![] };
        let err = router.execute(&json!({})).await.unwrap_err();
        assert!(err.contains("no branches"));
    }

    #[tokio::test]
    async fn test_router_zero_weight_error() {
        let router = ProbabilisticRouter {
            branches: vec![("alpha".to_string(), 0.0)],
        };
        let err = router.execute(&json!({})).await.unwrap_err();
        assert!(err.contains("no positive"));
    }

    #[tokio::test]
    async fn test_router_invalid_confidence_error() {
        let router = ProbabilisticRouter {
            branches: vec![("alpha".to_string(), 1.0)],
        };
        let err = router
            .execute(&json!({ "omega_confidence": -1.0 }))
            .await
            .unwrap_err();
        assert!(err.contains("omega_confidence"));
    }
}
