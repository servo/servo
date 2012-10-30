// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.

// TODO(Rust #1723): #cfg doesn't work for impl methods, so we have
// to conditionally define the entire impl.

#[cfg(target_os = "macos")]
type NativeFontMatcher/& = quartz::native_font_matcher::QuartzNativeFontMatcher;

#[cfg(target_os = "linux")]
type NativeFontMatcher/& = freetype::native_font_matcher::FreeTypeNativeFontMatcher;

#[cfg(target_os = "macos")]
pub impl NativeFontMatcher {
    static pub fn new() -> NativeFontMatcher {
        quartz::native_font_matcher::QuartzNativeFontMatcher::new()
    }
}

#[cfg(target_os = "linux")]
pub impl NativeFontMatcher {
    static pub fn new() -> NativeFontMatcher {
        freetype::native_font_matcher::FreeTypeNativeFontMatcher::new()
    }
}
