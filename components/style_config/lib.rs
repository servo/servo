/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    static ref PREFS: Preferences = Preferences::default();
}

#[derive(Debug, Default)]
pub struct Preferences {
    bool_prefs: RwLock<HashMap<String, bool>>,
    i32_prefs: RwLock<HashMap<String, i32>>,
}

impl Preferences {
    pub fn get_bool(&self, key: &str) -> bool {
        let prefs = self.bool_prefs.write().expect("RwLock is poisoned");
        *prefs.get(key).unwrap_or(&false)
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        let prefs = self.i32_prefs.write().expect("RwLock is poisoned");
        *prefs.get(key).unwrap_or(&0)
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        let mut prefs = self.bool_prefs.write().expect("RwLock is poisoned");
        if let Some(pref) = prefs.get_mut(key) {
            *pref = value;
        } else {
            prefs.insert(key.to_owned(), value);
        }
    }

    pub fn set_i32(&self, key: &str, value: i32) {
        let mut prefs = self.i32_prefs.write().expect("RwLock is poisoned");
        if let Some(pref) = prefs.get_mut(key) {
            *pref = value;
        } else {
            prefs.insert(key.to_owned(), value);
        }
    }
}

pub fn get_bool(key: &str) -> bool {
    PREFS.get_bool(key)
}

pub fn get_i32(key: &str) -> i32 {
    PREFS.get_i32(key)
}

pub fn set_bool(key: &str, value: bool) {
    PREFS.set_bool(key, value)
}

pub fn set_i32(key: &str, value: i32) {
    PREFS.set_i32(key, value)
}
