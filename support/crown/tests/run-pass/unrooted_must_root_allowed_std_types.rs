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

fn new_std_types(_: &()) -> &StdTypes {
    unimplemented!()
}
fn new_std_types_mut(_: &()) -> &mut StdTypes {
    unimplemented!()
}

fn test_std_types() {
    let std_types = new_std_types(&());
    let std_types_mut = new_std_types_mut(&());

    // Ref
    let _ = std_types.refcell.borrow();
    // RefMut
    let _ = std_types.refcell.borrow_mut();

    // slice Iter
    for val in std_types.vector.iter() {
       let _ = val;
    }
    // iter
    let _ = std_types.vector[..].iter();
    // iter_mut
    let _ = std_types_mut.vector[..].iter_mut();

    // hashmap Entry
    match std_types_mut.hashmap.entry(()) {
        // OccupiedEntry
        Entry::Occupied(_) => (),
        // VacantEntry
        Entry::Vacant(_) => (),
    }

    // hashmap iter
    for (_, val) in std_types.hashmap.iter() {
       let _ = val;
    }

    // hasmap values
    for val in std_types.hashmap.values() {
       let _ = val;
    }

    // hashset iter
    for val in std_types.hashset.iter() {
       let _ = val;
    }
}

fn main() {}
