/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[crate_id = "github.com/mozilla/servo#gfx:0.1"];
#[crate_type = "lib"];

#[feature(globs, managed_boxes, macro_rules)];

extern mod azure;
extern mod extra;
extern mod geom;
extern mod layers;
extern mod stb_image;
extern mod png;
extern mod servo_net = "net";
extern mod servo_util = "util";
extern mod style;
extern mod servo_msg = "msg";

// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, this is a hard dependency.
extern mod harfbuzz;

// Linux and Android-specific library dependencies
#[cfg(target_os="linux")] #[cfg(target_os="android")] extern mod fontconfig;
#[cfg(target_os="linux")] #[cfg(target_os="android")] extern mod freetype;

// Mac OS-specific library dependencies
#[cfg(target_os="macos")] extern mod core_foundation;
#[cfg(target_os="macos")] extern mod core_graphics;
#[cfg(target_os="macos")] extern mod core_text;

pub use gfx_font = font;
pub use gfx_font_context = font_context;
pub use gfx_font_list = font_list;
pub use servo_gfx_font = font;
pub use servo_gfx_font_list = font_list;

// Macros
mod macros;

// Private rendering modules
mod render_context;

// Rendering
pub mod color;
pub mod display_list;
pub mod render_task;

// Fonts
pub mod font;
pub mod font_context;
pub mod font_list;

// Misc.
pub mod opts;
mod buffer_map;

// Platform-specific implementations.
#[path="platform/mod.rs"]
pub mod platform;

// Text
#[path = "text/mod.rs"]
pub mod text;

