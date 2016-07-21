/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::TestBindingBinding;
use dom::bindings::codegen::Bindings::TestBindingBinding::{TestBindingMethods, TestDictionary};
use dom::bindings::codegen::Bindings::TestBindingBinding::{TestDictionaryDefaults, TestEnum};
use dom::bindings::codegen::UnionTypes::{BlobOrBoolean, BlobOrBlobSequence, LongOrLongSequenceSequence};
use dom::bindings::codegen::UnionTypes::{BlobOrString, BlobOrUnsignedLong, EventOrString};
use dom::bindings::codegen::UnionTypes::{ByteStringOrLong, ByteStringSequenceOrLongOrString, ByteStringSequenceOrLong};
use dom::bindings::codegen::UnionTypes::{EventOrUSVString, HTMLElementOrLong};
use dom::bindings::codegen::UnionTypes::{HTMLElementOrUnsignedLongOrStringOrBoolean, LongSequenceOrBoolean};
use dom::bindings::codegen::UnionTypes::{StringOrLongSequence, StringOrStringSequence, StringSequenceOrUnsignedLong};
use dom::bindings::codegen::UnionTypes::{StringOrUnsignedLong, StringOrBoolean, UnsignedLongOrBoolean};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString, USVString};
use dom::bindings::weakref::MutableWeakRef;
use dom::blob::{Blob, BlobImpl};
use dom::url::URL;
use js::jsapi::{HandleObject, HandleValue, JSContext, JSObject};
use js::jsval::{JSVal, NullValue};
use std::borrow::ToOwned;
use std::ptr;
use std::rc::Rc;
use util::prefs::PREFS;

#[dom_struct]
pub struct TestBinding {
    reflector_: Reflector,
    url: MutableWeakRef<URL>,
}

impl TestBinding {
    fn new_inherited() -> TestBinding {
        TestBinding {
            reflector_: Reflector::new(),
            url: MutableWeakRef::new(None),
        }
    }

