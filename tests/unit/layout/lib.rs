/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate atomic_refcell;
extern crate layout;
extern crate script_layout_interface;
#[macro_use] extern crate size_of_test;

#[cfg(test)] mod align_of;
#[cfg(all(test, target_pointer_width = "64"))] mod size_of;
