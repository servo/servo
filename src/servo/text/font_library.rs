export FontLibrary, native;

import font::Font;

class FontLibrary {
    let native_lib: native::NativeFontLibrary;

    new() {
        self.native_lib = native::create_native_lib();
    }

    drop {
        native::destroy_native_lib(&self.native_lib);
    }

    fn get_font() -> @Font {
        let f = Font(font::test_font_bin());
        return @f;
    }

    fn get_test_font() -> @Font {
        self.get_font()
    }
}

#[cfg(target_os = "linux")]
mod native {
    import ptr::{null, addr_of};
    import azure::freetype;
    import freetype::{FT_Library, FT_Error};
    import freetype::bindgen::{FT_Init_FreeType, FT_Done_FreeType};

    type NativeFontLibrary = FT_Library;

    fn create_native_lib() -> NativeFontLibrary {
        let lib: FT_Library = null();
        let res = FT_Init_FreeType(addr_of(lib));
        // FIXME: error handling
        assert res == 0 as FT_Error;
        return lib;
    }

    fn destroy_native_lib(native_lib: &NativeFontLibrary) {
        assert native_lib.is_not_null();
        FT_Done_FreeType(*native_lib);
    }
}

#[cfg(target_os = "macos")]
mod native {
    type NativeFontLibrary = ();

    fn create_native_lib() -> NativeFontLibrary { () }
    fn destroy_native_lib(_native_lib: &NativeFontLibrary) { }
}

#[test]
fn should_get_fonts() {
    let lib = FontLibrary();
    lib.get_font();
}
