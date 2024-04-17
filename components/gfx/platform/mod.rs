/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use crate::platform::freetype::{font, font_list, library_handle};
#[cfg(target_os = "macos")]
pub use crate::platform::macos::{core_text_font_cache, font, font_list};
#[cfg(target_os = "windows")]
pub use crate::platform::windows::{font, font_list};

#[cfg(any(target_os = "linux", target_os = "android"))]
mod freetype {
    use std::ffi::CStr;
    use std::str;

    use libc::c_char;

    /// Creates a String from the given null-terminated buffer.
    /// Panics if the buffer does not contain UTF-8.
    unsafe fn c_str_to_string(s: *const c_char) -> String {
        str::from_utf8(CStr::from_ptr(s).to_bytes())
            .unwrap()
            .to_owned()
    }

    pub mod font;

    #[cfg(target_os = "linux")]
    pub mod font_list;
    #[cfg(target_os = "android")]
    mod android {
        pub mod font_list;
        mod xml;
    }
    #[cfg(target_os = "android")]
    pub use self::android::font_list;

    pub mod library_handle;
}

#[cfg(target_os = "macos")]
mod macos {
    pub mod core_text_font_cache;
    pub mod font;
    pub mod font_list;
}

#[cfg(target_os = "windows")]
mod windows {
    pub mod font;
    pub mod font_list;
}
