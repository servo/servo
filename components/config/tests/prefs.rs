/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};

use servo_config::basedir;
use servo_config::pref_util::Preferences;
use servo_config::prefs::{read_prefs_map, PrefValue};

#[test]
fn test_create_prefs_map() {
    let json_str = "{
        \"layout.writing-mode.enabled\": true,
        \"network.mime.sniff\": false,
        \"shell.homepage\": \"https://servo.org\"
    }";
    let prefs = read_prefs_map(json_str);
    assert!(prefs.is_ok());
    let prefs = prefs.unwrap();
    assert_eq!(prefs.len(), 3);
}

#[test]
fn test_generated_accessors_get() {
    let prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR).unwrap();
    let map: HashMap<String, PrefValue> = gen::TEST_PREF_ACCESSORS
        .iter()
        .map(move |(key, accessor)| {
            let pref_value = (accessor.getter)(&prefs);
            (key.clone(), pref_value)
        })
        .collect();

    assert_eq!(&PrefValue::from("hello"), map.get("pref_string").unwrap());
    assert_eq!(&PrefValue::from(23_i64), map.get("pref_i64").unwrap());
    assert_eq!(&PrefValue::from(1.5_f64), map.get("pref_f64").unwrap());
    assert_eq!(&PrefValue::from(true), map.get("pref_bool").unwrap());
    assert_eq!(
        &PrefValue::from(333_i64),
        map.get("group.nested.nested_i64").unwrap()
    );
    assert_eq!(&PrefValue::from(42_i64), map.get("a.renamed.pref").unwrap());
}

#[test]
fn test_generated_accessors_set() {
    let mut prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR).unwrap();
    let setters: HashMap<String, _> = gen::TEST_PREF_ACCESSORS
        .iter()
        .map(|(key, accessor)| (key.clone(), &accessor.setter))
        .collect();

    (setters.get("pref_string").unwrap())(&mut prefs, PrefValue::Str(String::from("boo")));
    (setters.get("pref_i64").unwrap())(&mut prefs, PrefValue::Int(-25));
    (setters.get("pref_f64").unwrap())(&mut prefs, PrefValue::Float(-1.9));
    (setters.get("pref_bool").unwrap())(&mut prefs, PrefValue::Bool(false));
    (setters.get("group.nested.nested_i64").unwrap())(&mut prefs, PrefValue::Int(10));
    (setters.get("a.renamed.pref").unwrap())(&mut prefs, PrefValue::Int(11));

    assert_eq!("boo", prefs.pref_string);
    assert_eq!(-25, prefs.pref_i64);
    assert_eq!(-1.9, prefs.pref_f64);
    assert_eq!(false, prefs.pref_bool);
    assert_eq!(10, prefs.group.nested.nested_i64);
    assert_eq!(11, prefs.group.nested.renamed);
}

#[test]
fn test_static_struct() {
    let prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR).unwrap();
    assert_eq!("hello", prefs.pref_string);
    assert_eq!(23, prefs.pref_i64);
    assert_eq!(1.5, prefs.pref_f64);
    assert_eq!(true, prefs.pref_bool);
    assert_eq!(333, prefs.group.nested.nested_i64);
    assert_eq!(42, prefs.group.nested.renamed);
}

#[test]
fn test_set_pref() {
    let prefs = Preferences::new(gen::TestPrefs::default(), &gen::TEST_PREF_ACCESSORS);
    assert_eq!(Some(0), prefs.get("group.nested.nested_i64").as_i64());
    let result = prefs.set("group.nested.nested_i64", 1);
    assert_eq!(true, result.is_ok());
    assert_eq!(Some(1), prefs.get("group.nested.nested_i64").as_i64());
    assert_eq!(1, prefs.values().read().unwrap().group.nested.nested_i64);
}

#[test]
fn test_set_unknown_pref_is_err() -> Result<(), Box<dyn Error>> {
    let prefs = Preferences::new(gen::TestPrefs::default(), &gen::TEST_PREF_ACCESSORS);
    let result = prefs.set("unknown_pref", 1);
    assert_eq!(true, result.is_err());
    Ok(())
}

#[test]
fn test_reset_pref() -> Result<(), Box<dyn Error>> {
    let mut def_prefs = gen::TestPrefs::default();
    def_prefs.group.nested.nested_i64 = 999;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    assert_eq!(Some(999), prefs.get("group.nested.nested_i64").as_i64());

    prefs.set("group.nested.nested_i64", 1)?;
    assert_eq!(Some(1), prefs.get("group.nested.nested_i64").as_i64());

    prefs.reset("group.nested.nested_i64")?;
    assert_eq!(Some(999), prefs.get("group.nested.nested_i64").as_i64());
    assert_eq!(999, prefs.values().read().unwrap().group.nested.nested_i64);
    Ok(())
}

#[test]
fn test_default_values() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    assert_eq!(Some(0), prefs.get("default_value").as_i64());
    assert_eq!(Some(555), prefs.get("computed_default_value").as_i64());
    Ok(())
}

#[test]
fn test_override_default_values() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(WITHOUT_DEFAULTS_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    assert_eq!(Some(-1), prefs.get("default_value").as_i64());
    assert_eq!(Some(-1), prefs.get("computed_default_value").as_i64());
    Ok(())
}

#[test]
fn test_update_reset_default_values() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);

    prefs.set("default_value", 99)?;
    prefs.set("computed_default_value", 199)?;
    assert_eq!(Some(99), prefs.get("default_value").as_i64());
    assert_eq!(Some(199), prefs.get("computed_default_value").as_i64());

    prefs.reset("default_value")?;
    prefs.reset("computed_default_value")?;
    assert_eq!(Some(0), prefs.get("default_value").as_i64());
    assert_eq!(Some(555), prefs.get("computed_default_value").as_i64());
    Ok(())
}

