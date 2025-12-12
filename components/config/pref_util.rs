/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PrefValue {
    Float(f64),
    Int(i64),
    UInt(u64),
    Str(String),
    Bool(bool),
    Array(Vec<PrefValue>),
}

impl PrefValue {
    pub fn from_booleanish_str(input: &str) -> Self {
        match input {
            "false" => PrefValue::Bool(false),
            "true" => PrefValue::Bool(true),
            _ => input
                .parse::<i64>()
                .map(PrefValue::Int)
                .or_else(|_| input.parse::<f64>().map(PrefValue::Float))
                .unwrap_or_else(|_| PrefValue::from(input)),
        }
    }
}

impl TryFrom<&Value> for PrefValue {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Err("Cannot turn null into preference".into()),
            Value::Bool(value) => Ok((*value).into()),
            Value::Number(number) => number
                .as_i64()
                .map(Into::into)
                .or_else(|| number.as_f64().map(Into::into))
                .map(Ok)
                .unwrap_or(Err("Could not parse number from JSON".into())),
            Value::String(value) => Ok(value.clone().into()),
            Value::Array(array) => {
                let array = array
                    .iter()
                    .map(TryInto::<PrefValue>::try_into)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(PrefValue::Array(array))
            },
            Value::Object(_) => Err("Cannot turn object into preference".into()),
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
    }
}

macro_rules! impl_from_pref {
    ($($variant: path => $t: ty,)*) => {
        $(
            impl TryFrom<PrefValue> for $t {
                type Error = String;
                fn try_from(other: PrefValue) -> Result<Self, Self::Error> {
                    match other {
                        $variant(value) => Ok(value.into()),
                        _ => Err(format!("Cannot convert {other:?} to {}", std::any::type_name::<$t>())),
                    }
                }
            }
        )+
    }
}

impl_pref_from! {
    f64 => PrefValue::Float,
    i64 => PrefValue::Int,
    u64 => PrefValue::UInt,
    String => PrefValue::Str,
    &str => PrefValue::Str,
    bool => PrefValue::Bool,
}

impl_from_pref! {
    PrefValue::Float => f64,
    PrefValue::Int => i64,
    PrefValue::UInt => u64,
    PrefValue::Str => String,
    PrefValue::Bool => bool,
}

impl From<[f64; 4]> for PrefValue {
    fn from(other: [f64; 4]) -> PrefValue {
        PrefValue::Array(IntoIterator::into_iter(other).map(|v| v.into()).collect())
    }
}

impl From<PrefValue> for [f64; 4] {
    fn from(other: PrefValue) -> [f64; 4] {
        match other {
            PrefValue::Array(values) if values.len() == 4 => {
                let values: Vec<f64> = values
                    .into_iter()
                    .map(TryFrom::try_from)
                    .filter_map(Result::ok)
                    .collect();
                if values.len() == 4 {
                    [values[0], values[1], values[2], values[3]]
                } else {
                    panic!(
                        "Cannot convert PrefValue to {:?}",
                        std::any::type_name::<[f64; 4]>()
                    )
                }
            },
            _ => panic!(
                "Cannot convert {:?} to {:?}",
                other,
                std::any::type_name::<[f64; 4]>()
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pref_value_from_str() {
        let value = PrefValue::from_booleanish_str("21");
        assert_eq!(value, PrefValue::Int(21));

        let value = PrefValue::from_booleanish_str("12.5");
        assert_eq!(value, PrefValue::Float(12.5));

        let value = PrefValue::from_booleanish_str("a string");
        assert_eq!(value, PrefValue::Str("a string".into()));

        let value = PrefValue::from_booleanish_str("false");
        assert_eq!(value, PrefValue::Bool(false));

        let value = PrefValue::from_booleanish_str("true");
        assert_eq!(value, PrefValue::Bool(true));
    }
}
