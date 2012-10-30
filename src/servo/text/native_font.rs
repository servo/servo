/**
NativeFont encapsulates access to the platform's font API,
e.g. quartz, FreeType. It provides access to metrics and tables
needed by the text shaper as well as access to the underlying
font resources needed by the graphics layer to draw glyphs.
*/

use font_cache::native::NativeFontCache;

#[cfg(target_os = "macos")]
pub type NativeFont/& = quartz::native_font::QuartzNativeFont;

#[cfg(target_os = "linux")]
pub type NativeFont/& = freetype::native_font::FreeTypeNativeFont;

// TODO: `new` should be part of trait NativeFont

// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.

// TODO(Rust #1723): #cfg doesn't work for impl methods, so we have
// to conditionally define the entire impl.
#[cfg(target_os = "macos")]
impl NativeFont {
    static pub fn new(native_lib: &NativeFontCache, buf: @~[u8], pt_size: float) -> Result<NativeFont, ()> {
        quartz::native_font::QuartzNativeFont::new(native_lib, buf, pt_size)
    }
}

#[cfg(target_os = "linux")]
impl NativeFont {
    static pub fn new(native_lib: &NativeFontCache, buf: @~[u8], pt_size: float) -> Result<NativeFont, ()> {
        freetype::native_font::FreeTypeNativeFont::new(native_lib, buf, pt_size)
    }
}