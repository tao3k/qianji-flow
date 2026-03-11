use include_dir::Dir;
use std::sync::{OnceLock, RwLock};

static RUNTIME_WENDAO_MOUNTS: OnceLock<RwLock<Vec<RuntimeWendaoMount>>> = OnceLock::new();

/// Runtime mount descriptor used by semantic URI resolution hooks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeWendaoMount {
    /// Semantic skill name (host segment in `wendao://skills/<name>/...`).
    pub(crate) semantic_name: &'static str,
    /// Relative references root inside mounted embedded directory.
    pub(crate) references_dir: &'static str,
    /// Embedded directory providing referenced resources.
    pub(crate) dir: &'static Dir<'static>,
}

/// RAII guard that restores previous runtime mount registry on drop.
pub(crate) struct RuntimeWendaoMountGuard {
    previous: Vec<RuntimeWendaoMount>,
}

impl Drop for RuntimeWendaoMountGuard {
    fn drop(&mut self) {
        if let Ok(mut slot) = runtime_wendao_mounts().write() {
            *slot = std::mem::take(&mut self.previous);
        }
    }
}

/// Installs runtime mounts for this execution scope.
pub(crate) fn install_runtime_wendao_mounts(
    mounts: Vec<RuntimeWendaoMount>,
) -> RuntimeWendaoMountGuard {
    if let Ok(mut slot) = runtime_wendao_mounts().write() {
        let previous = std::mem::replace(&mut *slot, mounts);
        return RuntimeWendaoMountGuard { previous };
    }
    RuntimeWendaoMountGuard {
        previous: Vec::new(),
    }
}

pub(super) fn runtime_wendao_mounts() -> &'static RwLock<Vec<RuntimeWendaoMount>> {
    RUNTIME_WENDAO_MOUNTS.get_or_init(|| RwLock::new(Vec::new()))
}
