/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_text::font_descriptor::{CTFontDescriptor, CTFontDescriptorRef};
use core_text;
use std::mem;

pub fn get_available_families(callback: |String|) {
    let family_names = core_text::font_collection::get_family_names();
    for strref in family_names.iter() {
        let family_name_ref: CFStringRef = unsafe { mem::transmute(strref) };
        let family_name_cf: CFString = unsafe { TCFType::wrap_under_get_rule(family_name_ref) };
        let family_name = family_name_cf.to_string();
        callback(family_name);
    }
}

pub fn get_variations_for_family(family_name: &str, callback: |String|) {
    debug!("Looking for faces of family: {:s}", family_name);

    let family_collection =
        core_text::font_collection::create_for_family(family_name.as_slice());
    let family_descriptors = family_collection.get_descriptors();
    for descref in family_descriptors.iter() {
        let descref: CTFontDescriptorRef = unsafe { mem::transmute(descref) };
        let desc: CTFontDescriptor = unsafe { TCFType::wrap_under_get_rule(descref) };
        let postscript_name = desc.font_name();
        callback(postscript_name);
    }
}

pub fn get_last_resort_font_families() -> Vec<String> {
    vec!("Arial Unicode MS".to_string(), "Arial".to_string())
}
