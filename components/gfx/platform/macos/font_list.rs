/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_text::font_descriptor::{CTFontDescriptor, CTFontDescriptorRef};
use core_text;

use std::borrow::ToOwned;
use std::mem;

pub fn get_available_families<F>(mut callback: F) where F: FnMut(String) {
    let family_names = core_text::font_collection::get_family_names();
    for strref in family_names.iter() {
        let family_name_ref: CFStringRef = unsafe { mem::transmute(strref) };
        let family_name_cf: CFString = unsafe { TCFType::wrap_under_get_rule(family_name_ref) };
        let family_name = family_name_cf.to_string();
        callback(family_name);
    }
}

pub fn get_variations_for_family<F>(family_name: &str, mut callback: F) where F: FnMut(String) {
    debug!("Looking for faces of family: {}", family_name);

    let family_collection =
        core_text::font_collection::create_for_family(family_name.as_slice());
    match family_collection {
        Some(family_collection) => {
            let family_descriptors = family_collection.get_descriptors();
            for descref in family_descriptors.iter() {
                let descref: CTFontDescriptorRef = unsafe { mem::transmute(descref) };
                let desc: CTFontDescriptor = unsafe { TCFType::wrap_under_get_rule(descref) };
                let postscript_name = desc.font_name();
                callback(postscript_name);
            }
        }
        None => {}
    }
}

pub fn get_system_default_family(_generic_name: &str) -> Option<String> {
    None
}

pub fn get_last_resort_font_families() -> Vec<String> {
    vec!("Arial Unicode MS".to_owned(), "Arial".to_owned())
}
