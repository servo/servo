import glyph::GlyphIndex;

class NativeFont/& {
    let bogus: int;

    new() { self.bogus = 0; }

    fn glyph_index(_codepoint: char) -> option<GlyphIndex> {
        fail;
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(_glyph: GlyphIndex) -> int {
        fail;
    }
}
