/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use getopts::Matches;
use servo::config::opts;
use servo::config::prefs::{self, PrefValue};
use servo::servo_config::basedir;

pub fn register_user_prefs(opts_matches: &Matches) {
    // Read user's prefs.json and then parse --pref command line args.

    let user_prefs_path = opts::get()
        .config_dir
        .clone()
        .or_else(basedir::default_config_dir)
        .map(|path| path.join("prefs.json"))
        .filter(|path| path.exists());

    let mut userprefs = if let Some(path) = user_prefs_path {
        let mut file = File::open(path).expect("Error opening user prefs");
        let mut txt = String::new();
        file.read_to_string(&mut txt)
            .expect("Can't read user prefs file");
        prefs::read_prefs_map(&txt).expect("Can't parse user prefs file")
    } else {
        HashMap::new()
    };

    let argprefs: HashMap<String, PrefValue> = opts_matches
        .opt_strs("pref")
        .iter()
        .map(|pref| {
            let split: Vec<&str> = pref.splitn(2, '=').collect();
            let pref_name = split[0];
            let pref_value = match split.get(1).cloned() {
                Some("true") | None => PrefValue::Bool(true),
                Some("false") => PrefValue::Bool(false),
                Some(string) => {
                    if let Ok(int) = string.parse::<i64>() {
                        PrefValue::Int(int)
                    } else if let Ok(float) = string.parse::<f64>() {
                        PrefValue::Float(float)
                    } else {
                        PrefValue::from(string)
                    }
                },
            };
            (pref_name.to_string(), pref_value)
        })
        .collect();

    // --pref overrides user prefs.json
    userprefs.extend(argprefs);

    prefs::add_user_prefs(userprefs);
}

#[cfg(test)]
fn test_parse_pref(arg: &str) {
    let mut opts = getopts::Options::new();
    opts.optmulti("", "pref", "", "");
    let args = vec!["servo".to_string(), "--pref".to_string(), arg.to_string()];
    let matches = match opts::from_cmdline_args(opts, &args) {
        opts::ArgumentParsingResult::ContentProcess(m, _) => m,
        opts::ArgumentParsingResult::ChromeProcess(m) => m,
    };
    register_user_prefs(&matches);
}

#[test]
fn test_parse_pref_from_command_line() {
    use servo::servo_config::pref;
    // Test with boolean values.
    test_parse_pref("dom.bluetooth.enabled=true");
    assert_eq!(
        prefs::pref_map().get("dom.bluetooth.enabled"),
        PrefValue::Bool(true)
    );
    assert!(pref!(dom.bluetooth.enabled));

    test_parse_pref("dom.bluetooth.enabled=false");
    assert_eq!(
        prefs::pref_map().get("dom.bluetooth.enabled"),
        PrefValue::Bool(false)
    );
    assert_eq!(pref!(dom.bluetooth.enabled), false);

    // Test with numbers
    test_parse_pref("layout.threads=42");
    assert_eq!(pref!(layout.threads), 42);

    // Test string.
    test_parse_pref("shell.homepage=str");
    assert_eq!(pref!(shell.homepage), "str");

    // Test with no value (defaults to true).
    prefs::pref_map()
        .set("dom.bluetooth.enabled", false)
        .unwrap();
    test_parse_pref("dom.bluetooth.enabled");
    assert!(pref!(dom.bluetooth.enabled));
}

#[test]
fn test_invalid_prefs_from_command_line_panics() {
    let err_msg = std::panic::catch_unwind(|| {
        test_parse_pref("doesntexist=true");
    })
    .err()
    .and_then(|a| a.downcast_ref::<String>().cloned())
    .expect("Should panic");
    assert!(
        err_msg.starts_with("Error setting preference"),
        "Message should describe the problem"
    );
    assert!(
        err_msg.contains("doesntexist"),
        "Message should mention the name of the preference"
    );
}
