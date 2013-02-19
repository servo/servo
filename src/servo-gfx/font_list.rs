use font::{CSSFontWeight, SpecifiedFontStyle, UsedFontStyle};
use gfx_font::FontHandleMethods;
use native::FontHandle;

use dvec::DVec;
use core::hashmap::linear;

#[cfg(target_os = "linux")]
use fontconfig;
#[cfg(target_os = "macos")]
use quartz;
use native;
use util::time::time;

#[cfg(target_os = "macos")]
type FontListHandle/& = quartz::font_list::QuartzFontListHandle;

#[cfg(target_os = "linux")]
type FontListHandle/& = fontconfig::font_list::FontconfigFontListHandle;

pub impl FontListHandle {
    #[cfg(target_os = "macos")]
    static pub fn new(fctx: &native::FontContextHandle) -> Result<FontListHandle, ()> {
        Ok(quartz::font_list::QuartzFontListHandle::new(fctx))
    }

    #[cfg(target_os = "linux")]
    static pub fn new(fctx: &native::FontContextHandle) -> Result<FontListHandle, ()> {
        Ok(fontconfig::font_list::FontconfigFontListHandle::new(fctx))
    }
}

pub type FontFamilyMap = linear::LinearMap<~str, @FontFamily>;

trait FontListHandleMethods {
    fn get_available_families(&const self, fctx: &native::FontContextHandle) -> FontFamilyMap;
    fn load_variations_for_family(&const self, family: @FontFamily);
}

pub struct FontList {
    mut family_map: FontFamilyMap,
    handle: FontListHandle,
}

pub impl FontList {
    static fn new(fctx: &native::FontContextHandle) -> FontList {
        let handle = result::unwrap(FontListHandle::new(fctx));
        let list = FontList {
            handle: move handle,
            family_map: linear::LinearMap::new(),
        };
        list.refresh(fctx);
        return move list;
    }

    priv fn refresh(&self, _fctx: &native::FontContextHandle) {
        // TODO(Issue #186): don't refresh unless something actually
        // changed.  Does OSX have a notification for this event?
        //
        // Should font families with entries be invalidated/refreshed too?
        do time("gfx::font_list: regenerating available font families and faces") {
            self.family_map = self.handle.get_available_families();
        }
    }

    fn find_font_in_family(&self, family_name: &str, 
                           style: &SpecifiedFontStyle) -> Option<@FontEntry> {
        let family = self.find_family(family_name);
        let mut result : Option<@FontEntry> = None;

        // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.

        // if such family exists, try to match style to a font
        do family.iter |fam| {
            result = fam.find_font_for_style(&self.handle, style);
        }

        let decision = if result.is_some() { "Found" } else { "Couldn't find" };
        debug!("FontList: %s font face in family[%s] matching style", decision, family_name);

        return result;
    }

    priv fn find_family(&self, family_name: &str) -> Option<@FontFamily> {
        // look up canonical name
        let family = self.family_map.find(&str::from_slice(family_name));

        let decision = if family.is_some() { "Found" } else { "Couldn't find" };
        debug!("FontList: %s font family with name=%s", decision, family_name);

        // TODO(Issue #188): look up localized font family names if canonical name not found
        return family.map(|f| **f);
    }
}

// Holds a specific font family, and the various 
pub struct FontFamily {
    family_name: ~str,
    entries: DVec<@FontEntry>,
}

pub impl FontFamily {
    static fn new(family_name: &str) -> FontFamily {
        FontFamily {
            family_name: str::from_slice(family_name),
            entries: DVec(),
        }
    }

    priv fn load_family_variations(@self, list: &native::FontListHandle) {
        if self.entries.len() > 0 { return; }
        list.load_variations_for_family(self);
        assert self.entries.len() > 0;
    }

    fn find_font_for_style(@self, list: &native::FontListHandle, style: &SpecifiedFontStyle) -> Option<@FontEntry> {

        self.load_family_variations(list);

        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.

        // TODO(Issue #190): if not in the fast path above, do
        // expensive matching of weights, etc.
        for self.entries.each |entry| {
            if (style.weight.is_bold() == entry.is_bold()) && 
               (style.italic == entry.is_italic()) {

                return Some(*entry);
            }
        }

        return None;
    }
}

// This struct summarizes an available font's features. In the future,
// this will include fiddly settings such as special font table handling.

// In the common case, each FontFamily will have a singleton FontEntry, or
// it will have the standard four faces: Normal, Bold, Italic, BoldItalic.
pub struct FontEntry {
    family: @FontFamily,
    face_name: ~str,
    priv weight: CSSFontWeight,
    priv italic: bool,
    handle: FontHandle,
    // TODO: array of OpenType features, etc.
}

pub impl FontEntry {
    static fn new(family: @FontFamily, handle: FontHandle) -> FontEntry {
        FontEntry {
            family: family,
            face_name: handle.face_name(),
            weight: handle.boldness(),
            italic: handle.is_italic(),
            handle: move handle
        }
    }

    pure fn is_bold(&self) -> bool { 
        self.weight.is_bold()
    }

    pure fn is_italic(&self) -> bool { self.italic }
}
