use gfx::{
    FontFamily,
};

use dvec::DVec;

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

pub struct FontList {
    families: DVec<@FontFamily>,
    handle: FontListHandle,
}

pub impl FontList {
    static fn new(fctx: &native::FontContextHandle) -> FontList {
        let handle = result::unwrap(FontListHandle::new(fctx));
        let list = FontList {
            handle: move handle,
            families: DVec(),
        };
        list.refresh(fctx);
        return move list;
    }

    priv fn refresh(fctx: &native::FontContextHandle) {
        // TODO: don't refresh unless something actually changed.
        // Does OSX have a notification for this event?
        // It would be better to do piecemeal.
        do self.families.swap |_old_families: ~[@FontFamily]| {
            self.handle.get_available_families(fctx)
        }
    }
}