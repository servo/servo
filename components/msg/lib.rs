/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unused_imports, unused_variable)]

extern crate azure;
extern crate geom;
extern crate http;
extern crate layers;
extern crate serialize;
extern crate "util" as servo_util;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_foundation;
#[cfg(target_os="macos")]
extern crate io_surface;

pub mod compositor_msg;
pub mod constellation_msg;

pub mod platform {
    #[cfg(target_os="macos")]
    pub mod macos {
        #[cfg(target_os="macos")]
        pub mod surface;
    }

    #[cfg(target_os="linux")]
    pub mod linux {
        #[cfg(target_os="linux")]
        pub mod surface;
    }

    #[cfg(target_os="android")]
    pub mod android {
        #[cfg(target_os="android")]
        pub mod surface;
    }


    pub mod surface;
}

