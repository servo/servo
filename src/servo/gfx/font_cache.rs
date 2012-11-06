use font::{Font, FontStyle, FontWeight300};
use native::{FontHandle, FontContext};

// TODO(Issue #164): delete, and get default font from font list
const TEST_FONT: [u8 * 33004] = #include_bin("JosefinSans-SemiBold.ttf");

fn test_font_bin() -> ~[u8] {
    return vec::from_fn(33004, |i| TEST_FONT[i]);
}

// Dummy font cache.

struct FontCache {
    fctx: @FontContext,
    mut cached_font: Option<@Font>
}

impl FontCache {
    static pub fn new(fctx: @FontContext) -> FontCache {
        FontCache { 
            fctx: fctx,
            cached_font: None
        }
    }
    
    pub fn get_test_font(@self) -> @Font {
        let dummy_style = FontStyle {
            pt_size: 40f,
            weight: FontWeight300,
            italic: false,
            oblique: false
        };

        return match self.cached_font {
            Some(font) => font,
            None => match self.get_font(&dummy_style) {
                Ok(font) => { self.cached_font = Some(font); font }
                Err(*) => /* FIXME */ fail
            }
        }
    }

    // TODO: maybe FontStyle should be canonicalized when used in FontCache?
    priv fn create_font(style: &FontStyle) -> Result<@Font, ()> {
        let font_bin = @test_font_bin();
        let native_font = FontHandle::new(self.fctx, font_bin, style.pt_size);
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