/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TestBindingBinding::TestEnum;
use dom::bindings::codegen::Bindings::TestBindingBinding::TestEnumValues::_empty;
use dom::bindings::codegen::UnionTypes::BlobOrString::BlobOrString;
use dom::bindings::codegen::UnionTypes::EventOrString::{EventOrString, eString};
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong::{HTMLElementOrLong, eLong};
use dom::bindings::global::GlobalField;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::blob::Blob;
use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};

#[deriving(Encodable)]
pub struct TestBinding {
    reflector: Reflector,
    global: GlobalField,
}

pub trait TestBindingMethods {
    fn BooleanAttribute(&self) -> bool { false }
    fn SetBooleanAttribute(&self, _: bool) {}
    fn ByteAttribute(&self) -> i8 { 0 }
    fn SetByteAttribute(&self, _: i8) {}
    fn OctetAttribute(&self) -> u8 { 0 }
    fn SetOctetAttribute(&self, _: u8) {}
    fn ShortAttribute(&self) -> i16 { 0 }
    fn SetShortAttribute(&self, _: i16) {}
    fn UnsignedShortAttribute(&self) -> u16 { 0 }
    fn SetUnsignedShortAttribute(&self, _: u16) {}
    fn LongAttribute(&self) -> i32 { 0 }
    fn SetLongAttribute(&self, _: i32) {}
    fn UnsignedLongAttribute(&self) -> u32 { 0 }
    fn SetUnsignedLongAttribute(&self, _: u32) {}
    fn LongLongAttribute(&self) -> i64 { 0 }
    fn SetLongLongAttribute(&self, _: i64) {}
    fn UnsignedLongLongAttribute(&self) -> u64 { 0 }
    fn SetUnsignedLongLongAttribute(&self, _: u64) {}
    fn FloatAttribute(&self) -> f32 { 0. }
    fn SetFloatAttribute(&self, _: f32) {}
    fn DoubleAttribute(&self) -> f64 { 0. }
    fn SetDoubleAttribute(&self, _: f64) {}
    fn StringAttribute(&self) -> DOMString { "".to_string() }
    fn SetStringAttribute(&self, _: DOMString) {}
    fn ByteStringAttribute(&self) -> ByteString { ByteString::new(vec!()) }
    fn SetByteStringAttribute(&self, _: ByteString) {}
    fn EnumAttribute(&self) -> TestEnum { _empty }
    fn SetEnumAttribute(&self, _: TestEnum) {}
    fn InterfaceAttribute(&self) -> Temporary<Blob>;
    fn SetInterfaceAttribute(&self, _: &JSRef<Blob>) {}
    fn UnionAttribute(&self) -> HTMLElementOrLong { eLong(0) }
    fn SetUnionAttribute(&self, _: HTMLElementOrLong) {}
    fn Union2Attribute(&self) -> EventOrString { eString("".to_string()) }
    fn SetUnion2Attribute(&self, _: EventOrString) {}
    fn AnyAttribute(&self, _: *mut JSContext) -> JSVal { NullValue() }
    fn SetAnyAttribute(&self, _: *mut JSContext, _: JSVal) {}

