use crate::contracts::NodeDefinition;
use crate::error::QianjiError;

pub(super) fn branches(node_def: &NodeDefinition) -> Result<Vec<(String, f32)>, QianjiError> {
    let mut branches = Vec::new();
    if let Some(branches_config) = node_def.params["branches"].as_array() {
        for item in branches_config {
            let Some(branch) = item.as_array() else {
                continue;
            };
            let Some(name) = branch.first().and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(weight) = branch.get(1) else {
                continue;
            };
            branches.push((name.to_string(), branch_weight(weight)?));
        }
    }
    Ok(branches)
}

fn branch_weight(weight: &serde_json::Value) -> Result<f32, QianjiError> {
    let weight = serde_json::from_value::<f32>(weight.clone()).map_err(|_error| {
        QianjiError::Topology(
            "Router branch weight must be a finite number within f32 range".to_string(),
        )
    })?;
    if !weight.is_finite() {
        return Err(QianjiError::Topology(
            "Router branch weight must be a finite number".to_string(),
        ));
    }
    Ok(weight)
}
