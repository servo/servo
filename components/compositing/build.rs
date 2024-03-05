/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let lockfile_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..")
        .join("Cargo.lock");
    let revision_file_path =
        Path::new(&env::var_os("OUT_DIR").unwrap()).join("webrender_revision.rs");

    let mut lockfile = String::new();
    File::open(lockfile_path)
        .expect("Cannot open lockfile")
        .read_to_string(&mut lockfile)
        .expect("Failed to read lockfile");

    match toml::from_str::<toml::value::Table>(&lockfile) {
        Ok(result) => {
            let packages = result
                .get("package")
                .expect("Cargo lockfile should contain package list");

            match *packages {
                toml::Value::Array(ref arr) => {
                    let source = arr
                        .iter()
                        .find(|pkg| {
                            pkg.get("name").and_then(|name| name.as_str()).unwrap_or("") ==
                                "webrender"
                        })
                        .and_then(|pkg| pkg.get("source").and_then(|source| source.as_str()))
                        .unwrap_or("unknown");

                    let parsed: Vec<&str> = source.split('#').collect();
                    let revision = if parsed.len() > 1 { parsed[1] } else { source };

                    let mut revision_module_file = File::create(revision_file_path).unwrap();
                    write!(&mut revision_module_file, "\"{}\"", revision).unwrap();
                },
                _ => panic!("Cannot find package definitions in lockfile"),
            }
        },
        Err(e) => panic!("{}", e),
    }
}
