/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's internal preferences are not C-compatible. To expose the preference to the embedder,
//! we keep a C-compatible copy of the preferences alive (LOCALCPREFS). The embedder can
//! retrieve an array (CPREFS) of struct of pointers (CPrefs) to the C-compatible preferences
//! (LocalCPref).

use crate::simpleservo::{self, PrefValue};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

thread_local! {
    // CPREFS keeps alive a set of CPref that are sent over to the embedder.
    // The CPREFS are structs holding pointers to values held alive by LOCALCPREFS.
    // This is emptied in free_prefs the next time perform_updates is called.
    static CPREFS: RefCell<Vec<CPref>> = RefCell::new(Vec::new());
    static LOCALCPREFS: RefCell<BTreeMap<String, Box<LocalCPref>>> = RefCell::new(BTreeMap::new());
}

struct LocalCPref {
    key: CString,
    value: LocalCPrefValue,
    is_default: bool,
}

#[derive(Debug)]
enum LocalCPrefValue {
    Float(f64),
    Int(i64),
    Str(CString),
    Bool(bool),
    Missing,
}

impl LocalCPrefValue {
    pub fn new(v: &PrefValue) -> LocalCPrefValue {
        match v {
            PrefValue::Float(v) => LocalCPrefValue::Float(*v),
            PrefValue::Int(v) => LocalCPrefValue::Int(*v),
            PrefValue::Str(v) => LocalCPrefValue::Str(CString::new(v.clone()).unwrap()),
            PrefValue::Bool(v) => LocalCPrefValue::Bool(*v),
            PrefValue::Missing => LocalCPrefValue::Missing,
        }
    }
}

#[repr(C)]
pub struct CPrefList {
    pub len: usize,
    pub list: *const CPref,
}

impl CPrefList {
    pub fn convert(&self) -> HashMap<String, PrefValue> {
        let slice = unsafe { std::slice::from_raw_parts(self.list, self.len) };
        slice.iter().map(|cpref| cpref.convert()).collect()
    }
}

#[repr(C)]
pub struct CPref {
    pub pref_type: CPrefType,
    pub key: *const c_char,
    pub value: *const c_void,
    pub is_default: bool,
}

impl CPref {
    fn new(local: &Box<LocalCPref>) -> CPref {
        let (pref_type, value) = match &local.value {
            LocalCPrefValue::Float(v) => (CPrefType::Float, v as *const f64 as *const c_void),
            LocalCPrefValue::Int(v) => (CPrefType::Int, v as *const i64 as *const c_void),
            LocalCPrefValue::Bool(v) => (CPrefType::Bool, v as *const bool as *const c_void),
            LocalCPrefValue::Str(v) => (CPrefType::Str, v.as_ptr() as *const c_void),
            LocalCPrefValue::Missing => (CPrefType::Missing, std::ptr::null()),
        };
        CPref {
            key: local.key.as_ptr(),
            is_default: local.is_default,
            pref_type,
            value,
        }
    }
    fn convert(&self) -> (String, PrefValue) {
        let key = unsafe { CStr::from_ptr(self.key) };
        let key = key.to_str().expect("Can't read string").to_string();
        let value = unsafe {
            match self.pref_type {
                CPrefType::Float => PrefValue::Float(*(self.value as *const f64)),
                CPrefType::Int => PrefValue::Int(*(self.value as *const i64)),
                CPrefType::Bool => PrefValue::Bool(*(self.value as *const bool)),
                CPrefType::Str => PrefValue::Str({
                    let value = CStr::from_ptr(self.value as *const c_char);
                    value.to_str().expect("Can't read string").to_string()
                }),
                CPrefType::Missing => PrefValue::Missing,
            }
        };
        (key, value)
    }
}

#[repr(C)]
pub enum CPrefType {
    Float,
    Int,
    Str,
    Bool,
    Missing,
}

#[no_mangle]
pub extern "C" fn get_pref_as_float(ptr: *const c_void) -> *const f64 {
    ptr as *const f64
}

#[no_mangle]
pub extern "C" fn get_pref_as_int(ptr: *const c_void) -> *const i64 {
    ptr as *const i64
}

#[no_mangle]
pub extern "C" fn get_pref_as_str(ptr: *const c_void) -> *const c_char {
    ptr as *const c_char
}

#[no_mangle]
pub extern "C" fn get_pref_as_bool(ptr: *const c_void) -> *const bool {
    ptr as *const bool
}

#[no_mangle]
pub extern "C" fn reset_all_prefs() {
    debug!("reset_all_prefs");
    simpleservo::reset_all_prefs()
}

#[no_mangle]
pub extern "C" fn reset_pref(key: *const c_char) -> bool {
    debug!("reset_pref");
    let key = unsafe { CStr::from_ptr(key) };
    let key = key.to_str().expect("Can't read string");
    simpleservo::reset_pref(key)
}

#[no_mangle]
pub extern "C" fn get_pref(key: *const c_char) -> CPref {
    debug!("get_pref");
    LOCALCPREFS.with(|localmap| {
        let key = unsafe { CStr::from_ptr(key) };
        let key = key.to_str().expect("Can't read string");
        let (value, is_default) = simpleservo::get_pref(key);
        let local = Box::new(LocalCPref {
            key: CString::new(key).unwrap(),
            value: LocalCPrefValue::new(&value),
            is_default: is_default,
        });
        let cpref = CPref::new(&local);
        localmap.borrow_mut().insert(key.to_string(), local);
        cpref
    })
}

fn set_pref(key: *const c_char, value: PrefValue) -> bool {
    debug!("set_pref");
    let key = unsafe { CStr::from_ptr(key) };
    let key = key.to_str().expect("Can't read string");
    simpleservo::set_pref(key, value).is_ok()
}

#[no_mangle]
pub extern "C" fn set_float_pref(key: *const c_char, value: f64) -> bool {
    set_pref(key, PrefValue::Float(value))
}

#[no_mangle]
pub extern "C" fn set_int_pref(key: *const c_char, value: i64) -> bool {
    set_pref(key, PrefValue::Int(value))
}

#[no_mangle]
pub extern "C" fn set_bool_pref(key: *const c_char, value: bool) -> bool {
    set_pref(key, PrefValue::Bool(value))
}

#[no_mangle]
pub extern "C" fn set_str_pref(key: *const c_char, value: *const c_char) -> bool {
    let value = unsafe { CStr::from_ptr(value) };
    let value = value.to_str().expect("Can't read string").to_string();
    set_pref(key, PrefValue::Str(value))
}

#[no_mangle]
pub extern "C" fn get_prefs() -> CPrefList {
    // Called from any thread
    debug!("get_prefs");
    let map = simpleservo::get_prefs();
    let local: BTreeMap<String, Box<LocalCPref>> = map
        .into_iter()
        .map(|(key, (value, is_default))| {
            let l = Box::new(LocalCPref {
                key: CString::new(key.clone()).unwrap(),
                value: LocalCPrefValue::new(&value),
                is_default: is_default,
            });
            (key, l)
        })
        .collect();

    let ptrs: Vec<CPref> = local.iter().map(|(_, local)| CPref::new(&local)).collect();

    let list = CPrefList {
        len: ptrs.len(),
        list: ptrs.as_ptr(),
    };

    LOCALCPREFS.with(|p| *p.borrow_mut() = local);
    CPREFS.with(|p| *p.borrow_mut() = ptrs);

    list
}

pub(crate) fn free_prefs() {
    LOCALCPREFS.with(|p| p.borrow_mut().clear());
    CPREFS.with(|p| p.borrow_mut().clear());
}
