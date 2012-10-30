use font::{Font, FontStyle, FontWeight300};
use font_matcher::FontMatcher;
// Dummy font cache.

struct FontCache {
    matcher: @FontMatcher,
    mut cached_font: Option<@Font>
}

impl FontCache {
    static pub fn new(matcher: @FontMatcher) -> FontCache {
        FontCache { 
            matcher: matcher,
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
            None => match self.matcher.get_font(&dummy_style) {
                Ok(font) => { self.cached_font = Some(font); font }
                Err(*) => /* FIXME */ fail
            }
        }
    }
}