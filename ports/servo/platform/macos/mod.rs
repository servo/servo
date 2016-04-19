/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;

pub fn deinit() {
    // An unfortunate hack to make sure the linker's dead code stripping doesn't strip our
    // `Info.plist`.
    unsafe {
        ptr::read_volatile(&INFO_PLIST[0]);
    }
}

#[cfg(target_os = "macos")]
#[link_section = "__TEXT,__info_plist"]
#[no_mangle]
pub static INFO_PLIST: [u8; 619] = *include_bytes!("Info.plist");

