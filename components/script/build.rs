/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use phf_shared;
use serde_json::{self, Value};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str;
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
    }

    build.build();

    println!(
        "Binding generation completed in {}s",
        start.elapsed().as_secs()
    );

    let json = PathBuf::from(env::var_os("OUT_DIR").unwrap())
        .join("build")
        .join("InterfaceObjectMapData.json");
    let json: Value = serde_json::from_reader(File::open(&json).unwrap()).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in json.as_object().unwrap() {
        map.entry(Bytes(key), value.as_str().unwrap());
    }
    let phf = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("InterfaceObjectMapPhf.rs");
    let mut phf = File::create(&phf).unwrap();
    write!(
        &mut phf,
        "pub static MAP: phf::Map<&'static [u8], /* unsafe */ fn(/* *mut */ JSContext, HandleObject)> = "
    )
    .unwrap();
    map.build(&mut phf).unwrap();
    write!(&mut phf, ";\n").unwrap();
}

#[derive(Eq, Hash, PartialEq)]
struct Bytes<'a>(&'a str);

impl<'a> fmt::Debug for Bytes<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // https://github.com/rust-lang/rust/issues/55223
        // should technically be just `write!(formatter, "b\"{}\"", self.0)
        // but the referenced issue breaks promotion in the surrounding code
        write!(formatter, "{{ const FOO: &[u8] = b\"{}\"; FOO }}", self.0)
    }
}

impl<'a> phf_shared::PhfHash for Bytes<'a> {
    fn phf_hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.0.as_bytes().phf_hash(hasher)
    }
}
