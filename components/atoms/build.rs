/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate string_cache_codegen;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;

fn main() {
    let static_atoms = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("static_atoms.txt");
    let static_atoms = BufReader::new(File::open(&static_atoms).unwrap());
    string_cache_codegen::AtomType::new("Atom", "atom!")
        .atoms(static_atoms.lines().map(Result::unwrap))
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("atom.rs"))
        .unwrap();
}
