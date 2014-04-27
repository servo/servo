/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::codegen::TestBindingBinding;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong;
use self::TestBindingBinding::TestEnum;
use self::TestBindingBinding::TestEnumValues::_empty;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::blob::Blob;
use dom::window::Window;
use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};

#[deriving(Encodable)]
pub struct TestBinding {
    pub reflector: Reflector,
    pub window: JS<Window>,
}

impl TestBinding {
    pub fn BooleanAttribute(&self) -> bool { false }
    pub fn SetBooleanAttribute(&self, _: bool) {}
    pub fn ByteAttribute(&self) -> i8 { 0 }
    pub fn SetByteAttribute(&self, _: i8) {}
    pub fn OctetAttribute(&self) -> u8 { 0 }
    pub fn SetOctetAttribute(&self, _: u8) {}
    pub fn ShortAttribute(&self) -> i16 { 0 }
    pub fn SetShortAttribute(&self, _: i16) {}
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
    pub fn StringAttribute(&self) -> DOMString { ~"" }
    pub fn SetStringAttribute(&self, _: DOMString) {}
    pub fn EnumAttribute(&self) -> TestEnum { _empty }
    pub fn SetEnumAttribute(&self, _: TestEnum) {}
    pub fn InterfaceAttribute(&self) -> JS<Blob> { Blob::new(&self.window) }
    pub fn SetInterfaceAttribute(&self, _: &JS<Blob>) {}
    pub fn AnyAttribute(&self, _: *JSContext) -> JSVal { NullValue() }
    pub fn SetAnyAttribute(&self, _: *JSContext, _: JSVal) {}

    pub fn GetBooleanAttributeNullable(&self) -> Option<bool> { Some(false) }
    pub fn SetBooleanAttributeNullable(&self, _: Option<bool>) {}
    pub fn GetByteAttributeNullable(&self) -> Option<i8> { Some(0) }
    pub fn SetByteAttributeNullable(&self, _: Option<i8>) {}
    pub fn GetOctetAttributeNullable(&self) -> Option<u8> { Some(0) }
    pub fn SetOctetAttributeNullable(&self, _: Option<u8>) {}
    pub fn GetShortAttributeNullable(&self) -> Option<i16> { Some(0) }
    pub fn SetShortAttributeNullable(&self, _: Option<i16>) {}
    pub fn GetUnsignedShortAttributeNullable(&self) -> Option<u16> { Some(0) }
    pub fn SetUnsignedShortAttributeNullable(&self, _: Option<u16>) {}
    pub fn GetLongAttributeNullable(&self) -> Option<i32> { Some(0) }
    pub fn SetLongAttributeNullable(&self, _: Option<i32>) {}
    pub fn GetUnsignedLongAttributeNullable(&self) -> Option<u32> { Some(0) }
    pub fn SetUnsignedLongAttributeNullable(&self, _: Option<u32>) {}
    pub fn GetLongLongAttributeNullable(&self) -> Option<i64> { Some(0) }
    pub fn SetLongLongAttributeNullable(&self, _: Option<i64>) {}
    pub fn GetUnsignedLongLongAttributeNullable(&self) -> Option<u64> { Some(0) }
    pub fn SetUnsignedLongLongAttributeNullable(&self, _: Option<u64>) {}
    pub fn GetFloatAttributeNullable(&self) -> Option<f32> { Some(0.) }
    pub fn SetFloatAttributeNullable(&self, _: Option<f32>) {}
    pub fn GetDoubleAttributeNullable(&self) -> Option<f64> { Some(0.) }
    pub fn SetDoubleAttributeNullable(&self, _: Option<f64>) {}
    pub fn GetStringAttributeNullable(&self) -> Option<DOMString> { Some(~"") }
    pub fn SetStringAttributeNullable(&self, _: Option<DOMString>) {}
    pub fn GetEnumAttributeNullable(&self) -> Option<TestEnum> { Some(_empty) }
    pub fn GetInterfaceAttributeNullable(&self) -> Option<JS<Blob>> { Some(Blob::new(&self.window)) }
    pub fn SetInterfaceAttributeNullable(&self, _: Option<JS<Blob>>) {}

