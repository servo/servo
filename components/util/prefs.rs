/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_files::resources_dir_path;
use rustc_serialize::json::Json;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref PREFS: Arc<Mutex<HashMap<String, bool>>> = {
        let prefs = read_prefs().unwrap_or(HashMap::new());
        Arc::new(Mutex::new(prefs))
    };
}

fn read_prefs() -> Result<HashMap<String, bool>, ()> {
    let mut path = resources_dir_path();
    path.push("prefs.json");

    let mut file = try!(File::open(path).or_else(|e| {
        println!("Error opening preferences: {:?}.", e);
        Err(())
    }));
    let json = try!(Json::from_reader(&mut file).or_else(|e| {
        println!("Ignoring invalid JSON in preferences: {:?}.", e);
        Err(())
    }));

    let mut prefs = HashMap::new();
    if let Some(obj) = json.as_object() {
        for (name, value) in obj.iter() {
            if let Some(bool_value) = value.as_boolean() {
                prefs.insert(name.clone(), bool_value);
            } else {
                println!("Ignoring non-boolean preference value for {:?}", name);
            }
        }
    }
    Ok(prefs)
}

pub fn get_pref(name: &str, default: bool) -> bool {
    *PREFS.lock().unwrap().get(name).unwrap_or(&default)
}

pub fn set_pref(name: &str, value: bool) {
    let _ = PREFS.lock().unwrap().insert(name.to_owned(), value);
}
