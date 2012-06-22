export FontLibrary;

import font::Font;

class FontLibrary {
    let bogus: int;

    new() { self.bogus = 0; }

    fn get_font() -> @Font {
        let f = Font(font::test_font_bin());
        ret @f;
    }
}

#[test]
fn should_get_fonts() {
    let lib = FontLibrary();
    lib.get_font();
}