    pub fn PassBoolean(&self, _: bool) {}
    pub fn PassByte(&self, _: i8) {}
    pub fn PassOctet(&self, _: u8) {}
    pub fn PassShort(&self, _: i16) {}
    pub fn PassUnsignedShort(&self, _: u16) {}
    pub fn PassLong(&self, _: i32) {}
    pub fn PassUnsignedLong(&self, _: u32) {}
    pub fn PassLongLong(&self, _: i64) {}
    pub fn PassUnsignedLongLong(&self, _: u64) {}
    pub fn PassFloat(&self, _: f32) {}
    pub fn PassDouble(&self, _: f64) {}
    pub fn PassString(&self, _: DOMString) {}
    pub fn PassEnum(&self, _: TestEnum) {}
    pub fn PassInterface(&self, _: &JS<Blob>) {}
    pub fn PassUnion(&self, _: HTMLElementOrLong) {}
    pub fn PassAny(&self, _: *JSContext, _: JSVal) {}

    pub fn PassNullableBoolean(&self, _: Option<bool>) {}
    pub fn PassNullableByte(&self, _: Option<i8>) {}
    pub fn PassNullableOctet(&self, _: Option<u8>) {}
    pub fn PassNullableShort(&self, _: Option<i16>) {}
    pub fn PassNullableUnsignedShort(&self, _: Option<u16>) {}
    pub fn PassNullableLong(&self, _: Option<i32>) {}
    pub fn PassNullableUnsignedLong(&self, _: Option<u32>) {}
    pub fn PassNullableLongLong(&self, _: Option<i64>) {}
    pub fn PassNullableUnsignedLongLong(&self, _: Option<u64>) {}
    pub fn PassNullableFloat(&self, _: Option<f32>) {}
    pub fn PassNullableDouble(&self, _: Option<f64>) {}
    pub fn PassNullableString(&self, _: Option<DOMString>) {}
    // pub fn PassNullableEnum(&self, _: Option<TestEnum>) {}
    pub fn PassNullableInterface(&self, _: Option<JS<Blob>>) {}
    pub fn PassNullableUnion(&self, _: Option<HTMLElementOrLong>) {}
    pub fn PassNullableAny(&self, _: *JSContext, _: Option<JSVal>) {}

    pub fn PassOptionalBoolean(&self, _: Option<bool>) {}
    pub fn PassOptionalByte(&self, _: Option<i8>) {}
    pub fn PassOptionalOctet(&self, _: Option<u8>) {}
    pub fn PassOptionalShort(&self, _: Option<i16>) {}
    pub fn PassOptionalUnsignedShort(&self, _: Option<u16>) {}
    pub fn PassOptionalLong(&self, _: Option<i32>) {}
    pub fn PassOptionalUnsignedLong(&self, _: Option<u32>) {}
    pub fn PassOptionalLongLong(&self, _: Option<i64>) {}
    pub fn PassOptionalUnsignedLongLong(&self, _: Option<u64>) {}
    pub fn PassOptionalFloat(&self, _: Option<f32>) {}
    pub fn PassOptionalDouble(&self, _: Option<f64>) {}
    pub fn PassOptionalString(&self, _: Option<DOMString>) {}
    // pub fn PassOptionalEnum(&self, _: Option<TestEnum>) {}
    pub fn PassOptionalInterface(&self, _: Option<JS<Blob>>) {}
    // pub fn PassOptionalUnion(&self, _: Option<HTMLElementOrLong>) {}
    pub fn PassOptionalAny(&self, _: *JSContext, _: Option<JSVal>) {}

    pub fn PassOptionalNullableBoolean(&self, _: Option<Option<bool>>) {}
    pub fn PassOptionalNullableByte(&self, _: Option<Option<i8>>) {}
    pub fn PassOptionalNullableOctet(&self, _: Option<Option<u8>>) {}
    pub fn PassOptionalNullableShort(&self, _: Option<Option<i16>>) {}
    pub fn PassOptionalNullableUnsignedShort(&self, _: Option<Option<u16>>) {}
    pub fn PassOptionalNullableLong(&self, _: Option<Option<i32>>) {}
    pub fn PassOptionalNullableUnsignedLong(&self, _: Option<Option<u32>>) {}
    pub fn PassOptionalNullableLongLong(&self, _: Option<Option<i64>>) {}
    pub fn PassOptionalNullableUnsignedLongLong(&self, _: Option<Option<u64>>) {}
    pub fn PassOptionalNullableFloat(&self, _: Option<Option<f32>>) {}
    pub fn PassOptionalNullableDouble(&self, _: Option<Option<f64>>) {}
    pub fn PassOptionalNullableString(&self, _: Option<Option<DOMString>>) {}
    // pub fn PassOptionalNullableEnum(&self, _: Option<Option<TestEnum>>) {}
    // pub fn PassOptionalNullableInterface(&self, _: Option<Option<JS<Blob>>>) {}
    // pub fn PassOptionalNullableUnion(&self, _: Option<Option<HTMLElementOrLong>>) {}

