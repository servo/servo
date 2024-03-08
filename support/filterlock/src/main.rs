/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: The hash for the `vendorTarball` in etc/shell.nix should be
// regenerated when this file is changed.

//! Filter the given lockfile to only the given package and its dependencies.
//!
//! Usage: `filterlock <path/to/Cargo.lock> <package>`
//!
//! This helper is used only by the Nix shell environment (etc/shell.nix).

use std::{fs::File, env::{args_os, args}, io::Read, collections::BTreeSet};

use serde::{Deserialize, Serialize};
use toml::{map::Map, Value};

#[derive(Deserialize, Serialize)]
struct LockFile {
    package: Vec<Package>,
    #[serde(flatten)]
    other: Map<String, Value>,
}

#[derive(Deserialize, Serialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Option<Vec<String>>,
    #[serde(flatten)]
    other: Map<String, Value>,
}

fn main() {
    let usage = "Usage: filterlock <path/to/Cargo.lock> <package>";
    let path = args_os().nth(1).expect(usage);
    let package = args().nth(2).expect(usage);

    let mut file = File::open(path).expect("Failed to open lockfile");
    let mut toml = String::new();
    file.read_to_string(&mut toml).expect("Failed to read lockfile");
    let toml: LockFile = toml::from_str(&toml).expect("Failed to parse lockfile");

    // Find the closure of the given package and its dependencies.
    let mut keep = BTreeSet::new();
    let mut queue = vec![
        toml.package.iter()
            .find(|p| p.matches(&package))
            .expect("Failed to find package"),
    ];
    while !queue.is_empty() {
        let package = queue.pop().expect("Guaranteed by while");
        keep.insert((package.name.clone(), package.version.clone()));
        if let Some(dependencies) = package.dependencies.as_ref() {
            for dependency in dependencies {
                let package = toml.package.iter()
                    .find(|p| p.matches(&dependency))
                    .expect("Failed to find package");
                queue.push(package);
            }
        }
    }

    // Remove packages that are not in the closure.
    let mut toml = toml;
    let filtered_packages = toml.package.drain(..)
        .filter(|p| keep.contains(&(p.name.clone(), p.version.clone())))
        .collect();
    let toml = LockFile { package: filtered_packages, ..toml };

    println!("{}", toml::to_string(&toml).expect("Failed to serialise lockfile"));
}

impl Package {
    fn matches(&self, spec: &str) -> bool {
        if let Some((name, version)) = spec.split_once(" ") {
            self.name == name && self.version == version
        } else {
            self.name == spec
        }
    }
}
