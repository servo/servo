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

// TODO: this should be part of trait NativeFont
#[cfg(target_os = "macos")]
pub fn create(native_lib: &NativeFontCache, buf: @~[u8], pt_size: float) -> Result<NativeFont, ()> {
    quartz::native_font::create(native_lib, buf, pt_size)
}

#[cfg(target_os = "linux")]
pub fn create(native_lib: &NativeFontCache, buf: @~[u8], pt_size: float) -> Result<NativeFont, ()> {
    freetype::native_font::create(native_lib, buf, pt_size)
}
