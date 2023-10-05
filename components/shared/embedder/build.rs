/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // Cargo does not expose the profile name to crates or their build scripts,
    // but we can extract it from OUT_DIR and set a custom cfg() ourselves.
    let out = std::env::var("OUT_DIR")?;
    let out = Path::new(&out);
    let krate = out.parent().unwrap();
    let build = krate.parent().unwrap();
    let profile = build.parent().unwrap();
    if profile.file_name().unwrap() == "production" {
        println!("cargo:rustc-cfg=servo_production");
    } else {
        println!("cargo:rustc-cfg=servo_do_not_use_in_production");
    }

    Ok(())
}
