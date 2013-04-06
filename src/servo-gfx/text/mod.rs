//! This file exists just to make it easier to import things inside of
/// ./text/ without specifying the file they came out of imports.

pub use text::shaping::Shaper;
pub use text::text_run::SendableTextRun;
pub use text::text_run::TextRun;

pub mod glyph;
#[path="shaping/mod.rs"] pub mod shaping;
pub mod text_run;
pub mod util;

