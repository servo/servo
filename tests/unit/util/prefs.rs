/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::prefs::{PrefValue, extend_prefs, read_prefs_from_file, get_pref, set_pref, reset_pref};

#[test]
fn test_create_pref() {
    let json_str = "{\
  \"layout.writing-mode.enabled\": true,\
  \"net.mime.sniff\": false,\
  \"shell.homepage\": \"http://servo.org\"\
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

    assert_eq!(*get_pref("test"), PrefValue::Missing);
    set_pref("test", PrefValue::String("hi".to_owned()));
    assert_eq!(*get_pref("test"), PrefValue::String("hi".to_owned()));
    assert_eq!(*get_pref("shell.homepage"), PrefValue::String("http://servo.org".to_owned()));
    set_pref("shell.homepage", PrefValue::Boolean(true));
    assert_eq!(*get_pref("shell.homepage"), PrefValue::Boolean(true));
    reset_pref("shell.homepage");
    assert_eq!(*get_pref("shell.homepage"), PrefValue::String("http://servo.org".to_owned()));

    let extension = read_prefs_from_file(json_str.as_bytes()).unwrap();
    extend_prefs(extension);
    assert_eq!(*get_pref("shell.homepage"), PrefValue::String("https://google.com".to_owned()));
    assert_eq!(*get_pref("layout.writing-mode.enabled"), PrefValue::Boolean(true));
    assert_eq!(*get_pref("extra.stuff"), PrefValue::Boolean(false));
}
