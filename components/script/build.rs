/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cmake;
extern crate phf_codegen;
extern crate phf_shared;
extern crate regex;

use regex::Regex;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
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

    convert_phf();
}

fn convert_phf() {
    let filename = PathBuf::from(env::var("OUT_DIR").unwrap()).join("InterfaceObjectMap.rs");
    let mut source = String::new();
    File::open(&filename).unwrap().read_to_string(&mut source).unwrap();
    let map_macro = Regex::new("phf_map! \\{([^}]+)\\}").unwrap().captures(&source).unwrap();
    let entries_re = Regex::new("b\"([^\"]+)\" => ([^\n]+),\n").unwrap();
    let entries = entries_re.captures_iter(&map_macro[1]);

    let mut map = phf_codegen::Map::new();
    for entry in entries {
        map.entry(Bytes(entry.get(1).unwrap().as_str()), entry.get(2).unwrap().as_str());
    }

    let mut file = File::create(&filename).unwrap();
    let map_macro = map_macro.get(0).unwrap();
    file.write_all(source[..map_macro.start()].as_bytes()).unwrap();
    map.build(&mut file).unwrap();
    file.write_all(source[map_macro.end()..].as_bytes()).unwrap();
}

#[derive(Eq, PartialEq, Hash)]
struct Bytes<'a>(&'a str);

impl<'a> fmt::Debug for Bytes<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("b\"")?;
        formatter.write_str(self.0)?;
        formatter.write_str("\" as &'static [u8]")
    }
}

impl<'a> phf_shared::PhfHash for Bytes<'a> {
    fn phf_hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.0.as_bytes().phf_hash(hasher)
    }
}
