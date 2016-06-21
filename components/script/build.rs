/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cmake;

fn main() {
    // This must use the Ninja generator -- it's the only one that
    // parallelizes cmake's output properly.  (Cmake generates
    // separate makefiles, each of which try to build
    // ParserResults.pkl, and then stomp on eachother.)
    let _ = cmake::Config::new(".").generator("Ninja").build();
}
