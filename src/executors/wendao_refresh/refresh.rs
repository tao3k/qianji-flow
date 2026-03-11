use std::path::{Path, PathBuf};
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::LinkGraphRefreshMode;

pub(super) struct RefreshExecution {
    pub(super) mode: LinkGraphRefreshMode,
    pub(super) fallback: bool,
    pub(super) threshold: usize,
}

pub(super) fn build_index(
    root_dir: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> Result<LinkGraphIndex, String> {
    match LinkGraphIndex::build_with_cache(root_dir, include_dirs, excluded_dirs) {
        Ok(index) => Ok(index),
        Err(error) if include_dirs.is_empty() && excluded_dirs.is_empty() => {
            log::warn!(
                "qianji wendao_refresh cache bootstrap failed, fallback to build(): {error}"
            );
            LinkGraphIndex::build(root_dir)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn execute_refresh(
    index: &mut LinkGraphIndex,
    changed_paths: &[PathBuf],
    force_full: bool,
    prefer_incremental: bool,
    allow_full_fallback: bool,
    full_rebuild_threshold: Option<usize>,
) -> Result<RefreshExecution, String> {
    let mut fallback = false;
    let threshold = if prefer_incremental {
        usize::MAX
    } else {
        full_rebuild_threshold
            .unwrap_or_else(LinkGraphIndex::incremental_rebuild_threshold)
            .max(1)
    };

    let mode = if force_full {
        run_forced_full_refresh(index, changed_paths)?
    } else {
        match index.refresh_incremental_with_threshold(changed_paths, threshold) {
            Ok(mode) => mode,
            Err(error) if allow_full_fallback => {
                fallback = true;
                log::warn!(
                    "qianji wendao_refresh incremental failed, fallback to full rebuild: {error}"
                );
                run_forced_full_refresh(index, changed_paths)?
            }
            Err(error) => {
                return Err(format!(
                    "wendao_refresh incremental failed without fallback: {error}"
                ));
            }
        }
    };

    Ok(RefreshExecution {
        mode,
        fallback,
        threshold,
    })
}

fn run_forced_full_refresh(
    index: &mut LinkGraphIndex,
    changed_paths: &[PathBuf],
) -> Result<LinkGraphRefreshMode, String> {
    let force_paths = if changed_paths.is_empty() {
        vec![PathBuf::from("__qianji_force_full__.md")]
    } else {
        changed_paths.to_vec()
    };
    index.refresh_incremental_with_threshold(&force_paths, 1)
}

pub(super) fn refresh_mode_label(mode: LinkGraphRefreshMode) -> &'static str {
    match mode {
        LinkGraphRefreshMode::Noop => "noop",
        LinkGraphRefreshMode::Delta => "delta",
        LinkGraphRefreshMode::Full => "full",
    }
}
