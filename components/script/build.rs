/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cmake;
use std::env;
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // This must use the Ninja generator -- it's the only one that
    // parallelizes cmake's output properly.  (Cmake generates
    // separate makefiles, each of which try to build
    // ParserResults.pkl, and then stomp on eachother.)
    let mut build = cmake::Config::new(".");

    let target = env::var("TARGET").unwrap();
    if target.contains("windows-msvc") {
        // We must use Ninja on Windows for this -- msbuild is painfully slow,
        // and ninja is easier to install than make.
        build.generator("Ninja");
        // because we're using ninja, we need to explicitly set these
        // to VC++, otherwise it'll try to use cc
        build.define("CMAKE_C_COMPILER", "cl.exe")
             .define("CMAKE_CXX_COMPILER", "cl.exe");
        // We have to explicitly specify the full path to link.exe,
        // for reasons that I don't understand.  If we just give
        // link.exe, it tries to use script-*/out/link.exe, which of
        // course does not exist.
        build.define("CMAKE_LINKER", "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\VC\\bin\\amd64\\link.exe");
    }

    build.build();

    println!("Binding generation completed in {}s", start.elapsed().as_secs());
}
