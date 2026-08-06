/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
// compile-flags: --error-format=human
//@rustc-env:RUSTC_BOOTSTRAP=1

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::cell::{RefCell};

#[crown::unrooted_must_root_lint::must_root]
struct MustRoot;

#[crown::unrooted_must_root_lint::must_root]
pub struct StdTypes {
    refcell: RefCell<MustRoot>,
    hashmap: HashMap<(), MustRoot>,
    hashset: HashSet<MustRoot>,
    vector: Vec<MustRoot>,
}

fn test_std_types(std_types: &mut StdTypes) {
    // Ref
    let foo = std_types.refcell.borrow();
    // RefMut
    let foo = std_types.refcell.borrow_mut();

    // slice Iter
    let foo = std_types.vector[..].iter();
    // slice IterMut
    let foo = std_types.vector[..].iter_mut();

    // hashmap Entry
    let entry = std_types.hashmap.entry(());
    match entry {
        // OccupiedEntry
        Entry::Occupied(occupied_entry) => (),
        // VacantEntry
        Entry::Vacant(vacant_entry) => (),
    }

    // hashmap Iter
    for (_, val) in std_types.hashmap.iter() {
       let _ = val;
    }

    // hashmap Values
    for val in std_types.hashmap.values() {
       let _ = val;
    }

    // hashset Iter
    for val in std_types.hashset.iter() {
       let _ = val;
    }
}

fn main() {}
