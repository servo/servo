/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::path::Path;


fn main() {
    let python = if Command::new("python2.7").arg("--version").status().unwrap().success() {
        "python2.7"
    } else {
        "python"
    };
    let style = Path::new(file!()).parent().unwrap();
    let mako = style.join("Mako-0.9.1.zip");
    let template = style.join("properties.mako.rs");
    let result = Command::new(python)
        .env("PYTHONPATH", &mako)
        .env("TEMPLATE", &template)
        .arg("-c")
        .arg("from os import environ; from mako.template import Template; \
              from mako import exceptions; \n\
              try:\n    print(Template(filename=environ['TEMPLATE']).render());\n\
              except:\n    print exceptions.html_error_template().render()")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(result.status.success());
    let out = env::var("OUT_DIR").unwrap();
    File::create(&Path::new(&out).join("properties.rs")).unwrap().write_all(&result.stdout).unwrap();
}
