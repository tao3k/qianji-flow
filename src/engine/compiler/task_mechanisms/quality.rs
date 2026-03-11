use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::executors::calibration::SynapseCalibrator;
use std::sync::Arc;

use super::super::{calibration, security_scan};

pub(in crate::engine::compiler) fn calibration(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    Arc::new(SynapseCalibrator {
        target_node_id: calibration::target_node_id(node_def),
        drift_threshold: 0.5,
    })
}

pub(in crate::engine::compiler) fn mock(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
    Arc::new(crate::executors::MockMechanism {
        name: node_def.id.clone(),
        weight: node_def.weight,
        output_key: node_def
            .params
            .get("output_key")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        mock_output: node_def.params.get("mock_output").cloned(),
    })
}

pub(in crate::engine::compiler) fn security_scan(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = security_scan::mechanism_config(node_def);
    Arc::new(crate::executors::security_scan::SecurityScanMechanism {
        files_key: cfg.files_key,
        output_key: cfg.output_key,
        abort_on_violation: cfg.abort_on_violation,
        cwd_key: cfg.cwd_key,
    })
}
