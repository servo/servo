/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use layout::PersistentLayoutData;
use script_layout_interface::PartialPersistentLayoutData;
use std::mem::align_of;

fn check_layout_alignment(expected: usize, current: usize) {
    if current != expected {
        panic!("Your changes have altered the mem alignment of the PartialPersistentLayoutData \
                struct to {}, but it must match the {}-alignment of PersistentLayoutData struct. \
                Please fix alignment in components/script_layout_interface/lib.rs",
                current, expected);
    }
}

#[test]
fn test_persistent_layout_data_alignment() {
    check_layout_alignment(align_of::<PersistentLayoutData>(),
                           align_of::<PartialPersistentLayoutData>());

    check_layout_alignment(align_of::<AtomicRefCell<PersistentLayoutData>>(),
                           align_of::<AtomicRefCell<PartialPersistentLayoutData>>());
}
