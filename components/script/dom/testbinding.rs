/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use std::borrow::ToOwned;
use std::ptr::{self, NonNull};
use std::rc::Rc;
use std::time::Duration;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, JS_NewPlainObject};
use js::jsval::JSVal;
use js::rust::{CustomAutoRooterGuard, HandleObject, HandleValue, MutableHandleValue};
use js::typedarray::{self, Uint8ClampedArray};
use script_traits::serializable::BlobImpl;
use servo_config::prefs;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::TestBindingBinding::{
    SimpleCallback, TestBindingMethods, TestDictionary, TestDictionaryDefaults,
    TestDictionaryParent, TestDictionaryWithParent, TestEnum, TestURLLike,
};
use crate::dom::bindings::codegen::UnionTypes;
use crate::dom::bindings::codegen::UnionTypes::{
    BlobOrBlobSequence, BlobOrBoolean, BlobOrString, BlobOrUnsignedLong, ByteStringOrLong,
    ByteStringSequenceOrLong, ByteStringSequenceOrLongOrString, EventOrString, EventOrUSVString,
    HTMLElementOrLong, HTMLElementOrUnsignedLongOrStringOrBoolean, LongOrLongSequenceSequence,
    LongSequenceOrBoolean, StringOrBoolean, StringOrLongSequence, StringOrStringSequence,
    StringOrUnsignedLong, StringSequenceOrUnsignedLong, UnsignedLongOrBoolean,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::record::Record;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{ByteString, DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::weakref::MutableWeakRef;
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::Node;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::url::URL;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::timers::OneshotTimerCallback;

#[dom_struct]
pub(crate) struct TestBinding {
    reflector_: Reflector,
    url: MutableWeakRef<URL>,
}

#[allow(non_snake_case)]
impl TestBinding {
    fn new_inherited() -> TestBinding {
        TestBinding {
            reflector_: Reflector::new(),
            url: MutableWeakRef::new(None),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<TestBinding> {
        reflect_dom_object_with_proto(
            Box::new(TestBinding::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }
}

impl TestBindingMethods<crate::DomTypeHolder> for TestBinding {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TestBinding>> {
        Ok(TestBinding::new(global, proto, can_gc))
    }

    #[allow(unused_variables)]
    fn Constructor_(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        nums: Vec<f64>,
    ) -> Fallible<DomRoot<TestBinding>> {
        Ok(TestBinding::new(global, proto, can_gc))
    }

    #[allow(unused_variables)]
    fn Constructor__(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        num: f64,
    ) -> Fallible<DomRoot<TestBinding>> {
        Ok(TestBinding::new(global, proto, can_gc))
    }

    fn BooleanAttribute(&self) -> bool {
        false
    }
    fn SetBooleanAttribute(&self, _: bool) {}
    fn ByteAttribute(&self) -> i8 {
        0
    }
    fn SetByteAttribute(&self, _: i8) {}
    fn OctetAttribute(&self) -> u8 {
        0
    }
    fn SetOctetAttribute(&self, _: u8) {}
    fn ShortAttribute(&self) -> i16 {
        0
    }
    fn SetShortAttribute(&self, _: i16) {}
    fn UnsignedShortAttribute(&self) -> u16 {
        0
    }
    fn SetUnsignedShortAttribute(&self, _: u16) {}
    fn LongAttribute(&self) -> i32 {
        0
    }
    fn SetLongAttribute(&self, _: i32) {}
    fn UnsignedLongAttribute(&self) -> u32 {
        0
    }
    fn SetUnsignedLongAttribute(&self, _: u32) {}
    fn LongLongAttribute(&self) -> i64 {
        0
    }
    fn SetLongLongAttribute(&self, _: i64) {}
    fn UnsignedLongLongAttribute(&self) -> u64 {
        0
    }
    fn SetUnsignedLongLongAttribute(&self, _: u64) {}
    fn UnrestrictedFloatAttribute(&self) -> f32 {
        0.
    }
    fn SetUnrestrictedFloatAttribute(&self, _: f32) {}
    fn FloatAttribute(&self) -> Finite<f32> {
        Finite::wrap(0.)
    }
    fn SetFloatAttribute(&self, _: Finite<f32>) {}
    fn UnrestrictedDoubleAttribute(&self) -> f64 {
        0.
    }
    fn SetUnrestrictedDoubleAttribute(&self, _: f64) {}
    fn DoubleAttribute(&self) -> Finite<f64> {
        Finite::wrap(0.)
    }
    fn SetDoubleAttribute(&self, _: Finite<f64>) {}
    fn StringAttribute(&self) -> DOMString {
        DOMString::new()
    }
    fn SetStringAttribute(&self, _: DOMString) {}
    fn UsvstringAttribute(&self) -> USVString {
        USVString("".to_owned())
    }
    fn SetUsvstringAttribute(&self, _: USVString) {}
    fn ByteStringAttribute(&self) -> ByteString {
        ByteString::new(vec![])
    }
    fn SetByteStringAttribute(&self, _: ByteString) {}
    fn EnumAttribute(&self) -> TestEnum {
        TestEnum::_empty
    }
    fn SetEnumAttribute(&self, _: TestEnum) {}
    fn InterfaceAttribute(&self, can_gc: CanGc) -> DomRoot<Blob> {
        Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(vec![], "".to_owned()),
            can_gc,
        )
    }
    fn SetInterfaceAttribute(&self, _: &Blob) {}
    fn UnionAttribute(&self) -> HTMLElementOrLong {
        HTMLElementOrLong::Long(0)
    }
    fn SetUnionAttribute(&self, _: HTMLElementOrLong) {}
    fn Union2Attribute(&self) -> EventOrString {
        EventOrString::String(DOMString::new())
    }
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
        ByteStringOrLong::ByteString(ByteString::new(vec![]))
    }
    fn SetUnion9Attribute(&self, _: ByteStringOrLong) {}
    fn ArrayAttribute(&self, cx: SafeJSContext) -> Uint8ClampedArray {
        let data: [u8; 16] = [0; 16];

        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        create_buffer_source(cx, &data, array.handle_mut())
            .expect("Creating ClampedU8 array should never fail")
    }
    fn AnyAttribute(&self, _: SafeJSContext, _: MutableHandleValue) {}
    fn SetAnyAttribute(&self, _: SafeJSContext, _: HandleValue) {}
    #[allow(unsafe_code)]
    fn ObjectAttribute(&self, cx: SafeJSContext) -> NonNull<JSObject> {
        unsafe {
            rooted!(in(*cx) let obj = JS_NewPlainObject(*cx));
            NonNull::new(obj.get()).expect("got a null pointer")
        }
    }
    fn SetObjectAttribute(&self, _: SafeJSContext, _: *mut JSObject) {}

    fn GetBooleanAttributeNullable(&self) -> Option<bool> {
        Some(false)
    }
    fn SetBooleanAttributeNullable(&self, _: Option<bool>) {}
    fn GetByteAttributeNullable(&self) -> Option<i8> {
        Some(0)
    }
    fn SetByteAttributeNullable(&self, _: Option<i8>) {}
    fn GetOctetAttributeNullable(&self) -> Option<u8> {
        Some(0)
    }
    fn SetOctetAttributeNullable(&self, _: Option<u8>) {}
    fn GetShortAttributeNullable(&self) -> Option<i16> {
        Some(0)
    }
    fn SetShortAttributeNullable(&self, _: Option<i16>) {}
    fn GetUnsignedShortAttributeNullable(&self) -> Option<u16> {
        Some(0)
    }
    fn SetUnsignedShortAttributeNullable(&self, _: Option<u16>) {}
    fn GetLongAttributeNullable(&self) -> Option<i32> {
        Some(0)
    }
    fn SetLongAttributeNullable(&self, _: Option<i32>) {}
    fn GetUnsignedLongAttributeNullable(&self) -> Option<u32> {
        Some(0)
    }
    fn SetUnsignedLongAttributeNullable(&self, _: Option<u32>) {}
    fn GetLongLongAttributeNullable(&self) -> Option<i64> {
        Some(0)
    }
    fn SetLongLongAttributeNullable(&self, _: Option<i64>) {}
    fn GetUnsignedLongLongAttributeNullable(&self) -> Option<u64> {
        Some(0)
    }
    fn SetUnsignedLongLongAttributeNullable(&self, _: Option<u64>) {}
    fn GetUnrestrictedFloatAttributeNullable(&self) -> Option<f32> {
        Some(0.)
    }
    fn SetUnrestrictedFloatAttributeNullable(&self, _: Option<f32>) {}
    fn GetFloatAttributeNullable(&self) -> Option<Finite<f32>> {
        Some(Finite::wrap(0.))
    }
    fn SetFloatAttributeNullable(&self, _: Option<Finite<f32>>) {}
    fn GetUnrestrictedDoubleAttributeNullable(&self) -> Option<f64> {
        Some(0.)
    }
    fn SetUnrestrictedDoubleAttributeNullable(&self, _: Option<f64>) {}
    fn GetDoubleAttributeNullable(&self) -> Option<Finite<f64>> {
        Some(Finite::wrap(0.))
    }
    fn SetDoubleAttributeNullable(&self, _: Option<Finite<f64>>) {}
    fn GetByteStringAttributeNullable(&self) -> Option<ByteString> {
        Some(ByteString::new(vec![]))
    }
    fn SetByteStringAttributeNullable(&self, _: Option<ByteString>) {}
    fn GetStringAttributeNullable(&self) -> Option<DOMString> {
        Some(DOMString::new())
    }
    fn SetStringAttributeNullable(&self, _: Option<DOMString>) {}
    fn GetUsvstringAttributeNullable(&self) -> Option<USVString> {
        Some(USVString("".to_owned()))
    }
    fn SetUsvstringAttributeNullable(&self, _: Option<USVString>) {}
    fn SetBinaryRenamedAttribute(&self, _: DOMString) {}
    fn ForwardedAttribute(&self) -> DomRoot<TestBinding> {
        DomRoot::from_ref(self)
    }
    fn BinaryRenamedAttribute(&self) -> DOMString {
        DOMString::new()
    }
    fn SetBinaryRenamedAttribute2(&self, _: DOMString) {}
    fn BinaryRenamedAttribute2(&self) -> DOMString {
        DOMString::new()
    }
    fn Attr_to_automatically_rename(&self) -> DOMString {
        DOMString::new()
    }
    fn SetAttr_to_automatically_rename(&self, _: DOMString) {}
    fn GetEnumAttributeNullable(&self) -> Option<TestEnum> {
        Some(TestEnum::_empty)
    }
    fn GetInterfaceAttributeNullable(&self, can_gc: CanGc) -> Option<DomRoot<Blob>> {
        Some(Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(vec![], "".to_owned()),
            can_gc,
        ))
    }
    fn SetInterfaceAttributeNullable(&self, _: Option<&Blob>) {}
    fn GetInterfaceAttributeWeak(&self) -> Option<DomRoot<URL>> {
        self.url.root()
    }
    fn SetInterfaceAttributeWeak(&self, url: Option<&URL>) {
        self.url.set(url);
    }
    fn GetObjectAttributeNullable(&self, _: SafeJSContext) -> Option<NonNull<JSObject>> {
        None
    }
    fn SetObjectAttributeNullable(&self, _: SafeJSContext, _: *mut JSObject) {}
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
        Some(ByteStringOrLong::ByteString(ByteString::new(vec![])))
    }
    fn SetUnion6AttributeNullable(&self, _: Option<ByteStringOrLong>) {}
    fn BinaryRenamedMethod(&self) {}
    fn ReceiveVoid(&self) {}
    fn ReceiveBoolean(&self) -> bool {
        false
    }
    fn ReceiveByte(&self) -> i8 {
        0
    }
    fn ReceiveOctet(&self) -> u8 {
        0
    }
    fn ReceiveShort(&self) -> i16 {
        0
    }
    fn ReceiveUnsignedShort(&self) -> u16 {
        0
    }
    fn ReceiveLong(&self) -> i32 {
        0
    }
    fn ReceiveUnsignedLong(&self) -> u32 {
        0
    }
    fn ReceiveLongLong(&self) -> i64 {
        0
    }
    fn ReceiveUnsignedLongLong(&self) -> u64 {
        0
    }
    fn ReceiveUnrestrictedFloat(&self) -> f32 {
        0.
    }
    fn ReceiveFloat(&self) -> Finite<f32> {
        Finite::wrap(0.)
    }
    fn ReceiveUnrestrictedDouble(&self) -> f64 {
        0.
    }
    fn ReceiveDouble(&self) -> Finite<f64> {
        Finite::wrap(0.)
    }
    fn ReceiveString(&self) -> DOMString {
        DOMString::new()
    }
    fn ReceiveUsvstring(&self) -> USVString {
        USVString("".to_owned())
    }
    fn ReceiveByteString(&self) -> ByteString {
        ByteString::new(vec![])
    }
    fn ReceiveEnum(&self) -> TestEnum {
        TestEnum::_empty
    }
    fn ReceiveInterface(&self, can_gc: CanGc) -> DomRoot<Blob> {
        Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(vec![], "".to_owned()),
            can_gc,
        )
    }
    fn ReceiveAny(&self, _: SafeJSContext, _: MutableHandleValue) {}
    fn ReceiveObject(&self, cx: SafeJSContext) -> NonNull<JSObject> {
        self.ObjectAttribute(cx)
    }
    fn ReceiveUnion(&self) -> HTMLElementOrLong {
        HTMLElementOrLong::Long(0)
    }
    fn ReceiveUnion2(&self) -> EventOrString {
        EventOrString::String(DOMString::new())
    }
    fn ReceiveUnion3(&self) -> StringOrLongSequence {
        StringOrLongSequence::LongSequence(vec![])
    }
    fn ReceiveUnion4(&self) -> StringOrStringSequence {
        StringOrStringSequence::StringSequence(vec![])
    }
    fn ReceiveUnion5(&self) -> BlobOrBlobSequence {
        BlobOrBlobSequence::BlobSequence(vec![])
    }
    fn ReceiveUnion6(&self) -> StringOrUnsignedLong {
        StringOrUnsignedLong::String(DOMString::new())
    }
    fn ReceiveUnion7(&self) -> StringOrBoolean {
        StringOrBoolean::Boolean(true)
    }
    fn ReceiveUnion8(&self) -> UnsignedLongOrBoolean {
        UnsignedLongOrBoolean::UnsignedLong(0u32)
    }
    fn ReceiveUnion9(&self) -> HTMLElementOrUnsignedLongOrStringOrBoolean {
        HTMLElementOrUnsignedLongOrStringOrBoolean::Boolean(true)
    }
    fn ReceiveUnion10(&self) -> ByteStringOrLong {
        ByteStringOrLong::ByteString(ByteString::new(vec![]))
    }
    fn ReceiveUnion11(&self) -> ByteStringSequenceOrLongOrString {
        ByteStringSequenceOrLongOrString::ByteStringSequence(vec![ByteString::new(vec![])])
    }
    fn ReceiveSequence(&self) -> Vec<i32> {
        vec![1]
    }
    fn ReceiveInterfaceSequence(&self, can_gc: CanGc) -> Vec<DomRoot<Blob>> {
        vec![Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(vec![], "".to_owned()),
            can_gc,
        )]
    }
    fn ReceiveUnionIdentity(
        &self,
        _: SafeJSContext,
        arg: UnionTypes::StringOrObject,
    ) -> UnionTypes::StringOrObject {
        arg
    }

    fn ReceiveNullableBoolean(&self) -> Option<bool> {
        Some(false)
    }
    fn ReceiveNullableByte(&self) -> Option<i8> {
        Some(0)
    }
    fn ReceiveNullableOctet(&self) -> Option<u8> {
        Some(0)
    }
    fn ReceiveNullableShort(&self) -> Option<i16> {
        Some(0)
    }
    fn ReceiveNullableUnsignedShort(&self) -> Option<u16> {
        Some(0)
    }
    fn ReceiveNullableLong(&self) -> Option<i32> {
        Some(0)
    }
    fn ReceiveNullableUnsignedLong(&self) -> Option<u32> {
        Some(0)
    }
    fn ReceiveNullableLongLong(&self) -> Option<i64> {
        Some(0)
    }
    fn ReceiveNullableUnsignedLongLong(&self) -> Option<u64> {
        Some(0)
    }
    fn ReceiveNullableUnrestrictedFloat(&self) -> Option<f32> {
        Some(0.)
    }
    fn ReceiveNullableFloat(&self) -> Option<Finite<f32>> {
        Some(Finite::wrap(0.))
    }
    fn ReceiveNullableUnrestrictedDouble(&self) -> Option<f64> {
        Some(0.)
    }
    fn ReceiveNullableDouble(&self) -> Option<Finite<f64>> {
        Some(Finite::wrap(0.))
    }
    fn ReceiveNullableString(&self) -> Option<DOMString> {
        Some(DOMString::new())
    }
    fn ReceiveNullableUsvstring(&self) -> Option<USVString> {
        Some(USVString("".to_owned()))
    }
    fn ReceiveNullableByteString(&self) -> Option<ByteString> {
        Some(ByteString::new(vec![]))
    }
    fn ReceiveNullableEnum(&self) -> Option<TestEnum> {
        Some(TestEnum::_empty)
    }
    fn ReceiveNullableInterface(&self, can_gc: CanGc) -> Option<DomRoot<Blob>> {
        Some(Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(vec![], "".to_owned()),
            can_gc,
        ))
    }
    fn ReceiveNullableObject(&self, cx: SafeJSContext) -> Option<NonNull<JSObject>> {
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
        Some(ByteStringOrLong::ByteString(ByteString::new(vec![])))
    }
    fn ReceiveNullableSequence(&self) -> Option<Vec<i32>> {
        Some(vec![1])
    }
    fn ReceiveTestDictionaryWithSuccessOnKeyword(&self) -> RootedTraceableBox<TestDictionary> {
        RootedTraceableBox::new(TestDictionary {
            anyValue: RootedTraceableBox::new(Heap::default()),
            booleanValue: None,
            byteValue: None,
            dict: RootedTraceableBox::new(TestDictionaryDefaults {
                UnrestrictedDoubleValue: 0.0,
                anyValue: RootedTraceableBox::new(Heap::default()),
                arrayValue: Vec::new(),
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
                nullableObjectValue: RootedTraceableBox::new(Heap::default()),
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
            nonRequiredNullable2: Some(None),
            noCallbackImport: None,
            noCallbackImport2: None,
        })
    }

    fn DictMatchesPassedValues(&self, arg: RootedTraceableBox<TestDictionary>) -> bool {
        arg.type_.as_ref().map(|s| s == "success").unwrap_or(false) &&
            arg.nonRequiredNullable.is_none() &&
            arg.nonRequiredNullable2 == Some(None) &&
            arg.noCallbackImport.is_none() &&
            arg.noCallbackImport2.is_none()
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
    fn PassTypedArray(&self, _: CustomAutoRooterGuard<typedarray::Int8Array>) {}
    fn PassTypedArray2(&self, _: CustomAutoRooterGuard<typedarray::ArrayBuffer>) {}
    fn PassTypedArray3(&self, _: CustomAutoRooterGuard<typedarray::ArrayBufferView>) {}
    fn PassUnion(&self, _: HTMLElementOrLong) {}
    fn PassUnion2(&self, _: EventOrString) {}
    fn PassUnion3(&self, _: BlobOrString) {}
    fn PassUnion4(&self, _: StringOrStringSequence) {}
    fn PassUnion5(&self, _: StringOrBoolean) {}
    fn PassUnion6(&self, _: UnsignedLongOrBoolean) {}
    fn PassUnion7(&self, _: StringSequenceOrUnsignedLong) {}
    fn PassUnion8(&self, _: ByteStringSequenceOrLong) {}
    fn PassUnion9(&self, _: UnionTypes::TestDictionaryOrLong) {}
    fn PassUnion10(&self, _: SafeJSContext, _: UnionTypes::StringOrObject) {}
    fn PassUnion11(&self, _: UnionTypes::ArrayBufferOrArrayBufferView) {}
    fn PassUnionWithTypedef(&self, _: UnionTypes::DocumentOrStringOrURLOrBlob) {}
    fn PassUnionWithTypedef2(&self, _: UnionTypes::LongSequenceOrStringOrURLOrBlob) {}
    fn PassAny(&self, _: SafeJSContext, _: HandleValue) {}
    fn PassObject(&self, _: SafeJSContext, _: *mut JSObject) {}
    fn PassCallbackFunction(&self, _: Rc<Function>) {}
    fn PassCallbackInterface(&self, _: Rc<EventListener>) {}
    fn PassSequence(&self, _: Vec<i32>) {}
    fn PassAnySequence(&self, _: SafeJSContext, _: CustomAutoRooterGuard<Vec<JSVal>>) {}
    fn AnySequencePassthrough(
        &self,
        _: SafeJSContext,
        seq: CustomAutoRooterGuard<Vec<JSVal>>,
    ) -> Vec<JSVal> {
        (*seq).clone()
    }
    fn PassObjectSequence(&self, _: SafeJSContext, _: CustomAutoRooterGuard<Vec<*mut JSObject>>) {}
    fn PassStringSequence(&self, _: Vec<DOMString>) {}
    fn PassInterfaceSequence(&self, _: Vec<DomRoot<Blob>>) {}

    fn PassOverloaded(&self, _: CustomAutoRooterGuard<typedarray::ArrayBuffer>) {}
    fn PassOverloaded_(&self, _: DOMString) {}

    fn PassOverloadedDict(&self, _: &Node) -> DOMString {
        "node".into()
    }

    fn PassOverloadedDict_(&self, u: &TestURLLike) -> DOMString {
        u.href.clone()
    }

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
    fn PassNullableObject(&self, _: SafeJSContext, _: *mut JSObject) {}
    fn PassNullableTypedArray(&self, _: CustomAutoRooterGuard<Option<typedarray::Int8Array>>) {}
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
    fn PassOptionalAny(&self, _: SafeJSContext, _: HandleValue) {}
    fn PassOptionalObject(&self, _: SafeJSContext, _: Option<*mut JSObject>) {}
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
    fn PassOptionalNullableObject(&self, _: SafeJSContext, _: Option<*mut JSObject>) {}
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
    fn PassOptionalSequenceWithDefault(&self, _: Vec<i32>) {}

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
    fn PassOptionalNullableObjectWithDefault(&self, _: SafeJSContext, _: *mut JSObject) {}
    fn PassOptionalNullableUnionWithDefault(&self, _: Option<HTMLElementOrLong>) {}
    fn PassOptionalNullableUnion2WithDefault(&self, _: Option<EventOrString>) {}
    // fn PassOptionalNullableCallbackFunctionWithDefault(self, _: Option<Function>) {}
    fn PassOptionalNullableCallbackInterfaceWithDefault(&self, _: Option<Rc<EventListener>>) {}
    fn PassOptionalAnyWithDefault(&self, _: SafeJSContext, _: HandleValue) {}

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
    fn PassOptionalOverloaded(&self, a: &TestBinding, _: u32, _: u32) -> DomRoot<TestBinding> {
        DomRoot::from_ref(a)
    }
    fn PassOptionalOverloaded_(&self, _: &Blob, _: u32) {}

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
    fn PassVariadicAny(&self, _: SafeJSContext, _: Vec<HandleValue>) {}
    fn PassVariadicObject(&self, _: SafeJSContext, _: Vec<*mut JSObject>) {}
    fn BooleanMozPreference(&self, pref_name: DOMString) -> bool {
        prefs::get()
            .get_value(pref_name.as_ref())
            .try_into()
            .unwrap_or(false)
    }
    fn StringMozPreference(&self, pref_name: DOMString) -> DOMString {
        DOMString::from_string(
            prefs::get()
                .get_value(pref_name.as_ref())
                .try_into()
                .unwrap_or_default(),
        )
    }
    fn PrefControlledAttributeDisabled(&self) -> bool {
        false
    }
    fn PrefControlledAttributeEnabled(&self) -> bool {
        false
    }
    fn PrefControlledMethodDisabled(&self) {}
    fn PrefControlledMethodEnabled(&self) {}
    fn FuncControlledAttributeDisabled(&self) -> bool {
        false
    }
    fn FuncControlledAttributeEnabled(&self) -> bool {
        false
    }
    fn FuncControlledMethodDisabled(&self) {}
    fn FuncControlledMethodEnabled(&self) {}

    fn PassRecord(&self, _: Record<DOMString, i32>) {}
    fn PassRecordWithUSVStringKey(&self, _: Record<USVString, i32>) {}
    fn PassRecordWithByteStringKey(&self, _: Record<ByteString, i32>) {}
    fn PassNullableRecord(&self, _: Option<Record<DOMString, i32>>) {}
    fn PassRecordOfNullableInts(&self, _: Record<DOMString, Option<i32>>) {}
    fn PassOptionalRecordOfNullableInts(&self, _: Option<Record<DOMString, Option<i32>>>) {}
    fn PassOptionalNullableRecordOfNullableInts(
        &self,
        _: Option<Option<Record<DOMString, Option<i32>>>>,
    ) {
    }
    fn PassCastableObjectRecord(&self, _: Record<DOMString, DomRoot<TestBinding>>) {}
    fn PassNullableCastableObjectRecord(&self, _: Record<DOMString, Option<DomRoot<TestBinding>>>) {
    }
    fn PassCastableObjectNullableRecord(&self, _: Option<Record<DOMString, DomRoot<TestBinding>>>) {
    }
    fn PassNullableCastableObjectNullableRecord(
        &self,
        _: Option<Record<DOMString, Option<DomRoot<TestBinding>>>>,
    ) {
    }
    fn PassOptionalRecord(&self, _: Option<Record<DOMString, i32>>) {}
    fn PassOptionalNullableRecord(&self, _: Option<Option<Record<DOMString, i32>>>) {}
    fn PassOptionalNullableRecordWithDefaultValue(&self, _: Option<Record<DOMString, i32>>) {}
    fn PassOptionalObjectRecord(&self, _: Option<Record<DOMString, DomRoot<TestBinding>>>) {}
    fn PassStringRecord(&self, _: Record<DOMString, DOMString>) {}
    fn PassByteStringRecord(&self, _: Record<DOMString, ByteString>) {}
    fn PassRecordOfRecords(&self, _: Record<DOMString, Record<DOMString, i32>>) {}
    fn PassRecordUnion(&self, _: UnionTypes::LongOrStringByteStringRecord) {}
    fn PassRecordUnion2(&self, _: UnionTypes::TestBindingOrStringByteStringRecord) {}
    fn PassRecordUnion3(
        &self,
        _: UnionTypes::TestBindingOrByteStringSequenceSequenceOrStringByteStringRecord,
    ) {
    }
    fn ReceiveRecord(&self) -> Record<DOMString, i32> {
        Record::new()
    }
    fn ReceiveRecordWithUSVStringKey(&self) -> Record<USVString, i32> {
        Record::new()
    }
    fn ReceiveRecordWithByteStringKey(&self) -> Record<ByteString, i32> {
        Record::new()
    }
    fn ReceiveNullableRecord(&self) -> Option<Record<DOMString, i32>> {
        Some(Record::new())
    }
    fn ReceiveRecordOfNullableInts(&self) -> Record<DOMString, Option<i32>> {
        Record::new()
    }
    fn ReceiveNullableRecordOfNullableInts(&self) -> Option<Record<DOMString, Option<i32>>> {
        Some(Record::new())
    }
    fn ReceiveRecordOfRecords(&self) -> Record<DOMString, Record<DOMString, i32>> {
        Record::new()
    }
    fn ReceiveAnyRecord(&self) -> Record<DOMString, JSVal> {
        Record::new()
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn ReturnResolvedPromise(&self, cx: SafeJSContext, v: HandleValue) -> Rc<Promise> {
        Promise::new_resolved(&self.global(), cx, v)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn ReturnRejectedPromise(&self, cx: SafeJSContext, v: HandleValue) -> Rc<Promise> {
        Promise::new_rejected(&self.global(), cx, v)
    }

    fn PromiseResolveNative(&self, cx: SafeJSContext, p: &Promise, v: HandleValue) {
        p.resolve(cx, v);
    }

    fn PromiseRejectNative(&self, cx: SafeJSContext, p: &Promise, v: HandleValue) {
        p.reject(cx, v);
    }

    fn PromiseRejectWithTypeError(&self, p: &Promise, s: USVString) {
        p.reject_error(Error::Type(s.0));
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn ResolvePromiseDelayed(&self, p: &Promise, value: DOMString, delay: u64) {
        let promise = p.duplicate();
        let cb = TestBindingCallback {
            promise: TrustedPromise::new(promise),
            value,
        };
        let _ = self.global().schedule_callback(
            OneshotTimerCallback::TestBindingCallback(cb),
            Duration::from_millis(delay),
        );
    }

    fn PromiseNativeHandler(
        &self,
        resolve: Option<Rc<SimpleCallback>>,
        reject: Option<Rc<SimpleCallback>>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let global = self.global();
        let handler = PromiseNativeHandler::new(
            &global,
            resolve.map(SimpleHandler::new_boxed),
            reject.map(SimpleHandler::new_boxed),
            can_gc,
        );
        let p = Promise::new_in_current_realm(comp, can_gc);
        p.append_native_handler(&handler, comp, can_gc);
        return p;

        #[derive(JSTraceable, MallocSizeOf)]
        struct SimpleHandler {
            #[ignore_malloc_size_of = "Rc has unclear ownership semantics"]
            handler: Rc<SimpleCallback>,
        }
        impl SimpleHandler {
            fn new_boxed(callback: Rc<SimpleCallback>) -> Box<dyn Callback> {
                Box::new(SimpleHandler { handler: callback })
            }
        }
        impl Callback for SimpleHandler {
            fn callback(&self, cx: SafeJSContext, v: HandleValue, realm: InRealm, _can_gc: CanGc) {
                let global = GlobalScope::from_safe_context(cx, realm);
                let _ = self.handler.Call_(&*global, v, ExceptionHandling::Report);
            }
        }
    }

    fn PromiseAttribute(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        Promise::new_in_current_realm(comp, can_gc)
    }

    fn AcceptPromise(&self, _promise: &Promise) {}

    fn PassSequenceSequence(&self, _seq: Vec<Vec<i32>>) {}
    fn ReturnSequenceSequence(&self) -> Vec<Vec<i32>> {
        vec![]
    }
    fn PassUnionSequenceSequence(&self, seq: LongOrLongSequenceSequence) {
        match seq {
            LongOrLongSequenceSequence::Long(_) => (),
            LongOrLongSequenceSequence::LongSequenceSequence(seq) => {
                let _seq: Vec<Vec<i32>> = seq;
            },
        }
    }

    #[allow(unsafe_code)]
    fn CrashHard(&self) {
        unsafe { std::ptr::null_mut::<i32>().write(42) }
    }

    fn AdvanceClock(&self, ms: i32) {
        self.global().as_window().advance_animation_clock(ms);
    }

    fn Panic(&self) {
        panic!("explicit panic from script")
    }

    fn EntryGlobal(&self) -> DomRoot<GlobalScope> {
        GlobalScope::entry()
    }
    fn IncumbentGlobal(&self) -> DomRoot<GlobalScope> {
        GlobalScope::incumbent().unwrap()
    }

    fn SemiExposedBoolFromInterface(&self) -> bool {
        true
    }

    fn BoolFromSemiExposedPartialInterface(&self) -> bool {
        true
    }

    fn SemiExposedBoolFromPartialInterface(&self) -> bool {
        true
    }

    fn GetDictionaryWithParent(&self, s1: DOMString, s2: DOMString) -> TestDictionaryWithParent {
        TestDictionaryWithParent {
            parent: TestDictionaryParent {
                parentStringMember: Some(s1),
            },
            stringMember: Some(s2),
        }
    }

    fn MethodThrowToRejectPromise(&self) -> Fallible<Rc<Promise>> {
        Err(Error::Type("test".to_string()))
    }

    fn GetGetterThrowToRejectPromise(&self) -> Fallible<Rc<Promise>> {
        Err(Error::Type("test".to_string()))
    }

    fn MethodInternalThrowToRejectPromise(&self, _arg: u64) -> Rc<Promise> {
        unreachable!("Method should already throw")
    }

    fn StaticThrowToRejectPromise(_: &GlobalScope) -> Fallible<Rc<Promise>> {
        Err(Error::Type("test".to_string()))
    }

    fn StaticInternalThrowToRejectPromise(_: &GlobalScope, _arg: u64) -> Rc<Promise> {
        unreachable!("Method should already throw")
    }

    fn BooleanAttributeStatic(_: &GlobalScope) -> bool {
        false
    }
    fn SetBooleanAttributeStatic(_: &GlobalScope, _: bool) {}
    fn ReceiveVoidStatic(_: &GlobalScope) {}
    fn PrefControlledStaticAttributeDisabled(_: &GlobalScope) -> bool {
        false
    }
    fn PrefControlledStaticAttributeEnabled(_: &GlobalScope) -> bool {
        false
    }
    fn PrefControlledStaticMethodDisabled(_: &GlobalScope) {}
    fn PrefControlledStaticMethodEnabled(_: &GlobalScope) {}
    fn FuncControlledStaticAttributeDisabled(_: &GlobalScope) -> bool {
        false
    }
    fn FuncControlledStaticAttributeEnabled(_: &GlobalScope) -> bool {
        false
    }
    fn FuncControlledStaticMethodDisabled(_: &GlobalScope) {}
    fn FuncControlledStaticMethodEnabled(_: &GlobalScope) {}
}

impl TestBinding {
    pub(crate) fn condition_satisfied(_: SafeJSContext, _: HandleObject) -> bool {
        true
    }
    pub(crate) fn condition_unsatisfied(_: SafeJSContext, _: HandleObject) -> bool {
        false
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TestBindingCallback {
    #[ignore_malloc_size_of = "unclear ownership semantics"]
    promise: TrustedPromise,
    value: DOMString,
}

impl TestBindingCallback {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn invoke(self) {
        self.promise.root().resolve_native(&self.value);
    }
}
