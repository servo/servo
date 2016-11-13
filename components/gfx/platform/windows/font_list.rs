/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dwrote::{Font, FontDescriptor, FontCollection};
use servo_atoms::Atom;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{Ordering, AtomicUsize};

lazy_static! {
    static ref FONT_ATOM_COUNTER: AtomicUsize = AtomicUsize::new(1);
    static ref FONT_ATOM_MAP: Mutex<HashMap<Atom, FontDescriptor>> = Mutex::new(HashMap::new());
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "Arial";

pub fn system_default_family(_: &str) -> Option<String> {
    Some("Verdana".to_owned())
}

pub fn last_resort_font_families() -> Vec<String> {
    vec!("Arial".to_owned())
}

pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String) {
    let system_fc = FontCollection::system();
    for family in system_fc.families_iter() {
        callback(family.name());
    }
}

// for_each_variation is supposed to return a string that can be
// atomized and then uniquely used to return back to this font.
// Some platforms use the full postscript name (MacOS X), or
// a font filename.
//
// For windows we're going to use just a basic integer value that
// we'll stringify, and then put them all in a HashMap with
// the actual FontDescriptor there.

pub fn for_each_variation<F>(family_name: &str, mut callback: F) where F: FnMut(String) {
    let system_fc = FontCollection::system();
    if let Some(family) = system_fc.get_font_family_by_name(family_name) {
        let count = family.get_font_count();
        for i in 0..count {
            let font = family.get_font(i);
            let index = FONT_ATOM_COUNTER.fetch_add(1, Ordering::Relaxed);
            let index_str = format!("{}", index);
            let atom = Atom::from(index_str.clone());

            {
                let descriptor = font.to_descriptor();
                let mut fonts = FONT_ATOM_MAP.lock().unwrap();
                fonts.insert(atom, descriptor);
            }

            callback(index_str);
        }
    }
}

pub fn descriptor_from_atom(ident: &Atom) -> FontDescriptor {
    let fonts = FONT_ATOM_MAP.lock().unwrap();
    fonts.get(ident).unwrap().clone()
}

pub fn font_from_atom(ident: &Atom) -> Font {
    let fonts = FONT_ATOM_MAP.lock().unwrap();
    FontCollection::system().get_font_from_descriptor(fonts.get(ident).unwrap()).unwrap()
}
