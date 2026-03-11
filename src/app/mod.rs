//! Application-layer scheduler factories and built-in pipeline presets.

mod build;
mod presets;
mod qianji_app;

pub use presets::{MEMORY_PROMOTION_PIPELINE_TOML, RESEARCH_TRINITY_TOML};
pub use qianji_app::QianjiApp;
