/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String)
{
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
    where F: FnMut(String)
{
}

pub fn system_default_family(generic_name: &str) -> Option<String> {
    None
}

pub fn last_resort_font_families() -> Vec<String> {
    vec!(
        "Unknown".to_owned()
    )
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "Unknown";
