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
    let style = Path::new(file!()).parent().unwrap();
    let mako = style.join("Mako-0.9.1.zip");
    let template = style.join("properties.mako.rs");
    let product = if cfg!(feature = "gecko") { "gecko" } else { "servo" };
    let result = Command::new(python)
        .env("PYTHONPATH", &mako)
        .env("TEMPLATE", &template)
        .env("PRODUCT", product)
        .arg("-c")
        .arg(r#"
import os
import sys
from mako.template import Template
from mako import exceptions
try:
    template = Template(open(os.environ['TEMPLATE'], 'rb').read(), input_encoding='utf8')
    print(template.render(PRODUCT=os.environ['PRODUCT']).encode('utf8'))
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
