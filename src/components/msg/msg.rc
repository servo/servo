/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[crate_id = "github.com/mozilla/servo#msg:0.1"];
#[crate_type = "lib"];

extern mod azure;
extern mod extra;
extern mod geom;
extern mod layers;
extern mod std;

#[cfg(target_os="macos")]
extern mod core_foundation;
#[cfg(target_os="macos")]
extern mod io_surface;

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

