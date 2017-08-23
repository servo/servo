/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cmake;
extern crate phf_codegen;
extern crate phf_shared;
extern crate serde_json;

use serde_json::Value;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::Write;
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
        let link = std::process::Command::new("where").arg("link.exe").output().unwrap();
        let link_path: Vec<&str> = std::str::from_utf8(&link.stdout).unwrap().split("\r\n").collect();
        build.define("CMAKE_LINKER", link_path[0]);
    }

    build.build();

    println!("Binding generation completed in {}s", start.elapsed().as_secs());

    let json = PathBuf::from(env::var("OUT_DIR").unwrap()).join("build").join("InterfaceObjectMapData.json");
    let json: Value = serde_json::from_reader(File::open(&json).unwrap()).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in json.as_object().unwrap() {
        map.entry(Bytes(key), value.as_str().unwrap());
    }
    let phf = PathBuf::from(env::var("OUT_DIR").unwrap()).join("InterfaceObjectMapPhf.rs");
    let mut phf = File::create(&phf).unwrap();
    write!(&mut phf, "pub static MAP: phf::Map<&'static [u8], unsafe fn(*mut JSContext, HandleObject)> = ").unwrap();
    map.build(&mut phf).unwrap();
    write!(&mut phf, ";\n").unwrap();
}

#[derive(Eq, Hash, PartialEq)]
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
