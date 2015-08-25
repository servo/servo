/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::TestBindingBinding::TestBindingMethods;
use dom::bindings::codegen::Bindings::TestBindingBinding::TestEnum;
use dom::bindings::codegen::Bindings::TestBindingBinding::TestEnum::_empty;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::codegen::UnionTypes::EventOrString;
use dom::bindings::codegen::UnionTypes::EventOrString::eString;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong::eLong;
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::str::{ByteString, USVString};
use dom::bindings::utils::Reflector;
use dom::blob::Blob;
use util::str::DOMString;

use js::jsapi::{JSContext, JSObject, HandleValue};
use js::jsval::{JSVal, NullValue};

use std::borrow::ToOwned;
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct TestBinding {
    reflector_: Reflector,
    global: GlobalField,
}

impl<'a> TestBindingMethods for &'a TestBinding {
    fn BooleanAttribute(self) -> bool { false }
    fn SetBooleanAttribute(self, _: bool) {}
    fn ByteAttribute(self) -> i8 { 0 }
    fn SetByteAttribute(self, _: i8) {}
    fn OctetAttribute(self) -> u8 { 0 }
    fn SetOctetAttribute(self, _: u8) {}
    fn ShortAttribute(self) -> i16 { 0 }
    fn SetShortAttribute(self, _: i16) {}
    fn UnsignedShortAttribute(self) -> u16 { 0 }
    fn SetUnsignedShortAttribute(self, _: u16) {}
    fn LongAttribute(self) -> i32 { 0 }
    fn SetLongAttribute(self, _: i32) {}
    fn UnsignedLongAttribute(self) -> u32 { 0 }
    fn SetUnsignedLongAttribute(self, _: u32) {}
    fn LongLongAttribute(self) -> i64 { 0 }
    fn SetLongLongAttribute(self, _: i64) {}
    fn UnsignedLongLongAttribute(self) -> u64 { 0 }
    fn SetUnsignedLongLongAttribute(self, _: u64) {}
    fn UnrestrictedFloatAttribute(self) -> f32 { 0. }
    fn SetUnrestrictedFloatAttribute(self, _: f32) {}
    fn FloatAttribute(self) -> Finite<f32> { Finite::wrap(0.) }
    fn SetFloatAttribute(self, _: Finite<f32>) {}
    fn UnrestrictedDoubleAttribute(self) -> f64 { 0. }
    fn SetUnrestrictedDoubleAttribute(self, _: f64) {}
    fn DoubleAttribute(self) -> Finite<f64> { Finite::wrap(0.) }
    fn SetDoubleAttribute(self, _: Finite<f64>) {}
    fn StringAttribute(self) -> DOMString { "".to_owned() }
    fn SetStringAttribute(self, _: DOMString) {}
    fn UsvstringAttribute(self) -> USVString { USVString("".to_owned()) }
    fn SetUsvstringAttribute(self, _: USVString) {}
    fn ByteStringAttribute(self) -> ByteString { ByteString::new(vec!()) }
    fn SetByteStringAttribute(self, _: ByteString) {}
    fn EnumAttribute(self) -> TestEnum { _empty }
    fn SetEnumAttribute(self, _: TestEnum) {}
    fn InterfaceAttribute(self) -> Root<Blob> {
        let global = self.global.root();
        Blob::new(global.r(), None, "")
    }
    fn SetInterfaceAttribute(self, _: &Blob) {}
    fn UnionAttribute(self) -> HTMLElementOrLong { eLong(0) }
    fn SetUnionAttribute(self, _: HTMLElementOrLong) {}
    fn Union2Attribute(self) -> EventOrString { eString("".to_owned()) }
    fn SetUnion2Attribute(self, _: EventOrString) {}
    fn ArrayAttribute(self, _: *mut JSContext) -> *mut JSObject { NullValue().to_object_or_null() }
    fn AnyAttribute(self, _: *mut JSContext) -> JSVal { NullValue() }
    fn SetAnyAttribute(self, _: *mut JSContext, _: HandleValue) {}
    fn ObjectAttribute(self, _: *mut JSContext) -> *mut JSObject { panic!() }
    fn SetObjectAttribute(self, _: *mut JSContext, _: *mut JSObject) {}

