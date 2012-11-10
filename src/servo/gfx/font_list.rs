use gfx::{
    FontFamily,
};

use dvec::DVec;
use send_map::{linear, SendMap};

#[cfg(target_os = "macos")]
type FontListHandle/& = quartz::font_list::QuartzFontListHandle;

#[cfg(target_os = "linux")]
type FontListHandle/& = freetype::font_list::FreeTypeFontListHandle;

pub impl FontListHandle {
    #[cfg(target_os = "macos")]
    static pub fn new(fctx: &native::FontContextHandle) -> Result<FontListHandle, ()> {
        Ok(quartz::font_list::QuartzFontListHandle::new(fctx))
    }

    #[cfg(target_os = "linux")]
    static pub fn new(fctx: &native::FontContextHandle) -> Result<FontListHandle, ()> {
        Ok(freetype::font_list::FreeTypeFontListHandle::new(fctx))
    }
}

type FontFamilyMap = linear::LinearMap<~str, @FontFamily>;

pub struct FontList {
    mut family_map: FontFamilyMap,
    mut handle: FontListHandle,
}

pub impl FontList {
    static fn new(fctx: &native::FontContextHandle) -> FontList {
        let handle = result::unwrap(FontListHandle::new(fctx));
        let list = FontList {
            handle: move handle,
            family_map: linear::LinearMap(),
        };
        list.refresh(fctx);
        return move list;
    }

    priv fn refresh(fctx: &native::FontContextHandle) {
        // TODO(Issue #186): don't refresh unless something actually
        // changed.  Does OSX have a notification for this event?
        do util::time::time("gfx::font_list: regenerating available font families and faces") {
            self.family_map = self.handle.get_available_families(fctx);
        }
    }
}