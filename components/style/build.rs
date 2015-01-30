/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::os;
use std::path::Path;
use std::io::process::{Command, ProcessExit, StdioContainer};
use std::io::File;


fn main() {
    let python = if Command::new("python2.7").arg("--version").status() == Ok(ProcessExit::ExitStatus(0)) {
        "python2.7"
    } else {
        "python"
    };
    let style = Path::new(file!()).dir_path();
    let mako = style.join("Mako-0.9.1.zip");
    let template = style.join("properties.mako.rs");
    let result = Command::new(python)
        .env("PYTHONPATH", mako.as_str().unwrap())
        .env("TEMPLATE", template.as_str().unwrap())
        .arg("-c")
        .arg("from os import environ; from mako.template import Template; print(Template(filename=environ['TEMPLATE']).render())")
        .stderr(StdioContainer::InheritFd(2))
        .output()
        .unwrap();
    assert_eq!(result.status, ProcessExit::ExitStatus(0));
    let out = Path::new(os::getenv("OUT_DIR").unwrap());
    File::create(&out.join("properties.rs")).unwrap().write(&*result.output).unwrap();
}
