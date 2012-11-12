/* This file exists just to make it easier to import platform-specific
  implementations.
 
Note that you still must define each of the files as a module in
servo_gfx.rc. This is not ideal and may be changed in the future. */

pub use font::FontHandle;
pub use font_context::FontContextHandle;
pub use font_list::FontListHandle;
