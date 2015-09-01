/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use platform::freetype::{font, font_context, font_list, font_template};

#[cfg(target_os = "macos")]
pub use platform::macos::{font, font_context, font_list, font_template};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod freetype {
    pub mod font;
    pub mod font_context;
    pub mod font_list;
    pub mod font_template;
}

#[cfg(target_os = "macos")]
pub mod macos {
    pub mod font;
    pub mod font_context;
    pub mod font_list;
    pub mod font_template;
}
