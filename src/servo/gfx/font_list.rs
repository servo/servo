#[cfg(target_os = "macos")]
type FontListHandle/& = quartz::font_list::QuartzFontListHandle;

#[cfg(target_os = "linux")]
type FontListHandle/& = freetype::font_list::FreeTypeFontListHandle;

pub impl FontListHandle {
    #[cfg(target_os = "macos")]
    static pub fn new() -> FontListHandle {
        quartz::font_list::QuartzFontListHandle::new()
    }

    #[cfg(target_os = "linux")]
    static pub fn new() -> FontListHandle {
        freetype::font_list::FreeTypeFontListHandle::new()
    }
}
