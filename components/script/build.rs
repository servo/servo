/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::process::Command;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let num_jobs = env::var("NUM_JOBS").unwrap();
    assert!(Command::new("make")
        .args(&["-f", "makefile.cargo", "-j", &num_jobs])
        .status()
        .unwrap()
        .success());
    println!("Binding generation completed in {}s", start.elapsed().as_secs());
}
