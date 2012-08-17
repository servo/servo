export FontLibrary, native;

import font::{Font, test_font_bin};
import result::{result, ok, err};

struct FontLibrary {
    let native_lib: native::NativeFontLibrary;

    new() {
        self.native_lib = native::create_native_lib();
    }

    drop {
        native::destroy_native_lib(&self.native_lib);
    }

    fn get_font() -> @Font {
        match create_font(&self.native_lib) {
          ok(font) => font,
          err(*) => /* FIXME */ fail
        }
    }

    fn get_test_font() -> @Font {
        self.get_font()
    }
}


fn create_font(native_lib: &native::NativeFontLibrary) -> result<@Font, ()> {
    let font_bin = test_font_bin();
    let native_font = native_font::create(native_lib, &font_bin);
    let native_font = if native_font.is_ok() {
        result::unwrap(native_font)
    } else {
        return err(native_font.get_err());
    };
    return ok(@Font(font_bin, native_font));
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
