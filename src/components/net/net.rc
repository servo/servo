/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[crate_id = "github.com/mozilla/servo#net:0.1"];
#[crate_type = "lib"];

#[feature(globs, managed_boxes)];

extern mod geom;
extern mod http;
extern mod servo_util = "util";
extern mod stb_image;
extern mod extra;
extern mod png;

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
    pub mod holder;
}

pub mod file_loader;
pub mod http_loader;
pub mod data_loader;
pub mod image_cache_task;
pub mod local_image_cache;
pub mod resource_task;
pub mod util;