#[test]
fn test_update_reset_overridden_default_values() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(WITHOUT_DEFAULTS_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    prefs.set("default_value", 99)?;
    prefs.set("computed_default_value", 199)?;
    assert_eq!(Some(99), prefs.get("default_value").as_i64());
    assert_eq!(Some(199), prefs.get("computed_default_value").as_i64());

    prefs.reset("default_value")?;
    prefs.reset("computed_default_value")?;
    assert_eq!(Some(-1), prefs.get("default_value").as_i64());
    assert_eq!(Some(-1), prefs.get("computed_default_value").as_i64());
    Ok(())
}

#[test]
fn test_user_prefs_override_and_reset() -> Result<(), Box<dyn Error>> {
    let mut def_prefs = gen::TestPrefs::default();
    def_prefs.group.nested.nested_i64 = 999;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);

    prefs.set("group.nested.nested_i64", 45)?;
    assert_eq!(Some(45), prefs.get("group.nested.nested_i64").as_i64());

    prefs.reset("group.nested.nested_i64")?;
    assert_eq!(Some(999), prefs.get("group.nested.nested_i64").as_i64());
    Ok(())
}

#[test]
fn test_reset_all() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    prefs.set_all(read_prefs_map(USER_JSON_STR)?)?;

    let values = prefs.values();
    assert_eq!("bye", values.read().unwrap().pref_string);
    assert_eq!(-1, values.read().unwrap().pref_i64);
    assert_eq!(-1.0, values.read().unwrap().pref_f64);
    assert_eq!(false, values.read().unwrap().pref_bool);
    assert_eq!(-1, values.read().unwrap().group.nested.nested_i64);
    assert_eq!(-1, values.read().unwrap().group.nested.renamed);

    prefs.reset_all();

    let values = prefs.values();
    assert_eq!("hello", values.read().unwrap().pref_string);
    assert_eq!(23, values.read().unwrap().pref_i64);
    assert_eq!(1.5, values.read().unwrap().pref_f64);
    assert_eq!(true, values.read().unwrap().pref_bool);
    assert_eq!(333, values.read().unwrap().group.nested.nested_i64);
    assert_eq!(42, values.read().unwrap().group.nested.renamed);
    Ok(())
}

#[test]
fn test_set_all_from_map() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);
    prefs.set_all(read_prefs_map(USER_JSON_STR)?)?;

    let mut overrides = HashMap::new();
    overrides.insert(String::from("pref_string"), PrefValue::from("new value"));
    overrides.insert(
        String::from("group.nested.nested_i64"),
        PrefValue::from(1001),
    );
    overrides.insert(String::from("a.renamed.pref"), PrefValue::from(47));

    let result = prefs.set_all(overrides.into_iter());
    assert_eq!(true, result.is_ok());

    let values = prefs.values();
    assert_eq!("new value", values.read().unwrap().pref_string);
    assert_eq!(1001, values.read().unwrap().group.nested.nested_i64);
    assert_eq!(47, values.read().unwrap().group.nested.renamed);
    Ok(())
}

#[test]
fn test_set_all_error_on_unknown_field() -> Result<(), Box<dyn Error>> {
    let def_prefs: gen::TestPrefs = serde_json::from_str(DEF_JSON_STR)?;
    let prefs = Preferences::new(def_prefs, &gen::TEST_PREF_ACCESSORS);

    let mut overrides = HashMap::new();
    overrides.insert(String::from("doesnt_exist"), PrefValue::from(1001));

    let result = prefs.set_all(overrides.into_iter());
    assert_eq!(true, result.is_err());
    Ok(())
}

#[cfg(not(target_os = "android"))]
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

static DEF_JSON_STR: &'static str = r#"{
    "pref_string": "hello",
    "pref_i64": 23,
    "pref_f64": 1.5,
    "pref_bool": true,
    "group.nested.nested_i64": 333,
    "a.renamed.pref": 42
}"#;

static USER_JSON_STR: &'static str = r#"{
    "pref_string": "bye",
    "pref_i64": -1,
    "pref_f64": -1.0,
    "pref_bool": false,
    "group.nested.nested_i64": -1,
    "a.renamed.pref": -1
}"#;

static WITHOUT_DEFAULTS_JSON_STR: &'static str = r#"{
    "pref_string": "bye",
    "pref_i64": -1,
    "pref_f64": -1.0,
    "pref_bool": false,
    "group.nested.nested_i64": -1,
    "a.renamed.pref": -1,
    "computed_default_value": -1,
    "default_value": -1
}"#;

mod gen {
    use serde::{Deserialize, Serialize};
    use servo_config::pref_util::{Accessor, PrefValue};
    use servo_config_plugins::build_structs;

    fn compute_default() -> i64 {
        555
    }

    build_structs! {
        accessor_type = Accessor::<TestPrefs, PrefValue>,
        gen_accessors = TEST_PREF_ACCESSORS,
        gen_types = TestPrefs {
            pref_string: String,
            pref_i64: i64,
            pref_f64: f64,
            pref_bool: bool,
            #[serde(default)]
            default_value: i64,
            #[serde(default = "compute_default")]
            computed_default_value: i64,
            group: {
                nested: {
                    nested_i64: i64,
                    #[serde(rename = "a.renamed.pref")]
                    renamed: i64,
                }
            }
        }
    }
}
