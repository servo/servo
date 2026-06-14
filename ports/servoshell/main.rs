/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `servoshell` test application.
//!
//! Creates a `Servo` instance with a example implementation of a working
//! web browser.
//!
//! This browser's implementation of `WindowMethods` is built on top
//! of [winit], the cross-platform windowing library.
//!
//! For the engine itself look next door in `components/servo/lib.rs`.
//!
//! [winit]: https://github.com/rust-windowing/winit

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Console;

fn main() {
    #[cfg(target_os = "windows")]
    // SAFETY: No safety related side effects or requirements.
    // See <https://learn.microsoft.com/en-au/windows/console/freeconsole#remarks>
    unsafe {
        // Free the console pop-up when started by double clicking.
        // If started from the command line, nothing would happen.
        let _ = Console::FreeConsole();
        // Try to attach to the console of the parent process.
        // If servo was started from the command line,
        // this would allow continous stdout/stderr output to be seen in the console.
        // Otherwise, the call will fail, which we can ignore.
        let _result = Console::AttachConsole(Console::ATTACH_PARENT_PROCESS);
    }
    cfg_if::cfg_if! {
        if #[cfg(not(any(target_os = "android", target_env = "ohos")))] {
            servoshell::main()
        } else {
            // Android: see ports/servoshell/egl/android/mod.rs.
            // OpenHarmony: see ports/servoshell/egl/ohos/mod.rs.
            println!(
                "Cannot run the servoshell `bin` executable on platforms such as \
                 Android or OpenHarmony. On these platforms you need to compile \
                 the servoshell library as a `cdylib` and integrate it with the \
                 platform app code into an `apk` (android) or `hap` (OpenHarmony).\
                 For Android `mach build` will do these steps automatically for you."
            );
        }
    }
}
