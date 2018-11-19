/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::basedir::default_config_dir;
use crate::opts;
use embedder_traits::resources::{self, Resource};
use serde_json::{self, Value};
use std::borrow::ToOwned;
use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref PREFS: Preferences = {
        let defaults = default_prefs();
        if let Ok(prefs) = read_prefs(&resources::read_string(Resource::Preferences)) {
            defaults.extend(prefs);
        }
        defaults
    };
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PrefValue {
    Boolean(bool),
    String(String),
    Number(f64),
    Missing,
}

impl PrefValue {
    pub fn from_json(data: Value) -> Result<PrefValue, ()> {
        let value = match data {
            Value::Bool(x) => PrefValue::Boolean(x),
            Value::String(x) => PrefValue::String(x),
            Value::Number(x) => {
                if let Some(v) = x.as_f64() {
                    PrefValue::Number(v)
                } else {
                    return Err(());
                }
            },
            _ => return Err(()),
        };
        Ok(value)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            PrefValue::Boolean(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match *self {
            PrefValue::String(ref value) => Some(&value),
            _ => None,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Pref {
    NoDefault(Arc<PrefValue>),
    WithDefault(Arc<PrefValue>, Option<Arc<PrefValue>>),
}

impl Pref {
    pub fn new(value: PrefValue) -> Pref {
        Pref::NoDefault(Arc::new(value))
    }

    fn new_default(value: PrefValue) -> Pref {
        Pref::WithDefault(Arc::new(value), None)
    }

    fn from_json(data: Value) -> Result<Pref, ()> {
        let value = PrefValue::from_json(data)?;
        Ok(Pref::new_default(value))
    }

    pub fn value(&self) -> &Arc<PrefValue> {
        match *self {
            Pref::NoDefault(ref x) => x,
            Pref::WithDefault(ref default, ref override_value) => match *override_value {
                Some(ref x) => x,
                None => default,
            },
        }
    }

    fn set(&mut self, value: PrefValue) {
        // TODO - this should error if we try to override a pref of one type
        // with a value of a different type
        match *self {
            Pref::NoDefault(ref mut pref_value) => *pref_value = Arc::new(value),
            Pref::WithDefault(_, ref mut override_value) => *override_value = Some(Arc::new(value)),
        }
    }
}

pub fn default_prefs() -> Preferences {
    let prefs = Preferences(Arc::new(RwLock::new(HashMap::new())));
    prefs.set(
        "layout.threads",
        PrefValue::Number(max(num_cpus::get() * 3 / 4, 1) as f64),
    );
    prefs
}

pub fn read_prefs(txt: &str) -> Result<HashMap<String, Pref>, ()> {
    let json: Value = serde_json::from_str(txt).or_else(|e| {
        println!("Ignoring invalid JSON in preferences: {:?}.", e);
        Err(())
    })?;

    let mut prefs = HashMap::new();
    if let Value::Object(obj) = json {
        for (name, value) in obj.into_iter() {
            match Pref::from_json(value) {
                Ok(x) => {
                    prefs.insert(name, x);
                },
                Err(_) => println!(
                    "Ignoring non-boolean/string/i64 preference value for {:?}",
                    name
                ),
            }
        }
    }
    Ok(prefs)
}

pub fn add_user_prefs() {
    match opts::get().config_dir {
        Some(ref config_path) => {
            let mut path = PathBuf::from(config_path);
            init_user_prefs(&mut path);
        },
        None => {
            if let Some(mut path) = default_config_dir() {
                if path.join("prefs.json").exists() {
                    init_user_prefs(&mut path);
                }
            }
        },
    }
}

fn init_user_prefs(path: &mut PathBuf) {
    path.push("prefs.json");
    if let Ok(mut file) = File::open(path) {
        let mut txt = String::new();
        file.read_to_string(&mut txt).expect("Can't read use prefs");
        if let Ok(prefs) = read_prefs(&txt) {
            PREFS.extend(prefs);
        }
    } else {
        writeln!(
            &mut stderr(),
            "Error opening prefs.json from config directory"
        )
        .expect("failed printing to stderr");
    }
}

pub struct Preferences(Arc<RwLock<HashMap<String, Pref>>>);

impl Preferences {
    pub fn get(&self, name: &str) -> Arc<PrefValue> {
        self.0
            .read()
            .unwrap()
            .get(name)
            .map_or(Arc::new(PrefValue::Missing), |x| x.value().clone())
    }

    pub fn cloned(&self) -> HashMap<String, Pref> {
        self.0.read().unwrap().clone()
    }

    pub fn set(&self, name: &str, value: PrefValue) {
        let mut prefs = self.0.write().unwrap();
        if let Some(pref) = prefs.get_mut(name) {
            pref.set(value);
            return;
        }
        prefs.insert(name.to_owned(), Pref::new(value));
    }

    pub fn reset(&self, name: &str) -> Arc<PrefValue> {
        let mut prefs = self.0.write().unwrap();
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

    pub fn reset_all(&self) {
        let names = {
            self.0
                .read()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<String>>()
        };
        for name in names.iter() {
            self.reset(name);
        }
    }

    pub fn extend(&self, extension: HashMap<String, Pref>) {
        self.0.write().unwrap().extend(extension);
    }

    pub fn is_webvr_enabled(&self) -> bool {
        self.get("dom.webvr.enabled").as_boolean().unwrap_or(false)
    }

    pub fn is_dom_to_texture_enabled(&self) -> bool {
        self.get("dom.webgl.dom_to_texture.enabled")
            .as_boolean()
            .unwrap_or(false)
    }

    pub fn is_webgl2_enabled(&self) -> bool {
        self.get("dom.webgl2.enabled").as_boolean().unwrap_or(false)
    }
}
