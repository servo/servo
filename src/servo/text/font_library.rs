export FontLibrary, native;

use font::{Font, test_font_bin};

struct FontLibrary {
    // FIXME: This is a hack to hold onto a boxed reference to
    // the self pointer until explicit self types work on methods.
    // This is a huge space leak.
    mut at_self: Option<@FontLibrary>,
    native_lib: native::NativeFontLibrary,

    drop {
        native::destroy_native_lib(&self.native_lib);
    }
}

impl FontLibrary {
    fn get_font() -> @Font {
        assert self.at_self.is_some();
        match create_font(self.at_self.get(), &self.native_lib) {
          Ok(font) => font,
          Err(*) => /* FIXME */ fail
        }
    }

    fn get_test_font() -> @Font {
        self.get_font()
    }
}

fn FontLibrary() -> @FontLibrary {
    let lib = @FontLibrary {
        mut at_self: None,
        native_lib: native::create_native_lib()
    };

    lib.at_self = Some(lib);
    return lib;
}

fn create_font(lib: @FontLibrary, native_lib: &native::NativeFontLibrary) -> Result<@Font, ()> {
    let font_bin = @test_font_bin();
    let native_font = native_font::create(native_lib, font_bin);
    let native_font = if native_font.is_ok() {
        result::unwrap(native_font)
    } else {
        return Err(native_font.get_err());
    };
    return Ok(@Font(lib, font_bin, native_font));
}

#[cfg(target_os = "linux")]
mod native {
    use ptr::{null, addr_of};
    use azure::freetype;
    use freetype::{FT_Library, FT_Error};
    use freetype::bindgen::{FT_Init_FreeType, FT_Done_FreeType};

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
