/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate servo_config;

use servo_config::basedir;
use servo_config::prefs::{PREFS, PrefValue, read_prefs_from_file};
use std::fs::{self, File};
use std::io::{Read, Write};

#[test]
fn test_create_pref() {
    let json_str = "{\
  \"layout.writing-mode.enabled\": true,\
  \"network.mime.sniff\": false,\
  \"shell.homepage\": \"https://servo.org\"\
}";

    let prefs = read_prefs_from_file(json_str.as_bytes());
    assert!(prefs.is_ok());
    let prefs = prefs.unwrap();

    assert_eq!(prefs.len(), 3);
}

#[test]
fn test_get_set_reset_extend() {
    let json_str = "{\
  \"layout.writing-mode.enabled\": true,\
  \"extra.stuff\": false,\
  \"shell.homepage\": \"https://google.com\"\
}";

    assert_eq!(*PREFS.get("test"), PrefValue::Missing);
    PREFS.set("test", PrefValue::String("hi".to_owned()));
    assert_eq!(*PREFS.get("test"), PrefValue::String("hi".to_owned()));
    assert_eq!(*PREFS.get("shell.homepage"), PrefValue::String("https://servo.org".to_owned()));
    PREFS.set("shell.homepage", PrefValue::Boolean(true));
    assert_eq!(*PREFS.get("shell.homepage"), PrefValue::Boolean(true));
    PREFS.reset("shell.homepage");
    assert_eq!(*PREFS.get("shell.homepage"), PrefValue::String("https://servo.org".to_owned()));

    let extension = read_prefs_from_file(json_str.as_bytes()).unwrap();
    PREFS.extend(extension);
    assert_eq!(*PREFS.get("shell.homepage"), PrefValue::String("https://google.com".to_owned()));
    assert_eq!(*PREFS.get("layout.writing-mode.enabled"), PrefValue::Boolean(true));
    assert_eq!(*PREFS.get("extra.stuff"), PrefValue::Boolean(false));
}

#[test]
fn test_default_config_dir_create_read_write() {
  let json_str = "{\
  \"layout.writing-mode.enabled\": true,\
  \"extra.stuff\": false,\
  \"shell.homepage\": \"https://google.com\"\
}";
    let mut expected_json = String::new();
    let config_path = basedir::default_config_dir().unwrap();

    if !config_path.exists() {
      fs::create_dir_all(&config_path).unwrap();
    }

    let json_path = config_path.join("test_config.json");

    let mut fd = File::create(&json_path).unwrap();
    assert_eq!(json_path.exists(), true);

    fd.write_all(json_str.as_bytes()).unwrap();
    let mut fd = File::open(&json_path).unwrap();
    fd.read_to_string(&mut expected_json).unwrap();

    assert_eq!(json_str, expected_json);

    fs::remove_file(&json_path).unwrap();
}
