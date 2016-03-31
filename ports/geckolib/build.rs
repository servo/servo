/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio, exit};

#[cfg(windows)]
fn find_python() -> String {
    if Command::new("python27.exe").arg("--version").output().is_ok() {
        return "python27.exe".to_owned();
    }

    if Command::new("python.exe").arg("--version").output().is_ok() {
        return "python.exe".to_owned();
    }

    panic!("Can't find python (tried python27.exe and python.exe)! Try fixing PATH or setting the PYTHON env var");
}

#[cfg(not(windows))]
fn find_python() -> String {
    if Command::new("python2.7").arg("--version").output().unwrap().status.success() {
        "python2.7"
    } else {
        "python"
    }.to_owned()
}

fn main() {
    let python = match env::var("PYTHON") {
        Ok(python_path) => python_path,
        Err(_) => find_python(),
    };

    // Mako refuses to load templates outside the scope of the current working directory,
    // so we need to run it from the top source directory.
    let geckolib_dir = Path::new(file!()).parent().unwrap();
    let top_dir = geckolib_dir.join("..").join("..");

    let style_template = Path::new("components/style/properties.mako.rs");
    let geckolib_template = Path::new("ports/geckolib/properties.mako.rs");
    let mako = Path::new("components/style/Mako-0.9.1.zip");

    let result = Command::new(python)
        .current_dir(top_dir)
        .env("PYTHONPATH", &mako)
        .env("STYLE_TEMPLATE", &style_template)
        .env("GECKOLIB_TEMPLATE", &geckolib_template)
        .arg("-c")
        .arg(r#"
import json
import os
import sys
from mako.template import Template
from mako import exceptions
try:
    style_template = Template(filename=os.environ['STYLE_TEMPLATE'], input_encoding='utf8')
    style_template.render(PRODUCT='gecko')

    geckolib_template = Template(filename=os.environ['GECKOLIB_TEMPLATE'], input_encoding='utf8')
    output = geckolib_template.render(STYLE_STRUCTS = style_template.module.STYLE_STRUCTS)
    print(output.encode('utf8'))
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
