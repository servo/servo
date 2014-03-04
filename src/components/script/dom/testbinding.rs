/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflector, Reflectable};

#[deriving(Encodable)]
pub struct TestBinding {
    reflector: Reflector,
}

impl TestBinding {
    pub fn BooleanAttribute(&self) -> bool { false }
    pub fn SetBooleanAttribute(&self, _: bool) {}
    pub fn ByteAttribute(&self) -> i8 { 0 }
    pub fn OctetAttribute(&self) -> u8 { 0 }
    pub fn ShortAttribute(&self) -> i16 { 0 }
    pub fn UnsignedShortAttribute(&self) -> u16 { 0 }
    pub fn SetUnsignedShortAttribute(&self, _: u16) {}
    pub fn LongAttribute(&self) -> i32 { 0 }
    pub fn SetLongAttribute(&self, _: i32) {}
    pub fn UnsignedLongAttribute(&self) -> u32 { 0 }
    pub fn SetUnsignedLongAttribute(&self, _: u32) {}
    pub fn LongLongAttribute(&self) -> i64 { 0 }
    pub fn SetLongLongAttribute(&self, _: i64) {}
    pub fn UnsignedLongLongAttribute(&self) -> u64 { 0 }
    pub fn SetUnsignedLongLongAttribute(&self, _: u64) {}
    pub fn FloatAttribute(&self) -> f32 { 0. }
    pub fn SetFloatAttribute(&self, _: f32) {}
    pub fn DoubleAttribute(&self) -> f64 { 0. }
    pub fn SetDoubleAttribute(&self, _: f64) {}
}

impl Reflectable for TestBinding {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector
    }
}
