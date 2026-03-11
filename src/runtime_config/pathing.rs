use super::env_vars::env_var_or_override;
use super::model::QianjiRuntimeEnv;
use std::env;
use std::path::{Path, PathBuf};

pub(super) fn resolve_project_root(runtime_env: &QianjiRuntimeEnv) -> PathBuf {
    if let Some(path) = &runtime_env.prj_root {
        return path.clone();
    }
    if let Some(raw) = env_var_or_override(runtime_env, "PRJ_ROOT") {
        return PathBuf::from(raw);
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub(super) fn resolve_prj_config_home(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> PathBuf {
    if let Some(path) = &runtime_env.prj_config_home {
        return path.clone();
    }

    if let Some(raw) = env_var_or_override(runtime_env, "PRJ_CONFIG_HOME") {
        let path = PathBuf::from(raw);
        if path.is_absolute() {
            return path;
        }
        return project_root.join(path);
    }

    project_root.join(".config")
}

pub(super) fn runtime_env_has_path_overrides(runtime_env: &QianjiRuntimeEnv) -> bool {
    runtime_env.prj_root.is_some()
        || runtime_env.prj_config_home.is_some()
        || runtime_env.qianji_config_path.is_some()
        || runtime_env.extra_env.iter().any(|(key, _)| {
            matches!(
                key.as_str(),
                "PRJ_ROOT" | "PRJ_CONFIG_HOME" | "QIANJI_CONFIG_PATH"
            )
        })
}
