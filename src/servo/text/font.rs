export font, create;

// FIXME: This probably needs to be an arc type so it can be
// shared by layout and the renderer

#[doc = "
A font handle. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
"]
class font/& {
    let fontbuf: [u8];

    new(+fontbuf: [u8]) {
        self.fontbuf = fontbuf;
    }

    fn buf() -> &self.[u8] {
        &self.fontbuf
    }
}

fn create() -> font {
    let buf = #include_bin("JosefinSans-SemiBold.ttf");
    font(buf)
}
