/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PrefValue {
    Float(f64),
    Int(i64),
    Str(String),
    Bool(bool),
    Missing,
}

impl PrefValue {
    pub fn as_str(&self) -> Option<&str> {
        if let PrefValue::Str(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let PrefValue::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let PrefValue::Float(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let PrefValue::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn is_missing(&self) -> bool {
        match self {
            PrefValue::Missing => true,
            _ => false,
        }
    }

    pub fn from_json_value(value: &Value) -> Option<Self> {
        match value {
            Value::Bool(b) => Some(PrefValue::Bool(*b)),
            Value::Number(n) if n.is_i64() => Some(PrefValue::Int(n.as_i64().unwrap())),
            Value::Number(n) if n.is_f64() => Some(PrefValue::Float(n.as_f64().unwrap())),
            Value::String(s) => Some(PrefValue::Str(s.to_owned())),
            _ => None,
        }
    }
}

impl FromStr for PrefValue {
    type Err = PrefError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "false" {
            Ok(PrefValue::Bool(false))
        } else if s == "true" {
            Ok(PrefValue::Bool(true))
        } else if let Ok(float) = s.parse::<f64>() {
            Ok(PrefValue::Float(float))
        } else if let Ok(integer) = s.parse::<i64>() {
            Ok(PrefValue::Int(integer))
        } else {
            Ok(PrefValue::from(s))
        }
    }
}

macro_rules! impl_pref_from {
    ($($t: ty => $variant: path,)*) => {
        $(
            impl From<$t> for PrefValue {
                fn from(other: $t) -> Self {
                    $variant(other.into())
                }
            }
        )+
        $(
            impl From<Option<$t>> for PrefValue {
                fn from(other: Option<$t>) -> Self {
                    other.map(|val| $variant(val.into())).unwrap_or(PrefValue::Missing)
                }
            }
        )+
    }
}

macro_rules! impl_from_pref {
    ($($variant: path => $t: ty,)*) => {
        $(
            impl From<PrefValue> for $t {
                #[allow(unsafe_code)]
                fn from(other: PrefValue) -> Self {
                    if let $variant(value) = other {
                        value.into()
                    } else {
                        panic!(
                            format!("Cannot convert {:?} to {:?}",
                                other,
                                unsafe { std::intrinsics::type_name::<$t>() }
                            )
                        );
                    }
                }
            }
        )+
        $(
            impl From<PrefValue> for Option<$t> {
                fn from(other: PrefValue) -> Self {
                    if let PrefValue::Missing = other {
                        None
                    } else {
                        Some(other.into())
                    }
                }
            }
        )+
    }
}

impl_pref_from! {
    f64 => PrefValue::Float,
    i64 => PrefValue::Int,
    String => PrefValue::Str,
    &str => PrefValue::Str,
    bool => PrefValue::Bool,
}

impl_from_pref! {
    PrefValue::Float => f64,
    PrefValue::Int => i64,
    PrefValue::Str => String,
    PrefValue::Bool => bool,
}

#[derive(Debug)]
pub enum PrefError {
    NoSuchPref(String),
    InvalidValue(String),
    JsonParseErr(serde_json::error::Error),
}

impl fmt::Display for PrefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrefError::NoSuchPref(s) | PrefError::InvalidValue(s) => f.write_str(&s),
            PrefError::JsonParseErr(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for PrefError {}

pub struct Accessor<P, V> {
    pub getter: Box<Fn(&P) -> V + Sync>,
    pub setter: Box<Fn(&mut P, V) + Sync>,
}

impl<P, V> Accessor<P, V> {
    pub fn new<G, S>(getter: G, setter: S) -> Self
    where
        G: Fn(&P) -> V + Sync + 'static,
        S: Fn(&mut P, V) + Sync + 'static,
    {
        Accessor {
            getter: Box::new(getter),
            setter: Box::new(setter),
        }
    }
}

pub struct Preferences<'m, P> {
    user_prefs: Arc<RwLock<P>>,
    default_prefs: P,
    accessors: &'m HashMap<String, Accessor<P, PrefValue>>,
}

impl<'m, P: Clone> Preferences<'m, P> {
    /// Create a new `Preferences` object. The values provided in `default_prefs` are immutable and
    /// can always be restored using `reset` or `reset_all`.
    pub fn new(default_prefs: P, accessors: &'m HashMap<String, Accessor<P, PrefValue>>) -> Self {
        Self {
            user_prefs: Arc::new(RwLock::new(default_prefs.clone())),
            default_prefs,
            accessors,
        }
    }

    /// Access to the data structure holding the preference values.
    pub fn values(&self) -> Arc<RwLock<P>> {
        Arc::clone(&self.user_prefs)
    }

    /// Retrieve a preference using its key
    pub fn get(&self, key: &str) -> PrefValue {
        if let Some(accessor) = self.accessors.get(key) {
            let prefs = self.user_prefs.read().unwrap();
            (accessor.getter)(&prefs)
        } else {
            PrefValue::Missing
        }
    }

    /// Creates an iterator over all keys and values
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (String, PrefValue)> + 'a {
        let prefs = self.user_prefs.read().unwrap();
        self.accessors
            .iter()
            .map(move |(k, accessor)| (k.clone(), (accessor.getter)(&prefs)))
    }

    /// Creates an iterator over all keys
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a str> + 'a {
        self.accessors.keys().map(String::as_str)
    }

    fn set_inner<V>(&self, key: &str, mut prefs: &mut P, val: V) -> Result<(), PrefError>
    where
        V: Into<PrefValue>,
    {
        if let Some(accessor) = self.accessors.get(key) {
            Ok((accessor.setter)(&mut prefs, val.into()))
        } else {
            Err(PrefError::NoSuchPref(String::from(key)))
        }
    }

    /// Set a new value for a preference, using its key.
    pub fn set<V>(&self, key: &str, val: V) -> Result<(), PrefError>
    where
        V: Into<PrefValue>,
    {
        let mut prefs = self.user_prefs.write().unwrap();
        self.set_inner(key, &mut prefs, val)
    }

    pub fn set_all<M>(&self, values: M) -> Result<(), PrefError>
    where
        M: IntoIterator<Item = (String, PrefValue)>,
    {
        let mut prefs = self.user_prefs.write().unwrap();
        for (k, v) in values.into_iter() {
            self.set_inner(&k, &mut prefs, v)?;
        }
        Ok(())
    }

    pub fn reset(&self, key: &str) -> Result<PrefValue, PrefError> {
        if let Some(accessor) = self.accessors.get(key) {
            let mut prefs = self.user_prefs.write().unwrap();
            let old_pref = (accessor.getter)(&prefs);
            let default_pref = (accessor.getter)(&self.default_prefs);
            (accessor.setter)(&mut prefs, default_pref);
            Ok(old_pref)
        } else {
            Err(PrefError::NoSuchPref(String::from(key)))
        }
    }

    pub fn reset_all(&self) {
        *self.user_prefs.write().unwrap() = self.default_prefs.clone();
    }
}
