/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn install() {}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn install() {
    use crate::backtrace;
    use libc::_exit;
    use sig::ffi::Sig;
    use std::{io::Write, sync::atomic, thread};

    extern "C" fn handler(sig: i32) {
        static BEEN_HERE_BEFORE: atomic::AtomicBool = atomic::AtomicBool::new(false);
        if !BEEN_HERE_BEFORE.swap(true, atomic::Ordering::SeqCst) {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let _ = write!(&mut stdout, "Stack trace");
            if let Some(name) = thread::current().name() {
                let _ = write!(&mut stdout, " for thread \"{}\"", name);
            }
            let _ = write!(&mut stdout, "\n");
            let _ = backtrace::print(&mut stdout);
        }
        unsafe {
            _exit(sig);
        }
    }

    signal!(Sig::SEGV, handler); // handle segfaults
    signal!(Sig::ILL, handler); // handle stack overflow and unsupported CPUs
    signal!(Sig::IOT, handler); // handle double panics
    signal!(Sig::BUS, handler); // handle invalid memory access
}
