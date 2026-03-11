//! Qianji Graphical Style (QGS) Constants
//! Inspired by `PaperBanana` academic aesthetics.

/// Default background color for the layout.
pub const BG_COLOR: &str = "#F9FAFB";
/// Default border color for nodes and edges.
pub const BORDER_COLOR: &str = "#374151";
/// Background color for standard Task nodes.
pub const TASK_BG: &str = "#EFF6FF";
/// Background color for Gateway nodes.
pub const GATEWAY_BG: &str = "#FFFBEB";
/// Default font family for labels.
pub const FONT_FAMILY: &str = "Inter, sans-serif";
/// Default stroke width for lines and borders.
pub const STROKE_WIDTH: f32 = 1.5;

/// Represents a graphical theme for Qianji visualization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QgsTheme {
    /// Global background color.
    pub background: String,
    /// Global border/stroke color.
    pub border: String,
    /// Fill color for Task-type nodes.
    pub task_fill: String,
    /// Fill color for Gateway-type nodes.
    pub gateway_fill: String,
    /// Font family used in the diagram.
    pub font_family: String,
    /// Stroke width for all elements.
    pub stroke_width: f32,
}

impl Default for QgsTheme {
    fn default() -> Self {
        Self {
            background: BG_COLOR.into(),
            border: BORDER_COLOR.into(),
            task_fill: TASK_BG.into(),
            gateway_fill: GATEWAY_BG.into(),
            font_family: FONT_FAMILY.into(),
            stroke_width: STROKE_WIDTH,
        }
    }
}
