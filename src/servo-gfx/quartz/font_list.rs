/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern mod core_foundation;
extern mod core_text;

use native;
use quartz;
use quartz::font_list::core_foundation::array::CFArray;
use quartz::font_list::core_foundation::base::CFWrapper;
use quartz::font_list::core_foundation::string::{CFString, CFStringRef};

use quartz::font_list::core_text::font_descriptor::CTFontDescriptorRef;
use quartz::font_list::core_text::font_collection::CTFontCollectionMethods;

use quartz::font::QuartzFontHandle;
use quartz::font_context::QuartzFontContextHandle;
use gfx_font::FontHandleMethods;
use gfx_font_context::FontContextHandleMethods;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};

use core::hashmap::HashMap;

pub struct QuartzFontListHandle {
    fctx: QuartzFontContextHandle,
}

pub impl QuartzFontListHandle {
    fn new(fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families(&self) -> FontFamilyMap {
        let family_names: CFArray<CFStringRef> =
            quartz::font_list::core_text::font_collection::get_family_names();
        let mut family_map : FontFamilyMap = HashMap::new();
        for family_names.each |&strref: &CFStringRef| {
            let family_name = CFString::wrap_extern(strref).to_str();
            debug!("Creating new FontFamily for family: %s", family_name);

            let new_family = @mut FontFamily::new(family_name);
            family_map.insert(family_name, new_family);
        }
        return family_map;
    }

    fn load_variations_for_family(&self, family: @mut FontFamily) {
        let fam : &mut FontFamily = family; // FIXME: borrow checker workaround
        let family_name = &fam.family_name;
        debug!("Looking for faces of family: %s", *family_name);

        let family_collection =
            quartz::font_list::core_text::font_collection::create_for_family(*family_name);
        for family_collection.get_descriptors().each |descref: &CTFontDescriptorRef| {
            let desc = CFWrapper::wrap_shared(*descref);
            let font = quartz::font_list::core_text::font::new_from_descriptor(&desc, 0.0);
            let handle = result::unwrap(QuartzFontHandle::new_from_CTFont(&self.fctx, font));

            debug!("Creating new FontEntry for face: %s", handle.face_name());
            let entry = @FontEntry::new(family, handle);
            family.entries.push(entry);
        }
    }
}
