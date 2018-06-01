/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! WebIDL constants.

use js::jsapi::{JSContext, JSPROP_ENUMERATE, JSPROP_PERMANENT};
use js::jsapi::JSPROP_READONLY;
use js::jsval::{BooleanValue, DoubleValue, Int32Value, JSVal, NullValue, UInt32Value};
use js::rust::HandleObject;
use js::rust::wrappers::JS_DefineProperty;
use libc;

/// Representation of an IDL constant.
#[derive(Clone)]
pub struct ConstantSpec {
    /// name of the constant.
    pub name: &'static [u8],
    /// value of the constant.
    pub value: ConstantVal,
}

/// Representation of an IDL constant value.
#[derive(Clone)]
#[allow(dead_code)]
pub enum ConstantVal {
    /// `long` constant.
    IntVal(i32),
    /// `unsigned long` constant.
    UintVal(u32),
    /// `double` constant.
    DoubleVal(f64),
    /// `boolean` constant.
    BoolVal(bool),
    /// `null` constant.
    NullVal,
}

impl ConstantSpec {
    /// Returns a `JSVal` that represents the value of this `ConstantSpec`.
    pub fn get_value(&self) -> JSVal {
        match self.value {
            ConstantVal::NullVal => NullValue(),
            ConstantVal::IntVal(i) => Int32Value(i),
            ConstantVal::UintVal(u) => UInt32Value(u),
            ConstantVal::DoubleVal(d) => DoubleValue(d),
            ConstantVal::BoolVal(b) => BooleanValue(b),
        }
    }
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
pub unsafe fn define_constants(
        cx: *mut JSContext,
        obj: HandleObject,
        constants: &[ConstantSpec]) {
    for spec in constants {
        rooted!(in(cx) let value = spec.get_value());
        assert!(JS_DefineProperty(cx,
                                  obj,
                                  spec.name.as_ptr() as *const libc::c_char,
                                  value.handle(),
                                  (JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT) as u32));
    }
}
