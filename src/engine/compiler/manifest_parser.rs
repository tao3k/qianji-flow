use crate::contracts::QianjiManifest;
use crate::error::QianjiError;

pub(super) fn parse(manifest_toml: &str) -> Result<QianjiManifest, QianjiError> {
    toml::from_str(manifest_toml)
        .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))
}