    fn GetBooleanAttributeNullable(&self) -> Option<bool> { Some(false) }
    fn SetBooleanAttributeNullable(&self, _: Option<bool>) {}
    fn GetByteAttributeNullable(&self) -> Option<i8> { Some(0) }
    fn SetByteAttributeNullable(&self, _: Option<i8>) {}
    fn GetOctetAttributeNullable(&self) -> Option<u8> { Some(0) }
    fn SetOctetAttributeNullable(&self, _: Option<u8>) {}
    fn GetShortAttributeNullable(&self) -> Option<i16> { Some(0) }
    fn SetShortAttributeNullable(&self, _: Option<i16>) {}
    fn GetUnsignedShortAttributeNullable(&self) -> Option<u16> { Some(0) }
    fn SetUnsignedShortAttributeNullable(&self, _: Option<u16>) {}
    fn GetLongAttributeNullable(&self) -> Option<i32> { Some(0) }
    fn SetLongAttributeNullable(&self, _: Option<i32>) {}
    fn GetUnsignedLongAttributeNullable(&self) -> Option<u32> { Some(0) }
    fn SetUnsignedLongAttributeNullable(&self, _: Option<u32>) {}
    fn GetLongLongAttributeNullable(&self) -> Option<i64> { Some(0) }
    fn SetLongLongAttributeNullable(&self, _: Option<i64>) {}
    fn GetUnsignedLongLongAttributeNullable(&self) -> Option<u64> { Some(0) }
    fn SetUnsignedLongLongAttributeNullable(&self, _: Option<u64>) {}
    fn GetFloatAttributeNullable(&self) -> Option<f32> { Some(0.) }
    fn SetFloatAttributeNullable(&self, _: Option<f32>) {}
    fn GetDoubleAttributeNullable(&self) -> Option<f64> { Some(0.) }
    fn SetDoubleAttributeNullable(&self, _: Option<f64>) {}
    fn GetByteStringAttributeNullable(&self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn SetByteStringAttributeNullable(&self, _: Option<ByteString>) {}
    fn GetStringAttributeNullable(&self) -> Option<DOMString> { Some("".to_string()) }
    fn SetStringAttributeNullable(&self, _: Option<DOMString>) {}
    fn GetEnumAttributeNullable(&self) -> Option<TestEnum> { Some(_empty) }
    fn GetInterfaceAttributeNullable(&self) -> Option<Temporary<Blob>>;
    fn SetInterfaceAttributeNullable(&self, _: Option<JSRef<Blob>>) {}
    fn GetUnionAttributeNullable(&self) -> Option<HTMLElementOrLong> { Some(eLong(0)) }
    fn SetUnionAttributeNullable(&self, _: Option<HTMLElementOrLong>) {}
    fn GetUnion2AttributeNullable(&self) -> Option<EventOrString> { Some(eString("".to_string())) }
    fn SetUnion2AttributeNullable(&self, _: Option<EventOrString>) {}
    fn ReceiveVoid(&self) -> () {}
    fn ReceiveBoolean(&self) -> bool { false }
    fn ReceiveByte(&self) -> i8 { 0 }
    fn ReceiveOctet(&self) -> u8 { 0 }
    fn ReceiveShort(&self) -> i16 { 0 }
    fn ReceiveUnsignedShort(&self) -> u16 { 0 }
    fn ReceiveLong(&self) -> i32 { 0 }
    fn ReceiveUnsignedLong(&self) -> u32 { 0 }
    fn ReceiveLongLong(&self) -> i64 { 0 }
    fn ReceiveUnsignedLongLong(&self) -> u64 { 0 }
    fn ReceiveFloat(&self) -> f32 { 0. }
    fn ReceiveDouble(&self) -> f64 { 0. }
    fn ReceiveString(&self) -> DOMString { "".to_string() }
    fn ReceiveByteString(&self) -> ByteString { ByteString::new(vec!()) }
    fn ReceiveEnum(&self) -> TestEnum { _empty }
    fn ReceiveInterface(&self) -> Temporary<Blob>;
    fn ReceiveAny(&self, _: *mut JSContext) -> JSVal { NullValue() }
    fn ReceiveUnion(&self) -> HTMLElementOrLong { eLong(0) }
    fn ReceiveUnion2(&self) -> EventOrString { eString("".to_string()) }

    fn ReceiveNullableBoolean(&self) -> Option<bool> { Some(false) }
    fn ReceiveNullableByte(&self) -> Option<i8> { Some(0) }
    fn ReceiveNullableOctet(&self) -> Option<u8> { Some(0) }
    fn ReceiveNullableShort(&self) -> Option<i16> { Some(0) }
    fn ReceiveNullableUnsignedShort(&self) -> Option<u16> { Some(0) }
    fn ReceiveNullableLong(&self) -> Option<i32> { Some(0) }
    fn ReceiveNullableUnsignedLong(&self) -> Option<u32> { Some(0) }
    fn ReceiveNullableLongLong(&self) -> Option<i64> { Some(0) }
    fn ReceiveNullableUnsignedLongLong(&self) -> Option<u64> { Some(0) }
    fn ReceiveNullableFloat(&self) -> Option<f32> { Some(0.) }
    fn ReceiveNullableDouble(&self) -> Option<f64> { Some(0.) }
    fn ReceiveNullableString(&self) -> Option<DOMString> { Some("".to_string()) }
    fn ReceiveNullableByteString(&self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn ReceiveNullableEnum(&self) -> Option<TestEnum> { Some(_empty) }
    fn ReceiveNullableInterface(&self) -> Option<Temporary<Blob>>;
    fn ReceiveNullableAny(&self, _: *mut JSContext) -> Option<JSVal> { Some(NullValue()) }
    fn ReceiveNullableUnion(&self) -> Option<HTMLElementOrLong> { Some(eLong(0)) }
    fn ReceiveNullableUnion2(&self) -> Option<EventOrString> { Some(eString("".to_string())) }

    fn PassBoolean(&self, _: bool) {}
    fn PassByte(&self, _: i8) {}
    fn PassOctet(&self, _: u8) {}
    fn PassShort(&self, _: i16) {}
    fn PassUnsignedShort(&self, _: u16) {}
    fn PassLong(&self, _: i32) {}
    fn PassUnsignedLong(&self, _: u32) {}
    fn PassLongLong(&self, _: i64) {}
    fn PassUnsignedLongLong(&self, _: u64) {}
    fn PassFloat(&self, _: f32) {}
    fn PassDouble(&self, _: f64) {}
    fn PassString(&self, _: DOMString) {}
    fn PassByteString(&self, _: ByteString) {}
    fn PassEnum(&self, _: TestEnum) {}
    fn PassInterface(&self, _: &JSRef<Blob>) {}
    fn PassUnion(&self, _: HTMLElementOrLong) {}
    fn PassUnion2(&self, _: EventOrString) {}
    fn PassUnion3(&self, _: BlobOrString) {}
    fn PassAny(&self, _: *mut JSContext, _: JSVal) {}

    fn PassNullableBoolean(&self, _: Option<bool>) {}
    fn PassNullableByte(&self, _: Option<i8>) {}
    fn PassNullableOctet(&self, _: Option<u8>) {}
    fn PassNullableShort(&self, _: Option<i16>) {}
    fn PassNullableUnsignedShort(&self, _: Option<u16>) {}
    fn PassNullableLong(&self, _: Option<i32>) {}
    fn PassNullableUnsignedLong(&self, _: Option<u32>) {}
    fn PassNullableLongLong(&self, _: Option<i64>) {}
    fn PassNullableUnsignedLongLong(&self, _: Option<u64>) {}
    fn PassNullableFloat(&self, _: Option<f32>) {}
    fn PassNullableDouble(&self, _: Option<f64>) {}
    fn PassNullableString(&self, _: Option<DOMString>) {}
    fn PassNullableByteString(&self, _: Option<ByteString>) {}
    // fn PassNullableEnum(&self, _: Option<TestEnum>) {}
    fn PassNullableInterface(&self, _: Option<JSRef<Blob>>) {}
    fn PassNullableUnion(&self, _: Option<HTMLElementOrLong>) {}
    fn PassNullableUnion2(&self, _: Option<EventOrString>) {}
    fn PassNullableAny(&self, _: *mut JSContext, _: Option<JSVal>) {}

    fn PassOptionalBoolean(&self, _: Option<bool>) {}
    fn PassOptionalByte(&self, _: Option<i8>) {}
    fn PassOptionalOctet(&self, _: Option<u8>) {}
    fn PassOptionalShort(&self, _: Option<i16>) {}
    fn PassOptionalUnsignedShort(&self, _: Option<u16>) {}
    fn PassOptionalLong(&self, _: Option<i32>) {}
    fn PassOptionalUnsignedLong(&self, _: Option<u32>) {}
    fn PassOptionalLongLong(&self, _: Option<i64>) {}
    fn PassOptionalUnsignedLongLong(&self, _: Option<u64>) {}
    fn PassOptionalFloat(&self, _: Option<f32>) {}
    fn PassOptionalDouble(&self, _: Option<f64>) {}
    fn PassOptionalString(&self, _: Option<DOMString>) {}
    fn PassOptionalByteString(&self, _: Option<ByteString>) {}
    fn PassOptionalEnum(&self, _: Option<TestEnum>) {}
    fn PassOptionalInterface(&self, _: Option<JSRef<Blob>>) {}
    fn PassOptionalUnion(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalUnion2(&self, _: Option<EventOrString>) {}
    fn PassOptionalAny(&self, _: *mut JSContext, _: JSVal) {}

    fn PassOptionalNullableBoolean(&self, _: Option<Option<bool>>) {}
    fn PassOptionalNullableByte(&self, _: Option<Option<i8>>) {}
    fn PassOptionalNullableOctet(&self, _: Option<Option<u8>>) {}
    fn PassOptionalNullableShort(&self, _: Option<Option<i16>>) {}
    fn PassOptionalNullableUnsignedShort(&self, _: Option<Option<u16>>) {}
    fn PassOptionalNullableLong(&self, _: Option<Option<i32>>) {}
    fn PassOptionalNullableUnsignedLong(&self, _: Option<Option<u32>>) {}
    fn PassOptionalNullableLongLong(&self, _: Option<Option<i64>>) {}
    fn PassOptionalNullableUnsignedLongLong(&self, _: Option<Option<u64>>) {}
    fn PassOptionalNullableFloat(&self, _: Option<Option<f32>>) {}
    fn PassOptionalNullableDouble(&self, _: Option<Option<f64>>) {}
    fn PassOptionalNullableString(&self, _: Option<Option<DOMString>>) {}
    fn PassOptionalNullableByteString(&self, _: Option<Option<ByteString>>) {}
    // fn PassOptionalNullableEnum(&self, _: Option<Option<TestEnum>>) {}
    fn PassOptionalNullableInterface(&self, _: Option<Option<JSRef<Blob>>>) {}
    fn PassOptionalNullableUnion(&self, _: Option<Option<HTMLElementOrLong>>) {}
    fn PassOptionalNullableUnion2(&self, _: Option<Option<EventOrString>>) {}

    fn PassOptionalBooleanWithDefault(&self, _: bool) {}
    fn PassOptionalByteWithDefault(&self, _: i8) {}
    fn PassOptionalOctetWithDefault(&self, _: u8) {}
    fn PassOptionalShortWithDefault(&self, _: i16) {}
    fn PassOptionalUnsignedShortWithDefault(&self, _: u16) {}
    fn PassOptionalLongWithDefault(&self, _: i32) {}
    fn PassOptionalUnsignedLongWithDefault(&self, _: u32) {}
    fn PassOptionalLongLongWithDefault(&self, _: i64) {}
    fn PassOptionalUnsignedLongLongWithDefault(&self, _: u64) {}
    fn PassOptionalStringWithDefault(&self, _: DOMString) {}
    fn PassOptionalEnumWithDefault(&self, _: TestEnum) {}

    fn PassOptionalNullableBooleanWithDefault(&self, _: Option<bool>) {}
    fn PassOptionalNullableByteWithDefault(&self, _: Option<i8>) {}
    fn PassOptionalNullableOctetWithDefault(&self, _: Option<u8>) {}
    fn PassOptionalNullableShortWithDefault(&self, _: Option<i16>) {}
    fn PassOptionalNullableUnsignedShortWithDefault(&self, _: Option<u16>) {}
    fn PassOptionalNullableLongWithDefault(&self, _: Option<i32>) {}
    fn PassOptionalNullableUnsignedLongWithDefault(&self, _: Option<u32>) {}
    fn PassOptionalNullableLongLongWithDefault(&self, _: Option<i64>) {}
    fn PassOptionalNullableUnsignedLongLongWithDefault(&self, _: Option<u64>) {}
    fn PassOptionalNullableFloatWithDefault(&self, _: Option<f32>) {}
    fn PassOptionalNullableDoubleWithDefault(&self, _: Option<f64>) {}
    fn PassOptionalNullableStringWithDefault(&self, _: Option<DOMString>) {}
    fn PassOptionalNullableByteStringWithDefault(&self, _: Option<ByteString>) {}
    // fn PassOptionalNullableEnumWithDefault(&self, _: Option<TestEnum>) {}
    fn PassOptionalNullableInterfaceWithDefault(&self, _: Option<JSRef<Blob>>) {}
    fn PassOptionalNullableUnionWithDefault(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalNullableUnion2WithDefault(&self, _: Option<EventOrString>) {}
    fn PassOptionalAnyWithDefault(&self, _: *mut JSContext, _: JSVal) {}

    fn PassOptionalNullableBooleanWithNonNullDefault(&self, _: Option<bool>) {}
    fn PassOptionalNullableByteWithNonNullDefault(&self, _: Option<i8>) {}
    fn PassOptionalNullableOctetWithNonNullDefault(&self, _: Option<u8>) {}
    fn PassOptionalNullableShortWithNonNullDefault(&self, _: Option<i16>) {}
    fn PassOptionalNullableUnsignedShortWithNonNullDefault(&self, _: Option<u16>) {}
    fn PassOptionalNullableLongWithNonNullDefault(&self, _: Option<i32>) {}
    fn PassOptionalNullableUnsignedLongWithNonNullDefault(&self, _: Option<u32>) {}
    fn PassOptionalNullableLongLongWithNonNullDefault(&self, _: Option<i64>) {}
    fn PassOptionalNullableUnsignedLongLongWithNonNullDefault(&self, _: Option<u64>) {}
    // fn PassOptionalNullableFloatWithNonNullDefault(&self, _: Option<f32>) {}
    // fn PassOptionalNullableDoubleWithNonNullDefault(&self, _: Option<f64>) {}
    fn PassOptionalNullableStringWithNonNullDefault(&self, _: Option<DOMString>) {}
    // fn PassOptionalNullableEnumWithNonNullDefault(&self, _: Option<TestEnum>) {}

    fn PassVariadicBoolean(&self, _: Vec<bool>) {}
    fn PassVariadicByte(&self, _: Vec<i8>) {}
    fn PassVariadicOctet(&self, _: Vec<u8>) {}
    fn PassVariadicShort(&self, _: Vec<i16>) {}
    fn PassVariadicUnsignedShort(&self, _: Vec<u16>) {}
    fn PassVariadicLong(&self, _: Vec<i32>) {}
    fn PassVariadicUnsignedLong(&self, _: Vec<u32>) {}
    fn PassVariadicLongLong(&self, _: Vec<i64>) {}
    fn PassVariadicUnsignedLongLong(&self, _: Vec<u64>) {}
    fn PassVariadicFloat(&self, _: Vec<f32>) {}
    fn PassVariadicDouble(&self, _: Vec<f64>) {}
    fn PassVariadicString(&self, _: Vec<DOMString>) {}
    fn PassVariadicByteString(&self, _: Vec<ByteString>) {}
    fn PassVariadicEnum(&self, _: Vec<TestEnum>) {}
    // fn PassVariadicInterface(&self, _: Vec<JSRef<Blob>>) {}
    fn PassVariadicUnion(&self, _: Vec<HTMLElementOrLong>) {}
    fn PassVariadicUnion2(&self, _: Vec<EventOrString>) {}
    fn PassVariadicUnion3(&self, _: Vec<BlobOrString>) {}
    fn PassVariadicAny(&self, _: *mut JSContext, _: Vec<JSVal>) {}
}

impl<'a> TestBindingMethods for JSRef<'a, TestBinding> {
    fn InterfaceAttribute(&self) -> Temporary<Blob> {
        let global = self.global.root();
        Blob::new(&global.root_ref())
    }
    fn GetInterfaceAttributeNullable(&self) -> Option<Temporary<Blob>> {
        let global = self.global.root();
        Some(Blob::new(&global.root_ref()))
    }
    fn ReceiveInterface(&self) -> Temporary<Blob> {
        let global = self.global.root();
        Blob::new(&global.root_ref())
    }
    fn ReceiveNullableInterface(&self) -> Option<Temporary<Blob>> {
        let global = self.global.root();
        Some(Blob::new(&global.root_ref()))
    }
}

impl TestBinding {
    pub fn BooleanAttributeStatic() -> bool { false }
    pub fn SetBooleanAttributeStatic(_: bool) {}
    pub fn ReceiveVoidStatic() {}
}

impl Reflectable for TestBinding {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector
    }
}
