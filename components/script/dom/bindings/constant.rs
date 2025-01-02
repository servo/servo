/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebIDL constants.

use std::ffi::CStr;

use js::jsapi::{JSPROP_ENUMERATE, JSPROP_PERMANENT, JSPROP_READONLY};
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UInt32Value};
use js::rust::wrappers::JS_DefineProperty;
use js::rust::HandleObject;

use crate::script_runtime::JSContext;

/// Representation of an IDL constant.
#[derive(Clone)]
pub struct ConstantSpec {
    /// name of the constant.
    pub name: &'static CStr,
    /// value of the constant.
    pub value: ConstantVal,
}

/// Representation of an IDL constant value.
#[derive(Clone)]
#[allow(dead_code)]
pub enum ConstantVal {
    /// `long` constant.
    Int(i32),
    /// `unsigned long` constant.
    Uint(u32),
    /// `double` constant.
    Double(f64),
    /// `boolean` constant.
    Bool(bool),
    /// `null` constant.
    Null,
}

impl ConstantSpec {
    /// Returns a `JSVal` that represents the value of this `ConstantSpec`.
    pub fn get_value(&self) -> JSVal {
        match self.value {
            ConstantVal::Null => NullValue(),
            ConstantVal::Int(i) => Int32Value(i),
            ConstantVal::Uint(u) => UInt32Value(u),
            ConstantVal::Double(d) => DoubleValue(d),
            ConstantVal::Bool(b) => BooleanValue(b),
        }
    }
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
pub fn define_constants(cx: JSContext, obj: HandleObject, constants: &[ConstantSpec]) {
    for spec in constants {
        rooted!(in(*cx) let value = spec.get_value());
        unsafe {
            assert!(JS_DefineProperty(
                *cx,
                obj,
                spec.name.as_ptr(),
                value.handle(),
                (JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT) as u32
            ));
        }
    }
}
