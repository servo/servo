/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `ringtail` browser - a simplified Servo browser that displays
//! a builtin HTML page instead of a full UI.

#![windows_subsystem = "windows"]

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Console;

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        let _result = Console::AttachConsole(Console::ATTACH_PARENT_PROCESS);
    }
    
    cfg_if::cfg_if! {
        if #[cfg(not(any(target_os = "android", target_env = "ohos")))] {
            ringtail::main()
        } else {
            println!(
                "Cannot run the ringtail `bin` executable on platforms such as \
                 Android or OpenHarmony."
            );
        }
    }
}
