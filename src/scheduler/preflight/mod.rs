//! Pre-execution context preflight for semantic placeholder resolution.

mod context_path;
mod mounts;
mod query;
mod semantic;
mod wendao_uri;

pub(crate) use context_path::{context_value_to_text, lookup_context_path};
pub(crate) use mounts::{RuntimeWendaoMount, install_runtime_wendao_mounts};
pub(crate) use semantic::{
    resolve_semantic_content, resolve_semantic_reference, resolve_wendao_placeholders_in_context,
};
pub(crate) use wendao_uri::resolve_wendao_uri_with_zhenfa;
