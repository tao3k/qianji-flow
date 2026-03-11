use super::env_vars::env_var_or_override;
use super::model::QianjiRuntimeEnv;
use super::pathing::runtime_env_has_path_overrides;
use super::toml_config::{QianjiToml, apply_llm_overlay, apply_memory_promotion_overlay};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use xiuxian_macros::project_config_paths;

pub(super) fn load_qianji_toml(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
    config_home: &Path,
) -> io::Result<QianjiToml> {
    let mut merged = QianjiToml::default();

    let candidates = if runtime_env_has_path_overrides(runtime_env) {
        let mut manual_candidates = vec![
            project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
            config_home.join("xiuxian-artisan-workshop/xiuxian.toml"),
            config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        ];
        if let Some(explicit) = runtime_env
            .qianji_config_path
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_CONFIG_PATH").map(PathBuf::from))
        {
            manual_candidates.push(explicit);
        }
        manual_candidates
    } else {
        project_config_paths!("qianji.toml", "QIANJI_CONFIG_PATH")
    };

    for path in candidates {
        if !path.exists() {
            continue;
        }
        let parsed = read_qianji_toml_file(&path)?;
        apply_llm_overlay(&mut merged.llm, parsed.llm);
        apply_memory_promotion_overlay(&mut merged.memory_promotion, parsed.memory_promotion);
    }

    Ok(merged)
}

fn read_qianji_toml_file(path: &Path) -> io::Result<QianjiToml> {
    let raw = fs::read_to_string(path).map_err(|e| {
        io::Error::other(format!(
            "failed to read qianji config {}: {e}",
            path.display()
        ))
    })?;
    toml::from_str::<QianjiToml>(&raw).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse qianji config {}: {e}", path.display()),
        )
    })
}
