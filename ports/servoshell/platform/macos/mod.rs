/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Duration;
use std::{ptr, thread};

use mach2::kern_return::KERN_SUCCESS;
use mach2::mach_port::mach_port_deallocate;
use mach2::mach_types::{thread_act_array_t, thread_act_t};
use mach2::task::task_threads;
use mach2::traps::current_task;
use mach2::vm::mach_vm_deallocate;

pub fn deinit(clean_shutdown: bool) {
    // An unfortunate hack to make sure the linker's dead code stripping doesn't strip our
    // `Info.plist`.
    unsafe {
        ptr::read_volatile(&INFO_PLIST[0]);
    }

    let Some(thread_count) = count_running_threads() else {
        log::error!("Could not get thread count during shutdown. Exiting immediately.");
        return;
    };

    if thread_count != 1 {
        log::debug!(
            "{} threads are still running after shutdown (bad).",
            thread_count
        );
        if clean_shutdown {
            log::debug!("Waiting until all threads have shutdown");
            loop {
                let Some(thread_count) = count_running_threads() else {
                    log::error!("Could not get thread count during shutdown. Exiting immediately.");
                    return;
                };

                if thread_count == 1 {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
                log::debug!("{thread_count} threads are still running.");
            }
        }
    } else {
        log::debug!("All threads have shutdown (good).");
    }
}

#[expect(unsafe_code)]
fn count_running_threads() -> Option<u32> {
    let current_task = unsafe { current_task() };
    let mut threads: thread_act_array_t = std::ptr::null_mut();
    let mut thread_count = 0;

    // SAFETY: Task is the return value of `current_task` and the parameters are initalized above.
    if unsafe { task_threads(current_task, &mut threads, &mut thread_count) } != KERN_SUCCESS {
        return None;
    }

    // SAFTEY: This only deallocates the number of threads that were returned from `task_threads`.
    for index in 0..thread_count as isize {
        unsafe { mach_port_deallocate(current_task, *threads.offset(index)) };
    }

    // SAFTEY: This only happens when the call above is successful.
    unsafe {
        mach_vm_deallocate(
            current_task,
            threads as _,
            (std::mem::size_of::<thread_act_t>() * thread_count as usize) as _,
        );
    }

    Some(thread_count)
}

#[unsafe(link_section = "__TEXT,__info_plist")]
#[unsafe(no_mangle)]
pub static INFO_PLIST: [u8; 1050] = *include_bytes!("Info.plist");
