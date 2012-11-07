#[cfg(target_os = "macos")]
type FontListHandle/& = quartz::font_list::QuartzFontListHandle;

#[cfg(target_os = "linux")]
type FontListHandle/& = freetype::font_list::FreeTypeFontListHandle;

#[cfg(target_os = "macos")]
pub impl FontListHandle {
    static pub fn new() -> FontListHandle {
        quartz::font_list::QuartzFontListHandle::new()
    }
}

#[cfg(target_os = "linux")]
pub impl FontListHandle {
    static pub fn new() -> FontListHandle {
        freetype::font_list::FreeTypeFontListHandle::new()
    }
}
