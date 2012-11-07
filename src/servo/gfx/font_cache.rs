/*
use font::{Font, FontStyle, FontWeight300};
use native::{FontHandle, FontContext};

// Font cache that reuses gfx::Font instances.

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


        return match self.cached_font {
            Some(font) => font,
            None => match self.get_font(&dummy_style) {
                Ok(font) => { self.cached_font = Some(font); font }
                Err(*) => fail
            }
        }
    }

    pub fn get_font(@self, style: &FontStyle) -> Result<@Font, ()> {
        self.create_font(style)
    }
}
*/