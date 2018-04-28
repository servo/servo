/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate regex;

use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    // https://github.com/rust-lang/cargo/issues/3544
    let style_out_dir = env::var_os("DEP_FOR SOME REASON THE LINKS KEY IS REQUIRED \
                                     TO PASS DATA AROUND BETWEEN BUILD SCRIPTS_OUT_DIR").unwrap();
    let root_path = Path::new("../../../");
    let bindings_file = Path::new(&style_out_dir).join("gecko/bindings.rs");
    let glue_file = root_path.join("ports/geckolib/glue.rs");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", glue_file.display());
    println!("cargo:rerun-if-changed={}", bindings_file.display());

    let env_out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&env_out_dir);

    {
        let output = out_dir.join("check_bindings.rs");
        let r = BufReader::new(File::open(bindings_file).unwrap());
        let mut w = File::create(output).unwrap();

        w.write_all(b"fn assert_types() {\n").unwrap();

        let matcher = Regex::new(r"fn\s*Servo_([a-zA-Z0-9_]+)\s*\(").unwrap();

        for line in r.lines() {
            let s = line.unwrap();
            for cap in matcher.captures_iter(&s) {
                // This causes a mismatch in old libclangs (the ones that are
                // used in linux32 mozilla-central) because it generates:
                //
                //   *const nsTArray<*const RawServoStyleSet>
                //
                // Instead of:
                //
                //   *const nsTArray<RawServoStyleSetBorrowed>
                //
                // Which is not a problem, but would cause this to not compile.
                //
                // Skip this until libclang is updated there.
                if &cap[1] == "InvalidateStyleForDocStateChanges" {
                    continue;
                }
                w.write_all(format!("    [ Servo_{0}, bindings::Servo_{0} ];\n", &cap[1]).as_bytes()).unwrap();
            }
        }

        w.write_all(b"}\n").unwrap();
    }

    {
        let output = out_dir.join("glue.rs");
        let r = BufReader::new(File::open(glue_file).unwrap());
        let mut w = File::create(output).unwrap();

        w.write_all(b"pub use style::gecko::arc_types::*;\n").unwrap();

        for line in r.lines() {
            let s = line.unwrap().replace("pub extern \"C\" fn", "pub unsafe extern \"C\" fn");
            w.write_all(s.as_bytes()).unwrap();
            w.write_all(b"\n").unwrap();
        }
    }

    File::create(out_dir.join("bindings.rs"))
        .unwrap()
        .write_all(format!("include!(concat!({:?}, \"/gecko/structs.rs\"));",
                           style_out_dir).as_bytes())
        .unwrap();

    if env::var_os("MOZ_SRC").is_some() {
        println!("cargo:rustc-cfg=linking_with_gecko")
    }
}
