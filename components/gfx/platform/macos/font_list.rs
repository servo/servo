/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_text;
use std::borrow::ToOwned;

pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String) {
    let family_names = core_text::font_collection::get_family_names();
    for family_name in family_names.iter() {
        callback(family_name.to_string());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F) where F: FnMut(String) {
    debug!("Looking for faces of family: {}", family_name);

    let family_collection = core_text::font_collection::create_for_family(family_name);
    if let Some(family_collection) = family_collection {
        let family_descriptors = family_collection.get_descriptors();
        for family_descriptor in family_descriptors.iter() {
            callback(family_descriptor.font_name());
        }
    }
}

pub fn system_default_family(_generic_name: &str) -> Option<String> {
    None
}

pub fn last_resort_font_families() -> Vec<String> {
    vec!("Arial Unicode MS".to_owned(), "Arial".to_owned())
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "Helvetica";
