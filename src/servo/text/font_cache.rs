export FontCache, native;
use font::{Font, test_font_bin};

struct FontCache {
    native_lib: native::NativeFontCache,
    mut cached_font: Option<@Font>,

    drop {
        native::destroy_native_lib(&self.native_lib);
    }
}

impl FontCache {
    fn get_font(@self) -> @Font {
        match self.cached_font {
            Some(font) => font,
            None => match create_font(self, &self.native_lib) {
                Ok(font) => {
                    self.cached_font = Some(font);
                    font
                }
                Err(*) => /* FIXME */ fail
            }
        }
    }

    fn get_test_font(@self) -> @Font {
        self.get_font()
    }
}

fn FontCache() -> @FontCache {
    @FontCache {
        native_lib: native::create_native_lib(),
        cached_font: None
    }
}

fn create_font(lib: @FontCache, native_lib: &native::NativeFontCache) -> Result<@Font, ()> {
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
pub mod native {
    use ptr::{null, addr_of};
    use azure::freetype;
    use freetype::{FT_Library, FT_Error};
    use freetype::bindgen::{FT_Init_FreeType, FT_Done_FreeType};

    pub type NativeFontCache = FT_Library;

    pub fn create_native_lib() -> NativeFontCache {
        let lib: FT_Library = null();
        let res = FT_Init_FreeType(addr_of(&lib));
        // FIXME: error handling
        assert res == 0 as FT_Error;
        return lib;
    }

    pub fn destroy_native_lib(native_lib: &NativeFontCache) {
        assert native_lib.is_not_null();
        FT_Done_FreeType(*native_lib);
    }
}

#[cfg(target_os = "macos")]
pub mod native {
    pub type NativeFontCache = ();

    pub fn create_native_lib() -> NativeFontCache { () }
    pub fn destroy_native_lib(_native_lib: &NativeFontCache) { }
}

#[test]
pub fn should_get_fonts() {
    let lib = FontCache();
    lib.get_font();
}
