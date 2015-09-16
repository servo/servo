/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_files::resources_dir_path;
use rustc_serialize::json::{Json, ToJson};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref PREFS: Arc<Mutex<HashMap<String, Pref>>> = {
        let prefs = read_prefs().unwrap_or(HashMap::new());
        Arc::new(Mutex::new(prefs))
    };
}

#[derive(PartialEq, Clone)]
pub enum PrefValue {
    Boolean(bool),
    String(String)
}

impl PrefValue {
    pub fn from_json(data: &Json) -> Result<PrefValue, ()> {
        let value = match data {
            &Json::Boolean(x) => PrefValue::Boolean(x),
            &Json::String(ref x) => PrefValue::String(x.clone()),
            _ => return Err(())
        };
        Ok(value)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            &PrefValue::Boolean(value) => {
                Some(value)
            },
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            &PrefValue::String(ref value) => {
                Some(value.clone())
            },
            _ => None
        }
    }
}

impl ToJson for PrefValue {
    fn to_json(&self) -> Json {
        match *self {
            PrefValue::Boolean(x) => {
                Json::Boolean(x)
            },
            PrefValue::String(ref x) => {
                Json::String(x.clone())
            }
        }
    }
}

enum Pref {
    NoDefault(Arc<PrefValue>),
    WithDefault(Arc<PrefValue>, Option<Arc<PrefValue>>)
}


impl Pref {
    pub fn new(value: PrefValue) -> Pref {
        Pref::NoDefault(Arc::new(value))
    }

    fn new_default(value: PrefValue) -> Pref {
        Pref::WithDefault(Arc::new(value), None)
    }

    fn from_json(data: &Json) -> Result<Pref, ()> {
        let value = try!(PrefValue::from_json(data));
        Ok(Pref::new_default(value))
    }

    pub fn value(&self) -> Arc<PrefValue> {
        match self {
            &Pref::NoDefault(ref x) => x.clone(),
            &Pref::WithDefault(ref default, ref override_value) => {
                match override_value {
                    &Some(ref x) => x.clone(),
                    &None => default.clone()
                }
            }
        }
    }

    fn set(&mut self, value: PrefValue) {
        // TODO - this should error if we try to override a pref of one type
        // with a value of a different type
        match self {
            &mut Pref::NoDefault(ref mut pref_value) => {
                *pref_value = Arc::new(value)
            },
            &mut Pref::WithDefault(_, ref mut override_value) => {
                *override_value = Some(Arc::new(value))
            }
        }
    }
}

impl ToJson for Pref {
    fn to_json(&self) -> Json {
        self.value().to_json()
    }
}

fn read_prefs() -> Result<HashMap<String, Pref>, ()> {
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
            match Pref::from_json(value) {
                Ok(x) => {
                    prefs.insert(name.clone(), x);
                },
                Err(_) => println!("Ignoring non-boolean/string preference value for {:?}", name)
            }
        }
    }
    Ok(prefs)
}

pub fn get_pref(name: &str) -> Option<Arc<PrefValue>> {
    PREFS.lock().unwrap().get(name).map(|x| x.value())
}

pub fn set_pref(name: &str, value: PrefValue) {
    let mut prefs = PREFS.lock().unwrap();
    if prefs.contains_key(name) {
        prefs.get_mut(name).unwrap().set(value)
    } else {
        prefs.insert(name.to_owned(), Pref::new(value));
    }
}

pub fn reset_pref(name: &str) -> Option<Arc<PrefValue>> {
    let mut prefs = PREFS.lock().unwrap();
    let mut rv = None;
    if prefs.contains_key(name) {
        let remove = match prefs.get_mut(name).unwrap() {
            &mut Pref::NoDefault(_) => {
                true
            },
            &mut Pref::WithDefault(_, ref mut set_value) => {
                *set_value = None;
                false
            }
        };
        if remove {
            prefs.remove(name);
        } else {
            rv = Some(prefs.get(name).unwrap().value());
        }
    };
    rv
}

pub fn reset_all_prefs() {
    let names = {
        PREFS.lock().unwrap().keys().map(|x| x.clone()).collect::<Vec<String>>()
    };
    for name in names.iter() {
        reset_pref(name);
    }
}
