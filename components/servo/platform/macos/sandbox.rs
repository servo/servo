/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A Mac OS-specific sandbox using Seatbelt (a.k.a. sandbox(7)).

use compositing::content_process::Zone;
use libc::{c_char, c_int};
use servo_util::resource_files;
use std::ptr;

// TODO(pcwalton): Lock down file reading and network access once the resource task is rewritten
// (should be soon).
static PROFILE_COMMON: &'static str = "
(version 1)
(deny default)
(allow mach-lookup
       (global-name \"com.apple.CoreServices.coreservicesd\")
       (global-name \"com.apple.FontServer\"))
(allow network-outbound
       (literal \"/private/var/run/mDNSResponder\")
       (remote tcp \"*:443\")
       (remote tcp \"*:80\"))
(allow sysctl-read)
(allow system-socket)
";

static PROFILE_LOCAL: &'static str = "
(allow file-read*
       (subpath \"/\"))
";

static PROFILE_REMOTE: &'static str = "
(allow file-read*
       (subpath \"/System/Library/Frameworks/ApplicationServices.framework\")
       (subpath \"/System/Library/Fonts\")
       (subpath \"/Library/Fonts\")
       (subpath \"/usr/share/zoneinfo\")
       (subpath \"%RESOURCES%\")
       (literal \"/dev/urandom\")
       (literal \"/private/etc/hosts\"))
(allow file-read-metadata
       (literal \"/private/etc/localtime\")
       (literal \"/etc\")
       (literal \"/var\"))
";

pub fn enter(zone: Zone) {
    let mut err = ptr::null_mut();
    let profile = format!("{}{}",
                          PROFILE_COMMON,
                          match zone {
                            Zone::Local => PROFILE_LOCAL,
                            Zone::Remote => PROFILE_REMOTE,
                          });

    // Substitute `%RESOURCES%`, being careful not to allow for silly SQL injection-like stuff.
    let resources_path = resource_files::resources_dir_path();
    if resources_path.display().as_cow().contains_char('"') {
        // TODO(pcwalton): Figure out the Scheme dialect syntax for this...
        panic!("can't sandbox on Mac OS X when the resource path contains a double quote in it")
    }
    let profile = profile.replace("%RESOURCES%", resources_path.display().as_cow().as_slice());

    let profile = profile.to_c_str();
    unsafe {
        if sandbox_init(profile.as_ptr() as *const c_char, 0, &mut err) != 0 {
            panic!("failed to enter sandbox")
        }
    }
}

extern {
    fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
}

