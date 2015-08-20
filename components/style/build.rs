/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio, exit};


fn main() {
    let python = if Command::new("python2.7").arg("--version").output().unwrap().status.success() {
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
        .arg(r#"
import os
import sys
from mako.template import Template
from mako import exceptions
try:
    print(Template(filename=os.environ['TEMPLATE'], input_encoding='utf8').render().encode('utf8'))
except:
    sys.stderr.write(exceptions.text_error_template().render().encode('utf8'))
    sys.exit(1)
"#)
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    if !result.status.success() {
        exit(1)
    }
    let out = env::var("OUT_DIR").unwrap();
    File::create(&Path::new(&out).join("properties.rs")).unwrap().write_all(&result.stdout).unwrap();
}
