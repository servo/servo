/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn install() {}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn install() {
    use std::io::Write;
    use std::sync::atomic;
    use std::thread;

    use sig::ffi::Sig;

    use crate::backtrace;

    extern "C" fn handler(sig: i32) {
        // Only print crash message and backtrace the first time, to avoid
        // infinite recursion if the printing causes another signal.
        static BEEN_HERE_BEFORE: atomic::AtomicBool = atomic::AtomicBool::new(false);
        if !BEEN_HERE_BEFORE.swap(true, atomic::Ordering::SeqCst) {
            // stderr is unbuffered, so we won’t lose output if we crash later
            // in this handler, and the std::io::stderr() call never allocates.
            // std::io::stdout() allocates the first time it’s called, which in
            // practice will often segfault (see below).
            let stderr = std::io::stderr();
            let mut stderr = stderr.lock();
            let _ = write!(&mut stderr, "Caught signal {sig}");
            if let Some(name) = thread::current().name() {
                let _ = write!(&mut stderr, " in thread \"{}\"", name);
            }
            let _ = writeln!(&mut stderr);

            // This call always allocates, which in practice will segfault if
            // we’re handling a non-main-thread (e.g. layout) segfault. Strictly
            // speaking in POSIX terms, this is also undefined behaviour.
            let _ = backtrace::print(&mut stderr);
        }

        // Outside the BEEN_HERE_BEFORE check, we must only call functions we
        // know to be “async-signal-safe”, which includes sigaction(), raise(),
        // and _exit(), but generally doesn’t include anything that allocates.
        // https://pubs.opengroup.org/onlinepubs/9699919799/functions/V2_chap02.html#tag_15_04_03_03
        unsafe {
            // Reset the signal to the default action, and reraise the signal.
            // Unlike libc::_exit(sig), which terminates the process normally,
            // this terminates abnormally just like an uncaught signal, allowing
            // mach (or your shell) to distinguish it from an ordinary exit, and
            // allows your kernel to make a core dump if configured to do so.
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = libc::SIG_DFL;
            libc::sigaction(sig, &action, std::ptr::null_mut());
            libc::raise(sig);
        }
    }

    signal!(Sig::SEGV, handler); // handle segfaults
    signal!(Sig::ILL, handler); // handle stack overflow and unsupported CPUs
    signal!(Sig::IOT, handler); // handle double panics
    signal!(Sig::BUS, handler); // handle invalid memory access
}
