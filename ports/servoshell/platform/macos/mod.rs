/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Duration;
use std::{ptr, thread};

pub fn deinit(clean_shutdown: bool) {
    // An unfortunate hack to make sure the linker's dead code stripping doesn't strip our
    // `Info.plist`.
    unsafe {
        ptr::read_volatile(&INFO_PLIST[0]);
    }

    let thread_count = unsafe { macos_count_running_threads() };

    if thread_count != 1 {
        println!(
            "{} threads are still running after shutdown (bad).",
            thread_count
        );
        if clean_shutdown {
            println!("Waiting until all threads have shutdown");
            loop {
                let thread_count = unsafe { macos_count_running_threads() };
                if thread_count == 1 {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
                println!("{} threads are still running.", thread_count);
            }
        }
    } else {
        println!("All threads have shutdown (good).");
    }
}

#[link_section = "__TEXT,__info_plist"]
#[no_mangle]
pub static INFO_PLIST: [u8; 619] = *include_bytes!("Info.plist");

#[link(name = "count_threads")]
extern "C" {
    fn macos_count_running_threads() -> i32;
}