    pub fn PassOptionalBooleanWithDefault(&self, _: bool) {}
    pub fn PassOptionalByteWithDefault(&self, _: i8) {}
    pub fn PassOptionalOctetWithDefault(&self, _: u8) {}
    pub fn PassOptionalShortWithDefault(&self, _: i16) {}
    pub fn PassOptionalUnsignedShortWithDefault(&self, _: u16) {}
    pub fn PassOptionalLongWithDefault(&self, _: i32) {}
    pub fn PassOptionalUnsignedLongWithDefault(&self, _: u32) {}
    pub fn PassOptionalLongLongWithDefault(&self, _: i64) {}
    pub fn PassOptionalUnsignedLongLongWithDefault(&self, _: u64) {}
    pub fn PassOptionalStringWithDefault(&self, _: DOMString) {}
    pub fn PassOptionalEnumWithDefault(&self, _: TestEnum) {}

    pub fn PassOptionalNullableBooleanWithDefault(&self, _: Option<bool>) {}
    pub fn PassOptionalNullableByteWithDefault(&self, _: Option<i8>) {}
    pub fn PassOptionalNullableOctetWithDefault(&self, _: Option<u8>) {}
    pub fn PassOptionalNullableShortWithDefault(&self, _: Option<i16>) {}
    pub fn PassOptionalNullableUnsignedShortWithDefault(&self, _: Option<u16>) {}
    pub fn PassOptionalNullableLongWithDefault(&self, _: Option<i32>) {}
    pub fn PassOptionalNullableUnsignedLongWithDefault(&self, _: Option<u32>) {}
    pub fn PassOptionalNullableLongLongWithDefault(&self, _: Option<i64>) {}
    pub fn PassOptionalNullableUnsignedLongLongWithDefault(&self, _: Option<u64>) {}
    pub fn PassOptionalNullableFloatWithDefault(&self, _: Option<f32>) {}
    pub fn PassOptionalNullableDoubleWithDefault(&self, _: Option<f64>) {}
    pub fn PassOptionalNullableStringWithDefault(&self, _: Option<DOMString>) {}
    // pub fn PassOptionalNullableEnumWithDefault(&self, _: Option<TestEnum>) {}
    pub fn PassOptionalNullableInterfaceWithDefault(&self, _: Option<JS<Blob>>) {}
    pub fn PassOptionalNullableUnionWithDefault(&self, _: Option<HTMLElementOrLong>) {}
    pub fn PassOptionalAnyWithDefault(&self, _: *JSContext, _: JSVal) {}

    pub fn PassOptionalNullableBooleanWithNonNullDefault(&self, _: Option<bool>) {}
    pub fn PassOptionalNullableByteWithNonNullDefault(&self, _: Option<i8>) {}
    pub fn PassOptionalNullableOctetWithNonNullDefault(&self, _: Option<u8>) {}
    pub fn PassOptionalNullableShortWithNonNullDefault(&self, _: Option<i16>) {}
    pub fn PassOptionalNullableUnsignedShortWithNonNullDefault(&self, _: Option<u16>) {}
    pub fn PassOptionalNullableLongWithNonNullDefault(&self, _: Option<i32>) {}
    pub fn PassOptionalNullableUnsignedLongWithNonNullDefault(&self, _: Option<u32>) {}
    pub fn PassOptionalNullableLongLongWithNonNullDefault(&self, _: Option<i64>) {}
    pub fn PassOptionalNullableUnsignedLongLongWithNonNullDefault(&self, _: Option<u64>) {}
    // pub fn PassOptionalNullableFloatWithNonNullDefault(&self, _: Option<f32>) {}
    // pub fn PassOptionalNullableDoubleWithNonNullDefault(&self, _: Option<f64>) {}
    pub fn PassOptionalNullableStringWithNonNullDefault(&self, _: Option<DOMString>) {}
    // pub fn PassOptionalNullableEnumWithNonNullDefault(&self, _: Option<TestEnum>) {}
}

impl Reflectable for TestBinding {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector
    }
}