    fn GetBooleanAttributeNullable(self) -> Option<bool> { Some(false) }
    fn SetBooleanAttributeNullable(self, _: Option<bool>) {}
    fn GetByteAttributeNullable(self) -> Option<i8> { Some(0) }
    fn SetByteAttributeNullable(self, _: Option<i8>) {}
    fn GetOctetAttributeNullable(self) -> Option<u8> { Some(0) }
    fn SetOctetAttributeNullable(self, _: Option<u8>) {}
    fn GetShortAttributeNullable(self) -> Option<i16> { Some(0) }
    fn SetShortAttributeNullable(self, _: Option<i16>) {}
    fn GetUnsignedShortAttributeNullable(self) -> Option<u16> { Some(0) }
    fn SetUnsignedShortAttributeNullable(self, _: Option<u16>) {}
    fn GetLongAttributeNullable(self) -> Option<i32> { Some(0) }
    fn SetLongAttributeNullable(self, _: Option<i32>) {}
    fn GetUnsignedLongAttributeNullable(self) -> Option<u32> { Some(0) }
    fn SetUnsignedLongAttributeNullable(self, _: Option<u32>) {}
    fn GetLongLongAttributeNullable(self) -> Option<i64> { Some(0) }
    fn SetLongLongAttributeNullable(self, _: Option<i64>) {}
    fn GetUnsignedLongLongAttributeNullable(self) -> Option<u64> { Some(0) }
    fn SetUnsignedLongLongAttributeNullable(self, _: Option<u64>) {}
    fn GetUnrestrictedFloatAttributeNullable(self) -> Option<f32> { Some(0.) }
    fn SetUnrestrictedFloatAttributeNullable(self, _: Option<f32>) {}
    fn GetFloatAttributeNullable(self) -> Option<Finite<f32>> { Some(Finite::wrap(0.)) }
    fn SetFloatAttributeNullable(self, _: Option<Finite<f32>>) {}
    fn GetUnrestrictedDoubleAttributeNullable(self) -> Option<f64> { Some(0.) }
    fn SetUnrestrictedDoubleAttributeNullable(self, _: Option<f64>) {}
    fn GetDoubleAttributeNullable(self) -> Option<Finite<f64>> { Some(Finite::wrap(0.)) }
    fn SetDoubleAttributeNullable(self, _: Option<Finite<f64>>) {}
    fn GetByteStringAttributeNullable(self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn SetByteStringAttributeNullable(self, _: Option<ByteString>) {}
    fn GetStringAttributeNullable(self) -> Option<DOMString> { Some("".to_owned()) }
    fn SetStringAttributeNullable(self, _: Option<DOMString>) {}
    fn GetUsvstringAttributeNullable(self) -> Option<USVString> { Some(USVString("".to_owned())) }
    fn SetUsvstringAttributeNullable(self, _: Option<USVString>) {}
    fn SetBinaryRenamedAttribute(self, _: DOMString) {}
    fn ForwardedAttribute(self) -> Root<TestBinding> { Root::from_ref(self) }
    fn BinaryRenamedAttribute(self) -> DOMString { "".to_owned() }
    fn GetEnumAttributeNullable(self) -> Option<TestEnum> { Some(_empty) }
    fn GetInterfaceAttributeNullable(self) -> Option<Root<Blob>> {
        let global = self.global.root();
        Some(Blob::new(global.r(), None, ""))
    }
    fn SetInterfaceAttributeNullable(self, _: Option<&Blob>) {}
    fn GetObjectAttributeNullable(self, _: *mut JSContext) -> *mut JSObject { ptr::null_mut() }
    fn SetObjectAttributeNullable(self, _: *mut JSContext, _: *mut JSObject) {}
    fn GetUnionAttributeNullable(self) -> Option<HTMLElementOrLong> { Some(eLong(0)) }
    fn SetUnionAttributeNullable(self, _: Option<HTMLElementOrLong>) {}
    fn GetUnion2AttributeNullable(self) -> Option<EventOrString> { Some(eString("".to_owned())) }
    fn SetUnion2AttributeNullable(self, _: Option<EventOrString>) {}
    fn BinaryRenamedMethod(self) -> () {}
    fn ReceiveVoid(self) -> () {}
    fn ReceiveBoolean(self) -> bool { false }
    fn ReceiveByte(self) -> i8 { 0 }
    fn ReceiveOctet(self) -> u8 { 0 }
    fn ReceiveShort(self) -> i16 { 0 }
    fn ReceiveUnsignedShort(self) -> u16 { 0 }
    fn ReceiveLong(self) -> i32 { 0 }
    fn ReceiveUnsignedLong(self) -> u32 { 0 }
    fn ReceiveLongLong(self) -> i64 { 0 }
    fn ReceiveUnsignedLongLong(self) -> u64 { 0 }
    fn ReceiveUnrestrictedFloat(self) -> f32 { 0. }
    fn ReceiveFloat(self) -> Finite<f32> { Finite::wrap(0.) }
    fn ReceiveUnrestrictedDouble(self) -> f64 { 0. }
    fn ReceiveDouble(self) -> Finite<f64> { Finite::wrap(0.) }
    fn ReceiveString(self) -> DOMString { "".to_owned() }
    fn ReceiveUsvstring(self) -> USVString { USVString("".to_owned()) }
    fn ReceiveByteString(self) -> ByteString { ByteString::new(vec!()) }
    fn ReceiveEnum(self) -> TestEnum { _empty }
    fn ReceiveInterface(self) -> Root<Blob> {
        let global = self.global.root();
        Blob::new(global.r(), None, "")
    }
    fn ReceiveAny(self, _: *mut JSContext) -> JSVal { NullValue() }
    fn ReceiveObject(self, _: *mut JSContext) -> *mut JSObject { panic!() }
    fn ReceiveUnion(self) -> HTMLElementOrLong { eLong(0) }
    fn ReceiveUnion2(self) -> EventOrString { eString("".to_owned()) }

    fn ReceiveNullableBoolean(self) -> Option<bool> { Some(false) }
    fn ReceiveNullableByte(self) -> Option<i8> { Some(0) }
    fn ReceiveNullableOctet(self) -> Option<u8> { Some(0) }
    fn ReceiveNullableShort(self) -> Option<i16> { Some(0) }
    fn ReceiveNullableUnsignedShort(self) -> Option<u16> { Some(0) }
    fn ReceiveNullableLong(self) -> Option<i32> { Some(0) }
    fn ReceiveNullableUnsignedLong(self) -> Option<u32> { Some(0) }
    fn ReceiveNullableLongLong(self) -> Option<i64> { Some(0) }
    fn ReceiveNullableUnsignedLongLong(self) -> Option<u64> { Some(0) }
    fn ReceiveNullableUnrestrictedFloat(self) -> Option<f32> { Some(0.) }
    fn ReceiveNullableFloat(self) -> Option<Finite<f32>> { Some(Finite::wrap(0.)) }
    fn ReceiveNullableUnrestrictedDouble(self) -> Option<f64> { Some(0.) }
    fn ReceiveNullableDouble(self) -> Option<Finite<f64>> { Some(Finite::wrap(0.)) }
    fn ReceiveNullableString(self) -> Option<DOMString> { Some("".to_owned()) }
    fn ReceiveNullableUsvstring(self) -> Option<USVString> { Some(USVString("".to_owned())) }
    fn ReceiveNullableByteString(self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn ReceiveNullableEnum(self) -> Option<TestEnum> { Some(_empty) }
    fn ReceiveNullableInterface(self) -> Option<Root<Blob>> {
        let global = self.global.root();
        Some(Blob::new(global.r(), None, ""))
    }
    fn ReceiveNullableObject(self, _: *mut JSContext) -> *mut JSObject { ptr::null_mut() }
    fn ReceiveNullableUnion(self) -> Option<HTMLElementOrLong> { Some(eLong(0)) }
    fn ReceiveNullableUnion2(self) -> Option<EventOrString> { Some(eString("".to_owned())) }

    fn PassBoolean(self, _: bool) {}
    fn PassByte(self, _: i8) {}
    fn PassOctet(self, _: u8) {}
    fn PassShort(self, _: i16) {}
    fn PassUnsignedShort(self, _: u16) {}
    fn PassLong(self, _: i32) {}
    fn PassUnsignedLong(self, _: u32) {}
    fn PassLongLong(self, _: i64) {}
    fn PassUnsignedLongLong(self, _: u64) {}
    fn PassUnrestrictedFloat(self, _: f32) {}
    fn PassFloat(self, _: Finite<f32>) {}
    fn PassUnrestrictedDouble(self, _: f64) {}
    fn PassDouble(self, _: Finite<f64>) {}
    fn PassString(self, _: DOMString) {}
    fn PassUsvstring(self, _: USVString) {}
    fn PassByteString(self, _: ByteString) {}
    fn PassEnum(self, _: TestEnum) {}
    fn PassInterface(self, _: &Blob) {}
    fn PassUnion(self, _: HTMLElementOrLong) {}
    fn PassUnion2(self, _: EventOrString) {}
    fn PassUnion3(self, _: BlobOrString) {}
    fn PassAny(self, _: *mut JSContext, _: HandleValue) {}
    fn PassObject(self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassCallbackFunction(self, _: Rc<Function>) {}
    fn PassCallbackInterface(self, _: Rc<EventListener>) {}

    fn PassNullableBoolean(self, _: Option<bool>) {}
    fn PassNullableByte(self, _: Option<i8>) {}
    fn PassNullableOctet(self, _: Option<u8>) {}
    fn PassNullableShort(self, _: Option<i16>) {}
    fn PassNullableUnsignedShort(self, _: Option<u16>) {}
    fn PassNullableLong(self, _: Option<i32>) {}
    fn PassNullableUnsignedLong(self, _: Option<u32>) {}
    fn PassNullableLongLong(self, _: Option<i64>) {}
    fn PassNullableUnsignedLongLong(self, _: Option<u64>) {}
    fn PassNullableUnrestrictedFloat(self, _: Option<f32>) {}
    fn PassNullableFloat(self, _: Option<Finite<f32>>) {}
    fn PassNullableUnrestrictedDouble(self, _: Option<f64>) {}
    fn PassNullableDouble(self, _: Option<Finite<f64>>) {}
    fn PassNullableString(self, _: Option<DOMString>) {}
    fn PassNullableUsvstring(self, _: Option<USVString>) {}
    fn PassNullableByteString(self, _: Option<ByteString>) {}
    // fn PassNullableEnum(self, _: Option<TestEnum>) {}
    fn PassNullableInterface(self, _: Option<&Blob>) {}
    fn PassNullableObject(self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassNullableUnion(self, _: Option<HTMLElementOrLong>) {}
    fn PassNullableUnion2(self, _: Option<EventOrString>) {}
    fn PassNullableCallbackFunction(self, _: Option<Rc<Function>>) {}
    fn PassNullableCallbackInterface(self, _: Option<Rc<EventListener>>) {}

    fn PassOptionalBoolean(self, _: Option<bool>) {}
    fn PassOptionalByte(self, _: Option<i8>) {}
    fn PassOptionalOctet(self, _: Option<u8>) {}
    fn PassOptionalShort(self, _: Option<i16>) {}
    fn PassOptionalUnsignedShort(self, _: Option<u16>) {}
    fn PassOptionalLong(self, _: Option<i32>) {}
    fn PassOptionalUnsignedLong(self, _: Option<u32>) {}
    fn PassOptionalLongLong(self, _: Option<i64>) {}
    fn PassOptionalUnsignedLongLong(self, _: Option<u64>) {}
    fn PassOptionalUnrestrictedFloat(self, _: Option<f32>) {}
    fn PassOptionalFloat(self, _: Option<Finite<f32>>) {}
    fn PassOptionalUnrestrictedDouble(self, _: Option<f64>) {}
    fn PassOptionalDouble(self, _: Option<Finite<f64>>) {}
    fn PassOptionalString(self, _: Option<DOMString>) {}
    fn PassOptionalUsvstring(self, _: Option<USVString>) {}
    fn PassOptionalByteString(self, _: Option<ByteString>) {}
    fn PassOptionalEnum(self, _: Option<TestEnum>) {}
    fn PassOptionalInterface(self, _: Option<&Blob>) {}
    fn PassOptionalUnion(self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalUnion2(self, _: Option<EventOrString>) {}
    fn PassOptionalAny(self, _: *mut JSContext, _: HandleValue) {}
    fn PassOptionalObject(self, _: *mut JSContext, _: Option<*mut JSObject>) {}
    fn PassOptionalCallbackFunction(self, _: Option<Rc<Function>>) {}
    fn PassOptionalCallbackInterface(self, _: Option<Rc<EventListener>>) {}

    fn PassOptionalNullableBoolean(self, _: Option<Option<bool>>) {}
    fn PassOptionalNullableByte(self, _: Option<Option<i8>>) {}
    fn PassOptionalNullableOctet(self, _: Option<Option<u8>>) {}
    fn PassOptionalNullableShort(self, _: Option<Option<i16>>) {}
    fn PassOptionalNullableUnsignedShort(self, _: Option<Option<u16>>) {}
    fn PassOptionalNullableLong(self, _: Option<Option<i32>>) {}
    fn PassOptionalNullableUnsignedLong(self, _: Option<Option<u32>>) {}
    fn PassOptionalNullableLongLong(self, _: Option<Option<i64>>) {}
    fn PassOptionalNullableUnsignedLongLong(self, _: Option<Option<u64>>) {}
    fn PassOptionalNullableUnrestrictedFloat(self, _: Option<Option<f32>>) {}
    fn PassOptionalNullableFloat(self, _: Option<Option<Finite<f32>>>) {}
    fn PassOptionalNullableUnrestrictedDouble(self, _: Option<Option<f64>>) {}
    fn PassOptionalNullableDouble(self, _: Option<Option<Finite<f64>>>) {}
    fn PassOptionalNullableString(self, _: Option<Option<DOMString>>) {}
    fn PassOptionalNullableUsvstring(self, _: Option<Option<USVString>>) {}
    fn PassOptionalNullableByteString(self, _: Option<Option<ByteString>>) {}
    // fn PassOptionalNullableEnum(self, _: Option<Option<TestEnum>>) {}
    fn PassOptionalNullableInterface(self, _: Option<Option<&Blob>>) {}
    fn PassOptionalNullableObject(self, _: *mut JSContext, _: Option<*mut JSObject>) {}
    fn PassOptionalNullableUnion(self, _: Option<Option<HTMLElementOrLong>>) {}
    fn PassOptionalNullableUnion2(self, _: Option<Option<EventOrString>>) {}
    fn PassOptionalNullableCallbackFunction(self, _: Option<Option<Rc<Function>>>) {}
    fn PassOptionalNullableCallbackInterface(self, _: Option<Option<Rc<EventListener>>>) {}

    fn PassOptionalBooleanWithDefault(self, _: bool) {}
    fn PassOptionalByteWithDefault(self, _: i8) {}
    fn PassOptionalOctetWithDefault(self, _: u8) {}
    fn PassOptionalShortWithDefault(self, _: i16) {}
    fn PassOptionalUnsignedShortWithDefault(self, _: u16) {}
    fn PassOptionalLongWithDefault(self, _: i32) {}
    fn PassOptionalUnsignedLongWithDefault(self, _: u32) {}
    fn PassOptionalLongLongWithDefault(self, _: i64) {}
    fn PassOptionalUnsignedLongLongWithDefault(self, _: u64) {}
    fn PassOptionalStringWithDefault(self, _: DOMString) {}
    fn PassOptionalUsvstringWithDefault(self, _: USVString) {}
    fn PassOptionalEnumWithDefault(self, _: TestEnum) {}

    fn PassOptionalNullableBooleanWithDefault(self, _: Option<bool>) {}
    fn PassOptionalNullableByteWithDefault(self, _: Option<i8>) {}
    fn PassOptionalNullableOctetWithDefault(self, _: Option<u8>) {}
    fn PassOptionalNullableShortWithDefault(self, _: Option<i16>) {}
    fn PassOptionalNullableUnsignedShortWithDefault(self, _: Option<u16>) {}
    fn PassOptionalNullableLongWithDefault(self, _: Option<i32>) {}
    fn PassOptionalNullableUnsignedLongWithDefault(self, _: Option<u32>) {}
    fn PassOptionalNullableLongLongWithDefault(self, _: Option<i64>) {}
    fn PassOptionalNullableUnsignedLongLongWithDefault(self, _: Option<u64>) {}
    // fn PassOptionalNullableUnrestrictedFloatWithDefault(self, _: Option<f32>) {}
    // fn PassOptionalNullableFloatWithDefault(self, _: Option<Finite<f32>>) {}
    // fn PassOptionalNullableUnrestrictedDoubleWithDefault(self, _: Option<f64>) {}
    // fn PassOptionalNullableDoubleWithDefault(self, _: Option<Finite<f64>>) {}
    fn PassOptionalNullableStringWithDefault(self, _: Option<DOMString>) {}
    fn PassOptionalNullableUsvstringWithDefault(self, _: Option<USVString>) {}
    fn PassOptionalNullableByteStringWithDefault(self, _: Option<ByteString>) {}
    // fn PassOptionalNullableEnumWithDefault(self, _: Option<TestEnum>) {}
    fn PassOptionalNullableInterfaceWithDefault(self, _: Option<&Blob>) {}
    fn PassOptionalNullableObjectWithDefault(self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassOptionalNullableUnionWithDefault(self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalNullableUnion2WithDefault(self, _: Option<EventOrString>) {}
    // fn PassOptionalNullableCallbackFunctionWithDefault(self, _: Option<Function>) {}
    fn PassOptionalNullableCallbackInterfaceWithDefault(self, _: Option<Rc<EventListener>>) {}
    fn PassOptionalAnyWithDefault(self, _: *mut JSContext, _: HandleValue) {}

    fn PassOptionalNullableBooleanWithNonNullDefault(self, _: Option<bool>) {}
    fn PassOptionalNullableByteWithNonNullDefault(self, _: Option<i8>) {}
    fn PassOptionalNullableOctetWithNonNullDefault(self, _: Option<u8>) {}
    fn PassOptionalNullableShortWithNonNullDefault(self, _: Option<i16>) {}
    fn PassOptionalNullableUnsignedShortWithNonNullDefault(self, _: Option<u16>) {}
    fn PassOptionalNullableLongWithNonNullDefault(self, _: Option<i32>) {}
    fn PassOptionalNullableUnsignedLongWithNonNullDefault(self, _: Option<u32>) {}
    fn PassOptionalNullableLongLongWithNonNullDefault(self, _: Option<i64>) {}
    fn PassOptionalNullableUnsignedLongLongWithNonNullDefault(self, _: Option<u64>) {}
    // fn PassOptionalNullableUnrestrictedFloatWithNonNullDefault(self, _: Option<f32>) {}
    // fn PassOptionalNullableFloatWithNonNullDefault(self, _: Option<Finite<f32>>) {}
    // fn PassOptionalNullableUnrestrictedDoubleWithNonNullDefault(self, _: Option<f64>) {}
    // fn PassOptionalNullableDoubleWithNonNullDefault(self, _: Option<Finite<f64>>) {}
    fn PassOptionalNullableStringWithNonNullDefault(self, _: Option<DOMString>) {}
    fn PassOptionalNullableUsvstringWithNonNullDefault(self, _: Option<USVString>) {}
    // fn PassOptionalNullableEnumWithNonNullDefault(self, _: Option<TestEnum>) {}

    fn PassVariadicBoolean(self, _: Vec<bool>) {}
    fn PassVariadicByte(self, _: Vec<i8>) {}
    fn PassVariadicOctet(self, _: Vec<u8>) {}
    fn PassVariadicShort(self, _: Vec<i16>) {}
    fn PassVariadicUnsignedShort(self, _: Vec<u16>) {}
    fn PassVariadicLong(self, _: Vec<i32>) {}
    fn PassVariadicUnsignedLong(self, _: Vec<u32>) {}
    fn PassVariadicLongLong(self, _: Vec<i64>) {}
    fn PassVariadicUnsignedLongLong(self, _: Vec<u64>) {}
    fn PassVariadicUnrestrictedFloat(self, _: Vec<f32>) {}
    fn PassVariadicFloat(self, _: Vec<Finite<f32>>) {}
    fn PassVariadicUnrestrictedDouble(self, _: Vec<f64>) {}
    fn PassVariadicDouble(self, _: Vec<Finite<f64>>) {}
    fn PassVariadicString(self, _: Vec<DOMString>) {}
    fn PassVariadicUsvstring(self, _: Vec<USVString>) {}
    fn PassVariadicByteString(self, _: Vec<ByteString>) {}
    fn PassVariadicEnum(self, _: Vec<TestEnum>) {}
    // fn PassVariadicInterface(self, _: Vec<&Blob>) {}
    fn PassVariadicUnion(self, _: Vec<HTMLElementOrLong>) {}
    fn PassVariadicUnion2(self, _: Vec<EventOrString>) {}
    fn PassVariadicUnion3(self, _: Vec<BlobOrString>) {}
    fn PassVariadicAny(self, _: *mut JSContext, _: Vec<HandleValue>) {}
    fn PassVariadicObject(self, _: *mut JSContext, _: Vec<*mut JSObject>) {}
}

impl TestBinding {
    pub fn BooleanAttributeStatic(_: GlobalRef) -> bool { false }
    pub fn SetBooleanAttributeStatic(_: GlobalRef, _: bool) {}
    pub fn ReceiveVoidStatic(_: GlobalRef) {}
}
