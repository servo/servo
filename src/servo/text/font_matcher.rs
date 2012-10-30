use font::{Font, FontStyle};
use native_font::NativeFont;
use native_font_matcher::NativeFontMatcher;

// TODO(Issue #164): delete, and get default font from NativeFontMatcher
const TEST_FONT: [u8 * 33004] = #include_bin("JosefinSans-SemiBold.ttf");

fn test_font_bin() -> ~[u8] {
    return vec::from_fn(33004, |i| TEST_FONT[i]);
}

struct FontMatcher {
    native_matcher: NativeFontMatcher,
    // TODO(Issue #165): move into FontCache
    mut cached_font: Option<@Font>,
}

impl FontMatcher {
    static pub fn new() -> FontMatcher {
        FontMatcher {
            native_matcher: NativeFontMatcher::new(),
            cached_font: None
        }
    }

    // TODO: maybe FontStyle should be canonicalized when used in FontCache?
    // TODO(Issue #166): move this to FontCache or something? At the least, use it there.
    priv fn create_font(style: &FontStyle) -> Result<@Font, ()> {
        let font_bin = @test_font_bin();
        let native_font = NativeFont::new(&self.native_matcher, font_bin, style.pt_size);
        let native_font = if native_font.is_ok() {
            result::unwrap(move native_font)
        } else {
            return Err(native_font.get_err());
        };

        return Ok(@Font::new(font_bin, move native_font, copy *style));
    }

    pub fn get_font(@self, style: &FontStyle) -> Result<@Font, ()> {
        self.create_font(style)
    }
}
