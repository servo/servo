/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::TestBindingBinding::{self, SimpleCallback};
use dom::bindings::codegen::Bindings::TestBindingBinding::{TestBindingMethods, TestDictionary};
use dom::bindings::codegen::Bindings::TestBindingBinding::{TestDictionaryDefaults, TestEnum};
use dom::bindings::codegen::UnionTypes;
use dom::bindings::codegen::UnionTypes::{BlobOrBoolean, BlobOrBlobSequence, LongOrLongSequenceSequence};
use dom::bindings::codegen::UnionTypes::{BlobOrString, BlobOrUnsignedLong, EventOrString};
use dom::bindings::codegen::UnionTypes::{ByteStringOrLong, ByteStringSequenceOrLongOrString};
use dom::bindings::codegen::UnionTypes::{ByteStringSequenceOrLong, DocumentOrTestTypedef};
use dom::bindings::codegen::UnionTypes::{EventOrUSVString, HTMLElementOrLong, LongSequenceOrTestTypedef};
use dom::bindings::codegen::UnionTypes::{HTMLElementOrUnsignedLongOrStringOrBoolean, LongSequenceOrBoolean};
use dom::bindings::codegen::UnionTypes::{StringOrLongSequence, StringOrStringSequence, StringSequenceOrUnsignedLong};
use dom::bindings::codegen::UnionTypes::{StringOrUnsignedLong, StringOrBoolean, UnsignedLongOrBoolean};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::mozmap::MozMap;
use dom::bindings::nonnull::NonNullJSObjectPtr;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::TrustedPromise;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::{ByteString, DOMString, USVString};
use dom::bindings::trace::RootedTraceableBox;
use dom::bindings::weakref::MutableWeakRef;
use dom::blob::{Blob, BlobImpl};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::promisenativehandler::{PromiseNativeHandler, Callback};
use dom::url::URL;
use dom_struct::dom_struct;
use js::jsapi::{HandleObject, HandleValue, Heap, JSContext, JSObject};
use js::jsapi::{JS_NewPlainObject, JS_NewUint8ClampedArray};
use js::jsval::{JSVal, NullValue};
use js::rust::CustomAutoRooterGuard;
use script_traits::MsDuration;
use servo_config::prefs::PREFS;
use std::borrow::ToOwned;
use std::ptr;
use std::rc::Rc;
use timers::OneshotTimerCallback;

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

    pub fn new(global: &GlobalScope) -> DomRoot<TestBinding> {
        reflect_dom_object(Box::new(TestBinding::new_inherited()),
                           global, TestBindingBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope) -> Fallible<DomRoot<TestBinding>> {
        Ok(TestBinding::new(global))
    }

    #[allow(unused_variables)]
    pub fn Constructor_(global: &GlobalScope, nums: Vec<f64>) -> Fallible<DomRoot<TestBinding>> {
        Ok(TestBinding::new(global))
    }

    #[allow(unused_variables)]
    pub fn Constructor__(global: &GlobalScope, num: f64) -> Fallible<DomRoot<TestBinding>> {
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
    fn InterfaceAttribute(&self) -> DomRoot<Blob> {
        Blob::new(&self.global(), BlobImpl::new_from_bytes(vec![]), "".to_owned())
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
    #[allow(unsafe_code)]
    unsafe fn ArrayAttribute(&self, cx: *mut JSContext) -> NonNullJSObjectPtr {
        rooted!(in(cx) let array = JS_NewUint8ClampedArray(cx, 16));
        assert!(!array.is_null());
        NonNullJSObjectPtr::new_unchecked(array.get())
    }
    #[allow(unsafe_code)]
    unsafe fn AnyAttribute(&self, _: *mut JSContext) -> JSVal { NullValue() }
    #[allow(unsafe_code)]
    unsafe fn SetAnyAttribute(&self, _: *mut JSContext, _: HandleValue) {}
    #[allow(unsafe_code)]
    unsafe fn ObjectAttribute(&self, cx: *mut JSContext) -> NonNullJSObjectPtr {
        rooted!(in(cx) let obj = JS_NewPlainObject(cx));
        assert!(!obj.is_null());
        NonNullJSObjectPtr::new_unchecked(obj.get())
    }
    #[allow(unsafe_code)]
    unsafe fn SetObjectAttribute(&self, _: *mut JSContext, _: *mut JSObject) {}

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
    fn ForwardedAttribute(&self) -> DomRoot<TestBinding> { DomRoot::from_ref(self) }
    fn BinaryRenamedAttribute(&self) -> DOMString { DOMString::new() }
    fn SetBinaryRenamedAttribute2(&self, _: DOMString) {}
    fn BinaryRenamedAttribute2(&self) -> DOMString { DOMString::new() }
    fn Attr_to_automatically_rename(&self) -> DOMString { DOMString::new() }
    fn SetAttr_to_automatically_rename(&self, _: DOMString) {}
    fn GetEnumAttributeNullable(&self) -> Option<TestEnum> { Some(TestEnum::_empty) }
    fn GetInterfaceAttributeNullable(&self) -> Option<DomRoot<Blob>> {
        Some(Blob::new(&self.global(), BlobImpl::new_from_bytes(vec![]), "".to_owned()))
    }
    fn SetInterfaceAttributeNullable(&self, _: Option<&Blob>) {}
    fn GetInterfaceAttributeWeak(&self) -> Option<DomRoot<URL>> {
        self.url.root()
    }
    fn SetInterfaceAttributeWeak(&self, url: Option<&URL>) {
        self.url.set(url);
    }
    #[allow(unsafe_code)]
    unsafe fn GetObjectAttributeNullable(&self, _: *mut JSContext) -> Option<NonNullJSObjectPtr> { None }
    #[allow(unsafe_code)]
    unsafe fn SetObjectAttributeNullable(&self, _: *mut JSContext, _: *mut JSObject) {}
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
    fn BinaryRenamedMethod(&self) {}
    fn ReceiveVoid(&self) {}
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
    fn ReceiveInterface(&self) -> DomRoot<Blob> {
        Blob::new(&self.global(), BlobImpl::new_from_bytes(vec![]), "".to_owned())
    }
    #[allow(unsafe_code)]
    unsafe fn ReceiveAny(&self, _: *mut JSContext) -> JSVal { NullValue() }
    #[allow(unsafe_code)]
    unsafe fn ReceiveObject(&self, cx: *mut JSContext) -> NonNullJSObjectPtr {
        self.ObjectAttribute(cx)
    }
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
    fn ReceiveInterfaceSequence(&self) -> Vec<DomRoot<Blob>> {
        vec![Blob::new(&self.global(), BlobImpl::new_from_bytes(vec![]), "".to_owned())]
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
    fn ReceiveNullableInterface(&self) -> Option<DomRoot<Blob>> {
        Some(Blob::new(&self.global(), BlobImpl::new_from_bytes(vec![]), "".to_owned()))
    }
    #[allow(unsafe_code)]
    unsafe fn ReceiveNullableObject(&self, cx: *mut JSContext) -> Option<NonNullJSObjectPtr> {
        self.GetObjectAttributeNullable(cx)
    }
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
    fn ReceiveTestDictionaryWithSuccessOnKeyword(&self) -> RootedTraceableBox<TestDictionary> {
        RootedTraceableBox::new(TestDictionary {
            anyValue: Heap::default(),
            booleanValue: None,
            byteValue: None,
            dict: RootedTraceableBox::new(TestDictionaryDefaults {
                UnrestrictedDoubleValue: 0.0,
                anyValue: Heap::default(),
                booleanValue: false,
                bytestringValue: ByteString::new(vec![]),
                byteValue: 0,
                doubleValue: Finite::new(1.0).unwrap(),
                enumValue: TestEnum::Foo,
                floatValue: Finite::new(1.0).unwrap(),
                longLongValue: 54,
                longValue: 12,
                nullableBooleanValue: None,
                nullableBytestringValue: None,
                nullableByteValue: None,
                nullableDoubleValue: None,
                nullableFloatValue: None,
                nullableLongLongValue: None,
                nullableLongValue: None,
                nullableObjectValue: Heap::default(),
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
            }),
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
            elementSequence: None,
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
        })
    }

    fn DictMatchesPassedValues(&self, arg: RootedTraceableBox<TestDictionary>) -> bool {
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
    fn PassUnion9(&self, _: UnionTypes::TestDictionaryOrLong) {}
    #[allow(unsafe_code)]
    unsafe fn PassUnion10(&self, _: *mut JSContext, _: UnionTypes::StringOrObject) {}
    fn PassUnionWithTypedef(&self, _: DocumentOrTestTypedef) {}
    fn PassUnionWithTypedef2(&self, _: LongSequenceOrTestTypedef) {}
    #[allow(unsafe_code)]
    unsafe fn PassAny(&self, _: *mut JSContext, _: HandleValue) {}
    #[allow(unsafe_code)]
    unsafe fn PassObject(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassCallbackFunction(&self, _: Rc<Function>) {}
    fn PassCallbackInterface(&self, _: Rc<EventListener>) {}
    fn PassSequence(&self, _: Vec<i32>) {}
    #[allow(unsafe_code)]
    unsafe fn PassAnySequence(&self, _: *mut JSContext, _: CustomAutoRooterGuard<Vec<JSVal>>) {}
    #[allow(unsafe_code)]
    unsafe fn AnySequencePassthrough(&self, _: *mut JSContext, seq: CustomAutoRooterGuard<Vec<JSVal>>) -> Vec<JSVal> {
        (*seq).clone()
    }
    #[allow(unsafe_code)]
    unsafe fn PassObjectSequence(&self, _: *mut JSContext, _: CustomAutoRooterGuard<Vec<*mut JSObject>>) {}
    fn PassStringSequence(&self, _: Vec<DOMString>) {}
    fn PassInterfaceSequence(&self, _: Vec<DomRoot<Blob>>) {}

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
    #[allow(unsafe_code)]
    unsafe fn PassNullableObject(&self, _: *mut JSContext, _: *mut JSObject) {}
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
    #[allow(unsafe_code)]
    unsafe fn PassOptionalAny(&self, _: *mut JSContext, _: HandleValue) {}
    #[allow(unsafe_code)]
    unsafe fn PassOptionalObject(&self, _: *mut JSContext, _: Option<*mut JSObject>) {}
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
    #[allow(unsafe_code)]
    unsafe fn PassOptionalNullableObject(&self, _: *mut JSContext, _: Option<*mut JSObject>) {}
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
    fn PassOptionalBytestringWithDefault(&self, _: ByteString) {}
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
    #[allow(unsafe_code)]
    unsafe fn PassOptionalNullableObjectWithDefault(&self, _: *mut JSContext, _: *mut JSObject) {}
    fn PassOptionalNullableUnionWithDefault(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalNullableUnion2WithDefault(&self, _: Option<EventOrString>) {}
    // fn PassOptionalNullableCallbackFunctionWithDefault(self, _: Option<Function>) {}
    fn PassOptionalNullableCallbackInterfaceWithDefault(&self, _: Option<Rc<EventListener>>) {}
    #[allow(unsafe_code)]
    unsafe fn PassOptionalAnyWithDefault(&self, _: *mut JSContext, _: HandleValue) {}

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
    fn PassOptionalOverloaded(&self, a: &TestBinding, _: u32, _: u32) -> DomRoot<TestBinding> { DomRoot::from_ref(a) }
    fn PassOptionalOverloaded_(&self, _: &Blob,  _: u32) { }

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
    #[allow(unsafe_code)]
    unsafe fn PassVariadicAny(&self, _: *mut JSContext, _: Vec<HandleValue>) {}
    #[allow(unsafe_code)]
    unsafe fn PassVariadicObject(&self, _: *mut JSContext, _: Vec<*mut JSObject>) {}
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

    fn PassMozMap(&self, _: MozMap<i32>) {}
    fn PassNullableMozMap(&self, _: Option<MozMap<i32> >) {}
    fn PassMozMapOfNullableInts(&self, _: MozMap<Option<i32>>) {}
    fn PassOptionalMozMapOfNullableInts(&self, _: Option<MozMap<Option<i32>>>) {}
    fn PassOptionalNullableMozMapOfNullableInts(&self, _: Option<Option<MozMap<Option<i32>> >>) {}
    fn PassCastableObjectMozMap(&self, _: MozMap<DomRoot<TestBinding>>) {}
    fn PassNullableCastableObjectMozMap(&self, _: MozMap<Option<DomRoot<TestBinding>>>) {}
    fn PassCastableObjectNullableMozMap(&self, _: Option<MozMap<DomRoot<TestBinding>>>) {}
    fn PassNullableCastableObjectNullableMozMap(&self, _: Option<MozMap<Option<DomRoot<TestBinding>>>>) {}
    fn PassOptionalMozMap(&self, _: Option<MozMap<i32>>) {}
    fn PassOptionalNullableMozMap(&self, _: Option<Option<MozMap<i32>>>) {}
    fn PassOptionalNullableMozMapWithDefaultValue(&self, _: Option<MozMap<i32>>) {}
    fn PassOptionalObjectMozMap(&self, _: Option<MozMap<DomRoot<TestBinding>>>) {}
    fn PassStringMozMap(&self, _: MozMap<DOMString>) {}
    fn PassByteStringMozMap(&self, _: MozMap<ByteString>) {}
    fn PassMozMapOfMozMaps(&self, _: MozMap<MozMap<i32>>) {}
    fn PassMozMapUnion(&self, _: UnionTypes::LongOrStringByteStringRecord) {}
    fn PassMozMapUnion2(&self, _: UnionTypes::TestBindingOrStringByteStringRecord) {}
    fn PassMozMapUnion3(&self, _: UnionTypes::TestBindingOrByteStringSequenceSequenceOrStringByteStringRecord) {}
    fn ReceiveMozMap(&self) -> MozMap<i32> { MozMap::new() }
    fn ReceiveNullableMozMap(&self) -> Option<MozMap<i32>> { Some(MozMap::new()) }
    fn ReceiveMozMapOfNullableInts(&self) -> MozMap<Option<i32>> { MozMap::new() }
    fn ReceiveNullableMozMapOfNullableInts(&self) -> Option<MozMap<Option<i32>>> { Some(MozMap::new()) }
    fn ReceiveMozMapOfMozMaps(&self) -> MozMap<MozMap<i32>> { MozMap::new() }
    fn ReceiveAnyMozMap(&self) -> MozMap<JSVal> { MozMap::new() }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn ReturnResolvedPromise(&self, cx: *mut JSContext, v: HandleValue) -> Fallible<Rc<Promise>> {
        Promise::new_resolved(&self.global(), cx, v)
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn ReturnRejectedPromise(&self, cx: *mut JSContext, v: HandleValue) -> Fallible<Rc<Promise>> {
        Promise::new_rejected(&self.global(), cx, v)
    }

    #[allow(unsafe_code)]
    unsafe fn PromiseResolveNative(&self, cx: *mut JSContext, p: &Promise, v: HandleValue) {
        p.resolve(cx, v);
    }

    #[allow(unsafe_code)]
    unsafe fn PromiseRejectNative(&self, cx: *mut JSContext, p: &Promise, v: HandleValue) {
        p.reject(cx, v);
    }

    fn PromiseRejectWithTypeError(&self, p: &Promise, s: USVString) {
        p.reject_error(Error::Type(s.0));
    }

    #[allow(unrooted_must_root)]
    fn ResolvePromiseDelayed(&self, p: &Promise, value: DOMString, delay: u64) {
        let promise = p.duplicate();
        let cb = TestBindingCallback {
            promise: TrustedPromise::new(promise),
            value: value,
        };
        let _ = self.global()
            .schedule_callback(
                OneshotTimerCallback::TestBindingCallback(cb),
                MsDuration::new(delay));
    }

    #[allow(unrooted_must_root)]
    fn PromiseNativeHandler(&self,
                            resolve: Option<Rc<SimpleCallback>>,
                            reject: Option<Rc<SimpleCallback>>) -> Rc<Promise> {
        let global = self.global();
        let handler = PromiseNativeHandler::new(&global,
                                                resolve.map(SimpleHandler::new),
                                                reject.map(SimpleHandler::new));
        let p = Promise::new(&global);
        p.append_native_handler(&handler);
        return p;

        #[derive(JSTraceable, MallocSizeOf)]
        struct SimpleHandler {
            #[ignore_malloc_size_of = "Rc has unclear ownership semantics"]
            handler: Rc<SimpleCallback>,
        }
        impl SimpleHandler {
            fn new(callback: Rc<SimpleCallback>) -> Box<Callback> {
                Box::new(SimpleHandler { handler: callback })
            }
        }
        impl Callback for SimpleHandler {
            #[allow(unsafe_code)]
            fn callback(&self, cx: *mut JSContext, v: HandleValue) {
                let global = unsafe { GlobalScope::from_context(cx) };
                let _ = self.handler.Call_(&*global, v, ExceptionHandling::Report);
            }
        }
    }

    #[allow(unrooted_must_root)]
    fn PromiseAttribute(&self) -> Rc<Promise> {
        Promise::new(&self.global())
    }

    fn AcceptPromise(&self, _promise: &Promise) {
    }

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

    fn AdvanceClock(&self, ms: i32, tick: bool) {
        self.global().as_window().advance_animation_clock(ms, tick);
    }

    fn Panic(&self) { panic!("explicit panic from script") }

    fn EntryGlobal(&self) -> DomRoot<GlobalScope> {
        GlobalScope::entry()
    }
    fn IncumbentGlobal(&self) -> DomRoot<GlobalScope> {
        GlobalScope::incumbent().unwrap()
    }
}

impl TestBinding {
    pub fn BooleanAttributeStatic(_: &GlobalScope) -> bool { false }
    pub fn SetBooleanAttributeStatic(_: &GlobalScope, _: bool) {}
    pub fn ReceiveVoidStatic(_: &GlobalScope) {}
    pub fn PrefControlledStaticAttributeDisabled(_: &GlobalScope) -> bool { false }
    pub fn PrefControlledStaticAttributeEnabled(_: &GlobalScope) -> bool { false }
    pub fn PrefControlledStaticMethodDisabled(_: &GlobalScope) {}
    pub fn PrefControlledStaticMethodEnabled(_: &GlobalScope) {}
    pub fn FuncControlledStaticAttributeDisabled(_: &GlobalScope) -> bool { false }
    pub fn FuncControlledStaticAttributeEnabled(_: &GlobalScope) -> bool { false }
    pub fn FuncControlledStaticMethodDisabled(_: &GlobalScope) {}
    pub fn FuncControlledStaticMethodEnabled(_: &GlobalScope) {}
}

#[allow(unsafe_code)]
impl TestBinding {
    pub unsafe fn condition_satisfied(_: *mut JSContext, _: HandleObject) -> bool { true }
    pub unsafe fn condition_unsatisfied(_: *mut JSContext, _: HandleObject) -> bool { false }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct TestBindingCallback {
    #[ignore_malloc_size_of = "unclear ownership semantics"]
    promise: TrustedPromise,
    value: DOMString,
}

impl TestBindingCallback {
    #[allow(unrooted_must_root)]
    pub fn invoke(self) {
        self.promise.root().resolve_native(&self.value);
    }
}
