use font::{Font, FontStyle};
use font_context::FontContext;

struct FontMatcher {
    fctx: @FontContext,
}

impl FontMatcher {
    static pub fn new(fctx: @FontContext) -> FontMatcher {
        FontMatcher {
            fctx: fctx,
        }
    }
}
