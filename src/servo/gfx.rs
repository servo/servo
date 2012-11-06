/* This file exists just to make it easier to import things inside of
 ./gfx/ without specifying the file they came out of imports.
 
Note that you still must define each of the files as a module in
servo.rc. This is not ideal and may be changed in the future. */

// shortcut names
pub use au = geometry;
pub use dl = display_list;

pub use display_list::DisplayList;
pub use font::Font;
pub use font_cache::FontCache;
pub use font_context::FontContext;
pub use font_matcher::FontMatcher;
pub use geometry::Au;
pub use native_font::NativeFont;
pub use shaper::Shaper;
pub use text_run::TextRun;
pub use text_run::SendableTextRun;

pub use render_context::RenderContext;
pub use render_layers::RenderLayer;