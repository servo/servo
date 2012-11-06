/**
Shaper encapsulates a specific shaper, such as Harfbuzz, 
Uniscribe, Pango, or Coretext.

Currently, only harfbuzz bindings are implemented.
*/
use font::Font;

pub type Shaper/& = harfbuzz::shaper::HarfbuzzShaper;

// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.
impl Shaper {
    static pub fn new(font: @Font) -> Shaper {
        harfbuzz::shaper::HarfbuzzShaper::new(font)
    }
}