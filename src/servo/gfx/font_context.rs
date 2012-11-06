// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.

// TODO(Rust #1723): #cfg doesn't work for impl methods, so we have
// to conditionally define the entire impl.

#[cfg(target_os = "macos")]
type FontContext/& = quartz::font_context::QuartzFontContext;

#[cfg(target_os = "linux")]
type FontContext/& = freetype::font_context::FreeTypeFontContext;

#[cfg(target_os = "macos")]
pub impl FontContext {
    static pub fn new() -> FontContext {
        quartz::font_context::QuartzFontContext::new()
    }
}

#[cfg(target_os = "linux")]
pub impl FontContext {
    static pub fn new() -> FontContext {
        freetype::font_context::FreeTypeFontContext::new()
    }
}
