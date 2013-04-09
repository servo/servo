//! Platform-specific modules.

#[cfg(target_os="linux")] pub use platform::linux::{font, font_context, font_list};
#[cfg(target_os="macos")] pub use platform::macos::{font, font_context, font_list};

#[cfg(target_os="linux")]
pub mod linux {
    pub mod font;
    pub mod font_context;
    pub mod font_list;
}

#[cfg(target_os="macos")]
pub mod macos {
    pub mod font;
    pub mod font_context;
    pub mod font_list;
}

