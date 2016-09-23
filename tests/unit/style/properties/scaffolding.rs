/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc_serialize::json::Json;
use std::env;
use std::fs::{File, remove_file};
use std::path::Path;
use std::process::Command;

#[test]
fn properties_list_json() {
    let top = Path::new(file!()).parent().unwrap().join("..").join("..").join("..").join("..");
    let json = top.join("target").join("doc").join("servo").join("css-properties.json");
    if json.exists() {
        remove_file(&json).unwrap()
    }
    let python = env::var("PYTHON").ok().unwrap_or_else(find_python);
    let script = top.join("components").join("style").join("properties").join("build.py");
    let status = Command::new(python)
        .arg(&script)
        .arg("servo")
        .arg("html")
        .arg("regular")
        .status()
        .unwrap();
    assert!(status.success());
    let properties = Json::from_reader(&mut File::open(json).unwrap()).unwrap();
    assert!(properties.as_object().unwrap().len() > 100);
    assert!(properties.find("margin").is_some());
    assert!(properties.find("margin-top").is_some());
}

#[cfg(windows)]
fn find_python() -> String {
    if Command::new("python2.7.exe").arg("--version").output().is_ok() {
        return "python2.7.exe".to_owned();
    }

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