    pub fn new(global: GlobalRef) -> Root<TestBinding> {
        reflect_dom_object(box TestBinding::new_inherited(),
                           global, TestBindingBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<TestBinding>> {
        Ok(TestBinding::new(global))
    }

    #[allow(unused_variables)]
    pub fn Constructor_(global: GlobalRef, nums: Vec<f64>) -> Fallible<Root<TestBinding>> {
        Ok(TestBinding::new(global))
    }

    #[allow(unused_variables)]
    pub fn Constructor__(global: GlobalRef, num: f64) -> Fallible<Root<TestBinding>> {
        Ok(TestBinding::new(global))
    }
}

impl TestBindingMethods for TestBinding {
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
    fn UnrestrictedFloatAttribute(&self) -> f32 { 0. }
    fn SetUnrestrictedFloatAttribute(&self, _: f32) {}
    fn FloatAttribute(&self) -> Finite<f32> { Finite::wrap(0.) }
    fn SetFloatAttribute(&self, _: Finite<f32>) {}
    fn UnrestrictedDoubleAttribute(&self) -> f64 { 0. }
    fn SetUnrestrictedDoubleAttribute(&self, _: f64) {}
    fn DoubleAttribute(&self) -> Finite<f64> { Finite::wrap(0.) }
    fn SetDoubleAttribute(&self, _: Finite<f64>) {}
    fn StringAttribute(&self) -> DOMString { DOMString::new() }
    fn SetStringAttribute(&self, _: DOMString) {}
    fn UsvstringAttribute(&self) -> USVString { USVString("".to_owned()) }
    fn SetUsvstringAttribute(&self, _: USVString) {}
    fn ByteStringAttribute(&self) -> ByteString { ByteString::new(vec!()) }
    fn SetByteStringAttribute(&self, _: ByteString) {}
    fn EnumAttribute(&self) -> TestEnum { TestEnum::_empty }
    fn SetEnumAttribute(&self, _: TestEnum) {}
    fn InterfaceAttribute(&self) -> Root<Blob> {
        Blob::new(self.global().r(), BlobImpl::new_from_bytes(vec![]), "".to_owned())
    }
    fn SetInterfaceAttribute(&self, _: &Blob) {}
    fn UnionAttribute(&self) -> HTMLElementOrLong { HTMLElementOrLong::Long(0) }
    fn SetUnionAttribute(&self, _: HTMLElementOrLong) {}
    fn Union2Attribute(&self) -> EventOrString { EventOrString::String(DOMString::new()) }
    fn SetUnion2Attribute(&self, _: EventOrString) {}
    fn Union3Attribute(&self) -> EventOrUSVString {
        EventOrUSVString::USVString(USVString("".to_owned()))
    }
    fn SetUnion3Attribute(&self, _: EventOrUSVString) {}
    fn Union4Attribute(&self) -> StringOrUnsignedLong {
        StringOrUnsignedLong::UnsignedLong(0u32)
    }
    fn SetUnion4Attribute(&self, _: StringOrUnsignedLong) {}
    fn Union5Attribute(&self) -> StringOrBoolean {
        StringOrBoolean::Boolean(true)
    }
    fn SetUnion5Attribute(&self, _: StringOrBoolean) {}
    fn Union6Attribute(&self) -> UnsignedLongOrBoolean {
        UnsignedLongOrBoolean::Boolean(true)
    }
    fn SetUnion6Attribute(&self, _: UnsignedLongOrBoolean) {}
    fn Union7Attribute(&self) -> BlobOrBoolean {
        BlobOrBoolean::Boolean(true)
    }
    fn SetUnion7Attribute(&self, _: BlobOrBoolean) {}
    fn Union8Attribute(&self) -> BlobOrUnsignedLong {
        BlobOrUnsignedLong::UnsignedLong(0u32)
    }
    fn SetUnion8Attribute(&self, _: BlobOrUnsignedLong) {}
    fn Union9Attribute(&self) -> ByteStringOrLong {
        ByteStringOrLong::ByteString(ByteString::new(vec!()))
    }
    fn SetUnion9Attribute(&self, _: ByteStringOrLong) {}
    fn ArrayAttribute(&self, _: *mut JSContext) -> *mut JSObject { NullValue().to_object_or_null() }
    fn AnyAttribute(&self, _: *mut JSContext) -> JSVal { NullValue() }
    fn SetAnyAttribute(&self, _: *mut JSContext, _: HandleValue) {}
    fn ObjectAttribute(&self, _: *mut JSContext) -> *mut JSObject { panic!() }
    fn SetObjectAttribute(&self, _: *mut JSContext, _: *mut JSObject) {}

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
    fn GetUnrestrictedFloatAttributeNullable(&self) -> Option<f32> { Some(0.) }
    fn SetUnrestrictedFloatAttributeNullable(&self, _: Option<f32>) {}
    fn GetFloatAttributeNullable(&self) -> Option<Finite<f32>> { Some(Finite::wrap(0.)) }
    fn SetFloatAttributeNullable(&self, _: Option<Finite<f32>>) {}
    fn GetUnrestrictedDoubleAttributeNullable(&self) -> Option<f64> { Some(0.) }
    fn SetUnrestrictedDoubleAttributeNullable(&self, _: Option<f64>) {}
    fn GetDoubleAttributeNullable(&self) -> Option<Finite<f64>> { Some(Finite::wrap(0.)) }
    fn SetDoubleAttributeNullable(&self, _: Option<Finite<f64>>) {}
    fn GetByteStringAttributeNullable(&self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn SetByteStringAttributeNullable(&self, _: Option<ByteString>) {}
    fn GetStringAttributeNullable(&self) -> Option<DOMString> { Some(DOMString::new()) }
    fn SetStringAttributeNullable(&self, _: Option<DOMString>) {}
    fn GetUsvstringAttributeNullable(&self) -> Option<USVString> { Some(USVString("".to_owned())) }
    fn SetUsvstringAttributeNullable(&self, _: Option<USVString>) {}
    fn SetBinaryRenamedAttribute(&self, _: DOMString) {}
    fn ForwardedAttribute(&self) -> Root<TestBinding> { Root::from_ref(self) }
    fn BinaryRenamedAttribute(&self) -> DOMString { DOMString::new() }
    fn SetBinaryRenamedAttribute2(&self, _: DOMString) {}
    fn BinaryRenamedAttribute2(&self) -> DOMString { DOMString::new() }
    fn Attr_to_automatically_rename(&self) -> DOMString { DOMString::new() }
    fn SetAttr_to_automatically_rename(&self, _: DOMString) {}
    fn GetEnumAttributeNullable(&self) -> Option<TestEnum> { Some(TestEnum::_empty) }
    fn GetInterfaceAttributeNullable(&self) -> Option<Root<Blob>> {
        Some(Blob::new(self.global().r(), BlobImpl::new_from_bytes(vec![]), "".to_owned()))
    }
    fn SetInterfaceAttributeNullable(&self, _: Option<&Blob>) {}
    fn GetInterfaceAttributeWeak(&self) -> Option<Root<URL>> {
        self.url.root()
    }
    fn SetInterfaceAttributeWeak(&self, url: Option<&URL>) {
        self.url.set(url);
    }
    fn GetObjectAttributeNullable(&self, _: *mut JSContext) -> *mut JSObject { ptr::null_mut() }
    fn SetObjectAttributeNullable(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn GetUnionAttributeNullable(&self) -> Option<HTMLElementOrLong> {
        Some(HTMLElementOrLong::Long(0))
    }
    fn SetUnionAttributeNullable(&self, _: Option<HTMLElementOrLong>) {}
    fn GetUnion2AttributeNullable(&self) -> Option<EventOrString> {
        Some(EventOrString::String(DOMString::new()))
    }
    fn SetUnion2AttributeNullable(&self, _: Option<EventOrString>) {}
    fn GetUnion3AttributeNullable(&self) -> Option<BlobOrBoolean> {
        Some(BlobOrBoolean::Boolean(true))
    }
    fn SetUnion3AttributeNullable(&self, _: Option<BlobOrBoolean>) {}
    fn GetUnion4AttributeNullable(&self) -> Option<UnsignedLongOrBoolean> {
        Some(UnsignedLongOrBoolean::Boolean(true))
    }
    fn SetUnion4AttributeNullable(&self, _: Option<UnsignedLongOrBoolean>) {}
    fn GetUnion5AttributeNullable(&self) -> Option<StringOrBoolean> {
        Some(StringOrBoolean::Boolean(true))
    }
    fn SetUnion5AttributeNullable(&self, _: Option<StringOrBoolean>) {}
    fn GetUnion6AttributeNullable(&self) -> Option<ByteStringOrLong> {
        Some(ByteStringOrLong::ByteString(ByteString::new(vec!())))
    }
    fn SetUnion6AttributeNullable(&self, _: Option<ByteStringOrLong>) {}
    fn BinaryRenamedMethod(&self) -> () {}
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
    fn ReceiveUnrestrictedFloat(&self) -> f32 { 0. }
    fn ReceiveFloat(&self) -> Finite<f32> { Finite::wrap(0.) }
    fn ReceiveUnrestrictedDouble(&self) -> f64 { 0. }
    fn ReceiveDouble(&self) -> Finite<f64> { Finite::wrap(0.) }
    fn ReceiveString(&self) -> DOMString { DOMString::new() }
    fn ReceiveUsvstring(&self) -> USVString { USVString("".to_owned()) }
    fn ReceiveByteString(&self) -> ByteString { ByteString::new(vec!()) }
    fn ReceiveEnum(&self) -> TestEnum { TestEnum::_empty }
    fn ReceiveInterface(&self) -> Root<Blob> {
        Blob::new(self.global().r(), BlobImpl::new_from_bytes(vec![]), "".to_owned())
    }
    fn ReceiveAny(&self, _: *mut JSContext) -> JSVal { NullValue() }
    fn ReceiveObject(&self, _: *mut JSContext) -> *mut JSObject { panic!() }
    fn ReceiveUnion(&self) -> HTMLElementOrLong { HTMLElementOrLong::Long(0) }
    fn ReceiveUnion2(&self) -> EventOrString { EventOrString::String(DOMString::new()) }
    fn ReceiveUnion3(&self) -> StringOrLongSequence { StringOrLongSequence::LongSequence(vec![]) }
    fn ReceiveUnion4(&self) -> StringOrStringSequence { StringOrStringSequence::StringSequence(vec![]) }
    fn ReceiveUnion5(&self) -> BlobOrBlobSequence { BlobOrBlobSequence::BlobSequence(vec![]) }
    fn ReceiveUnion6(&self) -> StringOrUnsignedLong { StringOrUnsignedLong::String(DOMString::new()) }
    fn ReceiveUnion7(&self) -> StringOrBoolean { StringOrBoolean::Boolean(true) }
    fn ReceiveUnion8(&self) -> UnsignedLongOrBoolean { UnsignedLongOrBoolean::UnsignedLong(0u32) }
    fn ReceiveUnion9(&self) -> HTMLElementOrUnsignedLongOrStringOrBoolean {
        HTMLElementOrUnsignedLongOrStringOrBoolean::Boolean(true)
    }
    fn ReceiveUnion10(&self) -> ByteStringOrLong { ByteStringOrLong::ByteString(ByteString::new(vec!())) }
    fn ReceiveUnion11(&self) -> ByteStringSequenceOrLongOrString {
        ByteStringSequenceOrLongOrString::ByteStringSequence(vec!(ByteString::new(vec!())))
    }
    fn ReceiveSequence(&self) -> Vec<i32> { vec![1] }
    fn ReceiveInterfaceSequence(&self) -> Vec<Root<Blob>> {
        vec![Blob::new(self.global().r(), BlobImpl::new_from_bytes(vec![]), "".to_owned())]
    }

    fn ReceiveNullableBoolean(&self) -> Option<bool> { Some(false) }
    fn ReceiveNullableByte(&self) -> Option<i8> { Some(0) }
    fn ReceiveNullableOctet(&self) -> Option<u8> { Some(0) }
    fn ReceiveNullableShort(&self) -> Option<i16> { Some(0) }
    fn ReceiveNullableUnsignedShort(&self) -> Option<u16> { Some(0) }
    fn ReceiveNullableLong(&self) -> Option<i32> { Some(0) }
    fn ReceiveNullableUnsignedLong(&self) -> Option<u32> { Some(0) }
    fn ReceiveNullableLongLong(&self) -> Option<i64> { Some(0) }
    fn ReceiveNullableUnsignedLongLong(&self) -> Option<u64> { Some(0) }
    fn ReceiveNullableUnrestrictedFloat(&self) -> Option<f32> { Some(0.) }
    fn ReceiveNullableFloat(&self) -> Option<Finite<f32>> { Some(Finite::wrap(0.)) }
    fn ReceiveNullableUnrestrictedDouble(&self) -> Option<f64> { Some(0.) }
    fn ReceiveNullableDouble(&self) -> Option<Finite<f64>> { Some(Finite::wrap(0.)) }
    fn ReceiveNullableString(&self) -> Option<DOMString> { Some(DOMString::new()) }
    fn ReceiveNullableUsvstring(&self) -> Option<USVString> { Some(USVString("".to_owned())) }
    fn ReceiveNullableByteString(&self) -> Option<ByteString> { Some(ByteString::new(vec!())) }
    fn ReceiveNullableEnum(&self) -> Option<TestEnum> { Some(TestEnum::_empty) }
    fn ReceiveNullableInterface(&self) -> Option<Root<Blob>> {
        Some(Blob::new(self.global().r(), BlobImpl::new_from_bytes(vec![]), "".to_owned()))
    }
    fn ReceiveNullableObject(&self, _: *mut JSContext) -> *mut JSObject { ptr::null_mut() }
    fn ReceiveNullableUnion(&self) -> Option<HTMLElementOrLong> {
        Some(HTMLElementOrLong::Long(0))
    }
    fn ReceiveNullableUnion2(&self) -> Option<EventOrString> {
        Some(EventOrString::String(DOMString::new()))
    }
    fn ReceiveNullableUnion3(&self) -> Option<StringOrLongSequence> {
        Some(StringOrLongSequence::String(DOMString::new()))
    }
    fn ReceiveNullableUnion4(&self) -> Option<LongSequenceOrBoolean> {
        Some(LongSequenceOrBoolean::Boolean(true))
    }
    fn ReceiveNullableUnion5(&self) -> Option<UnsignedLongOrBoolean> {
        Some(UnsignedLongOrBoolean::UnsignedLong(0u32))
    }
    fn ReceiveNullableUnion6(&self) -> Option<ByteStringOrLong> {
        Some(ByteStringOrLong::ByteString(ByteString::new(vec!())))
    }
    fn ReceiveNullableSequence(&self) -> Option<Vec<i32>> { Some(vec![1]) }
    fn ReceiveTestDictionaryWithSuccessOnKeyword(&self) -> TestDictionary {
        TestDictionary {
            anyValue: NullValue(),
            booleanValue: None,
            byteValue: None,
            dict: TestDictionaryDefaults {
                UnrestrictedDoubleValue: 0.0,
                anyValue: NullValue(),
                booleanValue: false,
                byteValue: 0,
                doubleValue: Finite::new(1.0).unwrap(),
                enumValue: TestEnum::Foo,
                floatValue: Finite::new(1.0).unwrap(),
                longLongValue: 54,
                longValue: 12,
                nullableBooleanValue: None,
                nullableByteValue: None,
                nullableDoubleValue: None,
                nullableFloatValue: None,
                nullableLongLongValue: None,
                nullableLongValue: None,
                nullableObjectValue: ptr::null_mut(),
                nullableOctetValue: None,
                nullableShortValue: None,
                nullableStringValue: None,
                nullableUnrestrictedDoubleValue: None,
                nullableUnrestrictedFloatValue: None,
                nullableUnsignedLongLongValue: None,
                nullableUnsignedLongValue: None,
                nullableUnsignedShortValue: None,
                nullableUsvstringValue: None,
                octetValue: 0,
                shortValue: 0,
                stringValue: DOMString::new(),
                unrestrictedFloatValue: 0.0,
                unsignedLongLongValue: 0,
                unsignedLongValue: 0,
                unsignedShortValue: 0,
                usvstringValue: USVString("".to_owned()),
            },
            doubleValue: None,
            enumValue: None,
            floatValue: None,
            interfaceValue: None,
            longLongValue: None,
            longValue: None,
            objectValue: None,
            octetValue: None,
            requiredValue: true,
            seqDict: None,
            shortValue: None,
            stringValue: None,
            type_: Some(DOMString::from("success")),
            unrestrictedDoubleValue: None,
            unrestrictedFloatValue: None,
            unsignedLongLongValue: None,
            unsignedLongValue: None,
            unsignedShortValue: None,
            usvstringValue: None,
            nonRequiredNullable: None,
            nonRequiredNullable2: Some(None), // null
        }
    }

    fn DictMatchesPassedValues(&self, arg: &TestDictionary) -> bool {
        arg.type_.as_ref().map(|s| s == "success").unwrap_or(false) &&
            arg.nonRequiredNullable.is_none() &&
            arg.nonRequiredNullable2 == Some(None)
    }

    fn PassBoolean(&self, _: bool) {}
    fn PassByte(&self, _: i8) {}
    fn PassOctet(&self, _: u8) {}
    fn PassShort(&self, _: i16) {}
    fn PassUnsignedShort(&self, _: u16) {}
    fn PassLong(&self, _: i32) {}
    fn PassUnsignedLong(&self, _: u32) {}
    fn PassLongLong(&self, _: i64) {}
    fn PassUnsignedLongLong(&self, _: u64) {}
    fn PassUnrestrictedFloat(&self, _: f32) {}
    fn PassFloat(&self, _: Finite<f32>) {}
    fn PassUnrestrictedDouble(&self, _: f64) {}
    fn PassDouble(&self, _: Finite<f64>) {}
    fn PassString(&self, _: DOMString) {}
    fn PassUsvstring(&self, _: USVString) {}
    fn PassByteString(&self, _: ByteString) {}
    fn PassEnum(&self, _: TestEnum) {}
    fn PassInterface(&self, _: &Blob) {}
    fn PassUnion(&self, _: HTMLElementOrLong) {}
    fn PassUnion2(&self, _: EventOrString) {}
    fn PassUnion3(&self, _: BlobOrString) {}
    fn PassUnion4(&self, _: StringOrStringSequence) {}
    fn PassUnion5(&self, _: StringOrBoolean) {}
    fn PassUnion6(&self, _: UnsignedLongOrBoolean) {}
    fn PassUnion7(&self, _: StringSequenceOrUnsignedLong) {}
    fn PassUnion8(&self, _: ByteStringSequenceOrLong) {}
    fn PassAny(&self, _: *mut JSContext, _: HandleValue) {}
    fn PassObject(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassCallbackFunction(&self, _: Rc<Function>) {}
    fn PassCallbackInterface(&self, _: Rc<EventListener>) {}
    fn PassSequence(&self, _: Vec<i32>) {}
    fn PassStringSequence(&self, _: Vec<DOMString>) {}
    fn PassInterfaceSequence(&self, _: Vec<Root<Blob>>) {}

    fn PassNullableBoolean(&self, _: Option<bool>) {}
    fn PassNullableByte(&self, _: Option<i8>) {}
    fn PassNullableOctet(&self, _: Option<u8>) {}
    fn PassNullableShort(&self, _: Option<i16>) {}
    fn PassNullableUnsignedShort(&self, _: Option<u16>) {}
    fn PassNullableLong(&self, _: Option<i32>) {}
    fn PassNullableUnsignedLong(&self, _: Option<u32>) {}
    fn PassNullableLongLong(&self, _: Option<i64>) {}
    fn PassNullableUnsignedLongLong(&self, _: Option<u64>) {}
    fn PassNullableUnrestrictedFloat(&self, _: Option<f32>) {}
    fn PassNullableFloat(&self, _: Option<Finite<f32>>) {}
    fn PassNullableUnrestrictedDouble(&self, _: Option<f64>) {}
    fn PassNullableDouble(&self, _: Option<Finite<f64>>) {}
    fn PassNullableString(&self, _: Option<DOMString>) {}
    fn PassNullableUsvstring(&self, _: Option<USVString>) {}
    fn PassNullableByteString(&self, _: Option<ByteString>) {}
    // fn PassNullableEnum(self, _: Option<TestEnum>) {}
    fn PassNullableInterface(&self, _: Option<&Blob>) {}
    fn PassNullableObject(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassNullableUnion(&self, _: Option<HTMLElementOrLong>) {}
    fn PassNullableUnion2(&self, _: Option<EventOrString>) {}
    fn PassNullableUnion3(&self, _: Option<StringOrLongSequence>) {}
    fn PassNullableUnion4(&self, _: Option<LongSequenceOrBoolean>) {}
    fn PassNullableUnion5(&self, _: Option<UnsignedLongOrBoolean>) {}
    fn PassNullableUnion6(&self, _: Option<ByteStringOrLong>) {}
    fn PassNullableCallbackFunction(&self, _: Option<Rc<Function>>) {}
    fn PassNullableCallbackInterface(&self, _: Option<Rc<EventListener>>) {}
    fn PassNullableSequence(&self, _: Option<Vec<i32>>) {}

    fn PassOptionalBoolean(&self, _: Option<bool>) {}
    fn PassOptionalByte(&self, _: Option<i8>) {}
    fn PassOptionalOctet(&self, _: Option<u8>) {}
    fn PassOptionalShort(&self, _: Option<i16>) {}
    fn PassOptionalUnsignedShort(&self, _: Option<u16>) {}
    fn PassOptionalLong(&self, _: Option<i32>) {}
    fn PassOptionalUnsignedLong(&self, _: Option<u32>) {}
    fn PassOptionalLongLong(&self, _: Option<i64>) {}
    fn PassOptionalUnsignedLongLong(&self, _: Option<u64>) {}
    fn PassOptionalUnrestrictedFloat(&self, _: Option<f32>) {}
    fn PassOptionalFloat(&self, _: Option<Finite<f32>>) {}
    fn PassOptionalUnrestrictedDouble(&self, _: Option<f64>) {}
    fn PassOptionalDouble(&self, _: Option<Finite<f64>>) {}
    fn PassOptionalString(&self, _: Option<DOMString>) {}
    fn PassOptionalUsvstring(&self, _: Option<USVString>) {}
    fn PassOptionalByteString(&self, _: Option<ByteString>) {}
    fn PassOptionalEnum(&self, _: Option<TestEnum>) {}
    fn PassOptionalInterface(&self, _: Option<&Blob>) {}
    fn PassOptionalUnion(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalUnion2(&self, _: Option<EventOrString>) {}
    fn PassOptionalUnion3(&self, _: Option<StringOrLongSequence>) {}
    fn PassOptionalUnion4(&self, _: Option<LongSequenceOrBoolean>) {}
    fn PassOptionalUnion5(&self, _: Option<UnsignedLongOrBoolean>) {}
    fn PassOptionalUnion6(&self, _: Option<ByteStringOrLong>) {}
    fn PassOptionalAny(&self, _: *mut JSContext, _: HandleValue) {}
    fn PassOptionalObject(&self, _: *mut JSContext, _: Option<*mut JSObject>) {}
    fn PassOptionalCallbackFunction(&self, _: Option<Rc<Function>>) {}
    fn PassOptionalCallbackInterface(&self, _: Option<Rc<EventListener>>) {}
    fn PassOptionalSequence(&self, _: Option<Vec<i32>>) {}

    fn PassOptionalNullableBoolean(&self, _: Option<Option<bool>>) {}
    fn PassOptionalNullableByte(&self, _: Option<Option<i8>>) {}
    fn PassOptionalNullableOctet(&self, _: Option<Option<u8>>) {}
    fn PassOptionalNullableShort(&self, _: Option<Option<i16>>) {}
    fn PassOptionalNullableUnsignedShort(&self, _: Option<Option<u16>>) {}
    fn PassOptionalNullableLong(&self, _: Option<Option<i32>>) {}
    fn PassOptionalNullableUnsignedLong(&self, _: Option<Option<u32>>) {}
    fn PassOptionalNullableLongLong(&self, _: Option<Option<i64>>) {}
    fn PassOptionalNullableUnsignedLongLong(&self, _: Option<Option<u64>>) {}
    fn PassOptionalNullableUnrestrictedFloat(&self, _: Option<Option<f32>>) {}
    fn PassOptionalNullableFloat(&self, _: Option<Option<Finite<f32>>>) {}
    fn PassOptionalNullableUnrestrictedDouble(&self, _: Option<Option<f64>>) {}
    fn PassOptionalNullableDouble(&self, _: Option<Option<Finite<f64>>>) {}
    fn PassOptionalNullableString(&self, _: Option<Option<DOMString>>) {}
    fn PassOptionalNullableUsvstring(&self, _: Option<Option<USVString>>) {}
    fn PassOptionalNullableByteString(&self, _: Option<Option<ByteString>>) {}
    // fn PassOptionalNullableEnum(self, _: Option<Option<TestEnum>>) {}
    fn PassOptionalNullableInterface(&self, _: Option<Option<&Blob>>) {}
    fn PassOptionalNullableObject(&self, _: *mut JSContext, _: Option<*mut JSObject>) {}
    fn PassOptionalNullableUnion(&self, _: Option<Option<HTMLElementOrLong>>) {}
    fn PassOptionalNullableUnion2(&self, _: Option<Option<EventOrString>>) {}
    fn PassOptionalNullableUnion3(&self, _: Option<Option<StringOrLongSequence>>) {}
    fn PassOptionalNullableUnion4(&self, _: Option<Option<LongSequenceOrBoolean>>) {}
    fn PassOptionalNullableUnion5(&self, _: Option<Option<UnsignedLongOrBoolean>>) {}
    fn PassOptionalNullableUnion6(&self, _: Option<Option<ByteStringOrLong>>) {}
    fn PassOptionalNullableCallbackFunction(&self, _: Option<Option<Rc<Function>>>) {}
    fn PassOptionalNullableCallbackInterface(&self, _: Option<Option<Rc<EventListener>>>) {}
    fn PassOptionalNullableSequence(&self, _: Option<Option<Vec<i32>>>) {}

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
    fn PassOptionalUsvstringWithDefault(&self, _: USVString) {}
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
    // fn PassOptionalNullableUnrestrictedFloatWithDefault(self, _: Option<f32>) {}
    // fn PassOptionalNullableFloatWithDefault(self, _: Option<Finite<f32>>) {}
    // fn PassOptionalNullableUnrestrictedDoubleWithDefault(self, _: Option<f64>) {}
    // fn PassOptionalNullableDoubleWithDefault(self, _: Option<Finite<f64>>) {}
    fn PassOptionalNullableStringWithDefault(&self, _: Option<DOMString>) {}
    fn PassOptionalNullableUsvstringWithDefault(&self, _: Option<USVString>) {}
    fn PassOptionalNullableByteStringWithDefault(&self, _: Option<ByteString>) {}
    // fn PassOptionalNullableEnumWithDefault(self, _: Option<TestEnum>) {}
    fn PassOptionalNullableInterfaceWithDefault(&self, _: Option<&Blob>) {}
    fn PassOptionalNullableObjectWithDefault(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassOptionalNullableUnionWithDefault(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalNullableUnion2WithDefault(&self, _: Option<EventOrString>) {}
    // fn PassOptionalNullableCallbackFunctionWithDefault(self, _: Option<Function>) {}
    fn PassOptionalNullableCallbackInterfaceWithDefault(&self, _: Option<Rc<EventListener>>) {}
    fn PassOptionalAnyWithDefault(&self, _: *mut JSContext, _: HandleValue) {}

    fn PassOptionalNullableBooleanWithNonNullDefault(&self, _: Option<bool>) {}
    fn PassOptionalNullableByteWithNonNullDefault(&self, _: Option<i8>) {}
    fn PassOptionalNullableOctetWithNonNullDefault(&self, _: Option<u8>) {}
    fn PassOptionalNullableShortWithNonNullDefault(&self, _: Option<i16>) {}
    fn PassOptionalNullableUnsignedShortWithNonNullDefault(&self, _: Option<u16>) {}
    fn PassOptionalNullableLongWithNonNullDefault(&self, _: Option<i32>) {}
    fn PassOptionalNullableUnsignedLongWithNonNullDefault(&self, _: Option<u32>) {}
    fn PassOptionalNullableLongLongWithNonNullDefault(&self, _: Option<i64>) {}
    fn PassOptionalNullableUnsignedLongLongWithNonNullDefault(&self, _: Option<u64>) {}
    // fn PassOptionalNullableUnrestrictedFloatWithNonNullDefault(self, _: Option<f32>) {}
    // fn PassOptionalNullableFloatWithNonNullDefault(self, _: Option<Finite<f32>>) {}
    // fn PassOptionalNullableUnrestrictedDoubleWithNonNullDefault(self, _: Option<f64>) {}
    // fn PassOptionalNullableDoubleWithNonNullDefault(self, _: Option<Finite<f64>>) {}
    fn PassOptionalNullableStringWithNonNullDefault(&self, _: Option<DOMString>) {}
    fn PassOptionalNullableUsvstringWithNonNullDefault(&self, _: Option<USVString>) {}
    // fn PassOptionalNullableEnumWithNonNullDefault(self, _: Option<TestEnum>) {}

    fn PassVariadicBoolean(&self, _: Vec<bool>) {}
    fn PassVariadicBooleanAndDefault(&self, _: bool, _: Vec<bool>) {}
    fn PassVariadicByte(&self, _: Vec<i8>) {}
    fn PassVariadicOctet(&self, _: Vec<u8>) {}
    fn PassVariadicShort(&self, _: Vec<i16>) {}
    fn PassVariadicUnsignedShort(&self, _: Vec<u16>) {}
    fn PassVariadicLong(&self, _: Vec<i32>) {}
    fn PassVariadicUnsignedLong(&self, _: Vec<u32>) {}
    fn PassVariadicLongLong(&self, _: Vec<i64>) {}
    fn PassVariadicUnsignedLongLong(&self, _: Vec<u64>) {}
    fn PassVariadicUnrestrictedFloat(&self, _: Vec<f32>) {}
    fn PassVariadicFloat(&self, _: Vec<Finite<f32>>) {}
    fn PassVariadicUnrestrictedDouble(&self, _: Vec<f64>) {}
    fn PassVariadicDouble(&self, _: Vec<Finite<f64>>) {}
    fn PassVariadicString(&self, _: Vec<DOMString>) {}
    fn PassVariadicUsvstring(&self, _: Vec<USVString>) {}
    fn PassVariadicByteString(&self, _: Vec<ByteString>) {}
    fn PassVariadicEnum(&self, _: Vec<TestEnum>) {}
    fn PassVariadicInterface(&self, _: &[&Blob]) {}
    fn PassVariadicUnion(&self, _: Vec<HTMLElementOrLong>) {}
    fn PassVariadicUnion2(&self, _: Vec<EventOrString>) {}
    fn PassVariadicUnion3(&self, _: Vec<BlobOrString>) {}
    fn PassVariadicUnion4(&self, _: Vec<BlobOrBoolean>) {}
    fn PassVariadicUnion5(&self, _: Vec<StringOrUnsignedLong>) {}
    fn PassVariadicUnion6(&self, _: Vec<UnsignedLongOrBoolean>) {}
    fn PassVariadicUnion7(&self, _: Vec<ByteStringOrLong>) {}
    fn PassVariadicAny(&self, _: *mut JSContext, _: Vec<HandleValue>) {}
    fn PassVariadicObject(&self, _: *mut JSContext, _: Vec<*mut JSObject>) {}
    fn BooleanMozPreference(&self, pref_name: DOMString) -> bool {
        PREFS.get(pref_name.as_ref()).as_boolean().unwrap_or(false)
    }
    fn StringMozPreference(&self, pref_name: DOMString) -> DOMString {
        PREFS.get(pref_name.as_ref()).as_string().map(|s| DOMString::from(s)).unwrap_or_else(|| DOMString::new())
    }
    fn PrefControlledAttributeDisabled(&self) -> bool { false }
    fn PrefControlledAttributeEnabled(&self) -> bool { false }
    fn PrefControlledMethodDisabled(&self) {}
    fn PrefControlledMethodEnabled(&self) {}
    fn FuncControlledAttributeDisabled(&self) -> bool { false }
    fn FuncControlledAttributeEnabled(&self) -> bool { false }
    fn FuncControlledMethodDisabled(&self) {}
    fn FuncControlledMethodEnabled(&self) {}

    fn PassSequenceSequence(&self, _seq: Vec<Vec<i32>>) {}
    fn ReturnSequenceSequence(&self) -> Vec<Vec<i32>> { vec![] }
    fn PassUnionSequenceSequence(&self, seq: LongOrLongSequenceSequence) {
        match seq {
            LongOrLongSequenceSequence::Long(_) => (),
            LongOrLongSequenceSequence::LongSequenceSequence(seq) => {
                let _seq: Vec<Vec<i32>> = seq;
            }
        }
    }

    #[allow(unsafe_code)]
    fn CrashHard(&self) {
        static READ_ONLY_VALUE: i32 = 0;
        unsafe {
            let p: *mut u32 = &READ_ONLY_VALUE as *const _ as *mut _;
            ptr::write_volatile(p, 0xbaadc0de);
        }
    }

    fn AdvanceClock(&self, ms: i32) {
        self.global().r().as_window().advance_animation_clock(ms);
    }

    fn Panic(&self) { panic!("explicit panic from script") }
}

impl TestBinding {
    pub fn BooleanAttributeStatic(_: GlobalRef) -> bool { false }
    pub fn SetBooleanAttributeStatic(_: GlobalRef, _: bool) {}
    pub fn ReceiveVoidStatic(_: GlobalRef) {}
    pub fn PrefControlledStaticAttributeDisabled(_: GlobalRef) -> bool { false }
    pub fn PrefControlledStaticAttributeEnabled(_: GlobalRef) -> bool { false }
    pub fn PrefControlledStaticMethodDisabled(_: GlobalRef) {}
    pub fn PrefControlledStaticMethodEnabled(_: GlobalRef) {}
    pub fn FuncControlledStaticAttributeDisabled(_: GlobalRef) -> bool { false }
    pub fn FuncControlledStaticAttributeEnabled(_: GlobalRef) -> bool { false }
    pub fn FuncControlledStaticMethodDisabled(_: GlobalRef) {}
    pub fn FuncControlledStaticMethodEnabled(_: GlobalRef) {}
}

#[allow(unsafe_code)]
impl TestBinding {
    pub unsafe fn condition_satisfied(_: *mut JSContext, _: HandleObject) -> bool { true }
    pub unsafe fn condition_unsatisfied(_: *mut JSContext, _: HandleObject) -> bool { false }
}
