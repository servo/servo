/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::FontHandleMethods;
use font_context::FontContextHandleMethods;
use font_list::{FontEntry, FontFamily, FontFamilyMap};
use platform::macos::font::FontHandle;
use platform::macos::font_context::FontContextHandle;

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_text;
use core_text::font_descriptor::{CTFontDescriptor, CTFontDescriptorRef};

use std::cast;
use std::hashmap::HashMap;

pub struct FontListHandle {
    fctx: FontContextHandle,
}

impl FontListHandle {
    pub fn new(fctx: &FontContextHandle) -> FontListHandle {
        FontListHandle {
            fctx: fctx.clone()
        }
    }

    pub fn get_available_families(&self) -> FontFamilyMap {
        let family_names = core_text::font_collection::get_family_names();
        let mut family_map: FontFamilyMap = HashMap::new();
        for strref in family_names.iter() {
            let family_name_ref: CFStringRef = unsafe { cast::transmute(strref) };
            let family_name_cf: CFString = unsafe { TCFType::wrap_under_get_rule(family_name_ref) };
            let family_name = family_name_cf.to_str();
            debug!("Creating new FontFamily for family: {:s}", family_name);

            let new_family = FontFamily::new(family_name);
            family_map.insert(family_name, new_family);
        }
        family_map
    }

    pub fn load_variations_for_family(&self, family: &mut FontFamily) {
        debug!("Looking for faces of family: {:s}", family.family_name);

        let family_collection = core_text::font_collection::create_for_family(family.family_name);
        let family_descriptors = family_collection.get_descriptors();
        for descref in family_descriptors.iter() {
            let descref: CTFontDescriptorRef = unsafe { cast::transmute(descref) };
            let desc: CTFontDescriptor = unsafe { TCFType::wrap_under_get_rule(descref) };
            let font = core_text::font::new_from_descriptor(&desc, 0.0);
            let handle = FontHandle::new_from_CTFont(&self.fctx, font).unwrap();

            debug!("Creating new FontEntry for face: {:s}", handle.face_name());
            let entry = FontEntry::new(handle);
            family.entries.push(entry)
        }
    }

    pub fn get_last_resort_font_families() -> ~[~str] {
        ~[~"Arial Unicode MS",~"Arial"]
    }
}
