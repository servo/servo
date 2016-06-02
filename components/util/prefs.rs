/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use basedir::default_config_dir;
use opts;
use resource_files::resources_dir_path;
use rustc_serialize::json::{Json, ToJson};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, stderr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref PREFS: Arc<Mutex<HashMap<String, Pref>>> = {
        let prefs = read_prefs().unwrap_or(HashMap::new());
        Arc::new(Mutex::new(prefs))
    };
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum PrefValue {
    Boolean(bool),
    String(String),
    Number(f64),
    Missing
}

impl PrefValue {
    pub fn from_json(data: Json) -> Result<PrefValue, ()> {
        let value = match data {
            Json::Boolean(x) => PrefValue::Boolean(x),
            Json::String(x) => PrefValue::String(x),
            Json::F64(x) => PrefValue::Number(x),
            Json::I64(x) => PrefValue::Number(x as f64),
            Json::U64(x) => PrefValue::Number(x as f64),
            _ => return Err(())
        };
        Ok(value)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            PrefValue::Boolean(value) => {
                Some(value)
            },
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match *self {
            PrefValue::String(ref value) => {
                Some(&value)
            },
            _ => None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            PrefValue::Number(x) => Some(x as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            PrefValue::Number(x) => Some(x as u64),
            _ => None,
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
            },
            PrefValue::Number(x) => {
                Json::F64(x)
            },
            PrefValue::Missing => Json::Null
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Pref {
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

    fn from_json(data: Json) -> Result<Pref, ()> {
        let value = try!(PrefValue::from_json(data));
        Ok(Pref::new_default(value))
    }

    pub fn value(&self) -> &Arc<PrefValue> {
        match *self {
            Pref::NoDefault(ref x) => x,
            Pref::WithDefault(ref default, ref override_value) => {
                match *override_value {
                    Some(ref x) => x,
                    None => default
                }
            }
        }
    }

    fn set(&mut self, value: PrefValue) {
        // TODO - this should error if we try to override a pref of one type
        // with a value of a different type
        match *self {
            Pref::NoDefault(ref mut pref_value) => {
                *pref_value = Arc::new(value)
            },
            Pref::WithDefault(_, ref mut override_value) => {
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

pub fn read_prefs_from_file<T>(mut file: T)
    -> Result<HashMap<String, Pref>, ()> where T: Read {
    let json = try!(Json::from_reader(&mut file).or_else(|e| {
        println!("Ignoring invalid JSON in preferences: {:?}.", e);
        Err(())
    }));

    let mut prefs = HashMap::new();
    if let Json::Object(obj) = json {
        for (name, value) in obj.into_iter() {
            match Pref::from_json(value) {
                Ok(x) => {
                    prefs.insert(name, x);
                },
                Err(_) => println!("Ignoring non-boolean/string/i64 preference value for {:?}", name),
            }
        }
    }
    Ok(prefs)
}

pub fn get_cloned() -> HashMap<String, Pref> {
    PREFS.lock().unwrap().clone()
}

pub fn extend_prefs(extension: HashMap<String, Pref>) {
    PREFS.lock().unwrap().extend(extension);
}

pub fn add_user_prefs() {
    match opts::get().config_dir {
        Some(ref config_path) => {
            let mut path = PathBuf::from(config_path);
            init_user_prefs(&mut path);
        }
        None => {
            let mut path = default_config_dir().unwrap();
            if path.join("prefs.json").exists() {
                init_user_prefs(&mut path);
            }
        }
    }
}

fn init_user_prefs(path: &mut PathBuf) {
    path.push("prefs.json");
    if let Ok(file) = File::open(path) {
        if let Ok(prefs) = read_prefs_from_file(file) {
            extend_prefs(prefs);
        }
    } else {
    writeln!(&mut stderr(), "Error opening prefs.json from config directory")
        .expect("failed printing to stderr");
    }
}

fn read_prefs() -> Result<HashMap<String, Pref>, ()> {
    let mut path = resources_dir_path();
    path.push("prefs.json");

    let file = try!(File::open(path).or_else(|e| {
        writeln!(&mut stderr(), "Error opening preferences: {:?}.", e)
            .expect("failed printing to stderr");
        Err(())
    }));

    read_prefs_from_file(file)
}

pub fn get_pref(name: &str) -> Arc<PrefValue> {
    PREFS.lock().unwrap().get(name).map_or(Arc::new(PrefValue::Missing), |x| x.value().clone())
}

pub fn set_pref(name: &str, value: PrefValue) {
    let mut prefs = PREFS.lock().unwrap();
    if let Some(pref) = prefs.get_mut(name) {
        pref.set(value);
        return;
    }
    prefs.insert(name.to_owned(), Pref::new(value));
}

pub fn reset_pref(name: &str) -> Arc<PrefValue> {
    let mut prefs = PREFS.lock().unwrap();
    let result = match prefs.get_mut(name) {
        None => return Arc::new(PrefValue::Missing),
        Some(&mut Pref::NoDefault(_)) => Arc::new(PrefValue::Missing),
        Some(&mut Pref::WithDefault(ref default, ref mut set_value)) => {
            *set_value = None;
            default.clone()
        },
    };
    if *result == PrefValue::Missing {
        prefs.remove(name);
    }
    result
}

pub fn reset_all_prefs() {
    let names = {
        PREFS.lock().unwrap().keys().cloned().collect::<Vec<String>>()
    };
    for name in names.iter() {
        reset_pref(name);
    }
}

pub fn mozbrowser_enabled() -> bool {
    get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false)
}
