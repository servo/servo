/**
Shaper encapsulates a specific shaper, such as Harfbuzz, 
Uniscribe, Pango, or Coretext.

Currently, only harfbuzz bindings are implemented.
*/

pub type Shaper/& = harfbuzz::shaper::HarfbuzzShaper;
