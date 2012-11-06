/**
FontHandle encapsulates access to the platform's font API,
e.g. quartz, FreeType. It provides access to metrics and tables
needed by the text shaper as well as access to the underlying
font resources needed by the graphics layer to draw glyphs.
*/

#[cfg(target_os = "macos")]
pub type FontHandle/& = quartz::font_handle::QuartzFontHandle;

#[cfg(target_os = "linux")]
pub type FontHandle/& = freetype::font_handle::FreeTypeFontHandle;

// TODO: `new` should be part of trait FontHandle

// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.

// TODO(Rust #1723): #cfg doesn't work for impl methods, so we have
// to conditionally define the entire impl.
#[cfg(target_os = "macos")]
impl FontHandle {
    static pub fn new(fctx: &native::FontContext, buf: @~[u8], pt_size: float) -> Result<FontHandle, ()> {
        quartz::font_handle::QuartzFontHandle::new(fctx, buf, pt_size)
    }
}

#[cfg(target_os = "linux")]
impl FontHandle {
    static pub fn new(fctx: &native::FontContext, buf: @~[u8], pt_size: float) -> Result<FontHandle, ()> {
        freetype::font_handle::FreeTypeFontHandle::new(fctx, buf, pt_size)
    }
}