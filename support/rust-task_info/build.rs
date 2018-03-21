/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/task_info.c")
        .compile("libtask_info.a");
}
