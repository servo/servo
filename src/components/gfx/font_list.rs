/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{CSSFontWeight, SpecifiedFontStyle};
use gfx_font::FontHandleMethods;
use platform::font::FontHandle;
use platform::font_context::FontContextHandle;
use platform::font_list::FontListHandle;
use servo_util::time;
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

use std::hashmap::HashMap;

pub type FontFamilyMap = HashMap<~str, @mut FontFamily>;

trait FontListHandleMethods {
    fn get_available_families(&self, fctx: &FontContextHandle) -> FontFamilyMap;
    fn load_variations_for_family(&self, family: @mut FontFamily);
    fn get_last_resort_font_families() -> ~[~str];
}

/// The platform-independent font list abstraction.
pub struct FontList {
    family_map: FontFamilyMap,
    handle: FontListHandle,
    prof_chan: ProfilerChan,
}

impl FontList {
    pub fn new(fctx: &FontContextHandle,
           prof_chan: ProfilerChan)
           -> FontList {
        let handle = FontListHandle::new(fctx);
        let mut list = FontList {
            handle: handle,
            family_map: HashMap::new(),
            prof_chan: prof_chan.clone(),
        };
        list.refresh(fctx);
        list
    }

    fn refresh(&mut self, _: &FontContextHandle) {
        // TODO(Issue #186): don't refresh unless something actually
        // changed.  Does OSX have a notification for this event?
        //
        // Should font families with entries be invalidated/refreshed too?
        do profile(time::GfxRegenAvailableFontsCategory, self.prof_chan.clone()) {
            self.family_map = self.handle.get_available_families();
        }
    }

    pub fn find_font_in_family(&self,
                           family_name: &str, 
                           style: &SpecifiedFontStyle) -> Option<@FontEntry> {
        let family = self.find_family(family_name);

        // TODO(Issue #192: handle generic font families, like 'serif' and 'sans-serif'.

        // if such family exists, try to match style to a font
        let mut result: Option<@FontEntry> = None;
        for fam in family.iter() {
            result = fam.find_font_for_style(&self.handle, style);
        }

        let decision = if result.is_some() {
            "Found"
        } else {
            "Couldn't find"
        };

        debug!("FontList: %s font face in family[%s] matching style", decision, family_name);

        result
    }

    fn find_family(&self, family_name: &str) -> Option<@mut FontFamily> {
        // look up canonical name
        let family = self.family_map.find_equiv(&family_name);

        let decision = if family.is_some() { "Found" } else { "Couldn't find" };
        debug!("FontList: %s font family with name=%s", decision, family_name);

        // TODO(Issue #188): look up localized font family names if canonical name not found
        family.map(|f| **f)
    }

    pub fn get_last_resort_font_families() -> ~[~str] {
        let last_resort = FontListHandle::get_last_resort_font_families();
        last_resort
    }
}

// Holds a specific font family, and the various 
pub struct FontFamily {
    family_name: ~str,
    entries: ~[@FontEntry],
}

impl FontFamily {
    pub fn new(family_name: &str) -> FontFamily {
        FontFamily {
            family_name: family_name.to_str(),
            entries: ~[],
        }
    }

    fn load_family_variations(@mut self, list: &FontListHandle) {
        if self.entries.len() > 0 {
            return
        }
        list.load_variations_for_family(self);
        assert!(self.entries.len() > 0)
    }

    pub fn find_font_for_style(@mut self, list: &FontListHandle, style: &SpecifiedFontStyle)
                            -> Option<@FontEntry> {
        self.load_family_variations(list);

        // TODO(Issue #189): optimize lookup for
        // regular/bold/italic/bolditalic with fixed offsets and a
        // static decision table for fallback between these values.

        // TODO(Issue #190): if not in the fast path above, do
        // expensive matching of weights, etc.
        for entry in self.entries.iter() {
            if (style.weight.is_bold() == entry.is_bold()) && 
               (style.italic == entry.is_italic()) {

                return Some(*entry);
            }
        }

        None
    }
}

/// This struct summarizes an available font's features. In the future, this will include fiddly
/// settings such as special font table handling.
///
/// In the common case, each FontFamily will have a singleton FontEntry, or it will have the
/// standard four faces: Normal, Bold, Italic, BoldItalic.
pub struct FontEntry {
    face_name: ~str,
    priv weight: CSSFontWeight,
    priv italic: bool,
    handle: FontHandle,
    // TODO: array of OpenType features, etc.
}

impl FontEntry {
    pub fn new(handle: FontHandle) -> FontEntry {
        FontEntry {
            face_name: handle.face_name(),
            weight: handle.boldness(),
            italic: handle.is_italic(),
            handle: handle
        }
    }

    pub fn is_bold(&self) -> bool {
        self.weight.is_bold()
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }
}

