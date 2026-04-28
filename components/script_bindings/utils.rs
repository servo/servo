/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::{self, NonNull};
use std::slice;

use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::gc::Handle;
use js::glue::{
    AppendToIdVector, CallJitGetterOp, CallJitMethodOp, CallJitSetterOp, JS_GetReservedSlot,
    RUST_FUNCTION_VALUE_TO_JITINFO,
};
use js::jsapi::{
    AtomToLinearString, CallArgs, ExceptionStackBehavior, GetLinearStringCharAt,
    GetLinearStringLength, GetNonCCWObjectGlobal, HandleId as RawHandleId,
    HandleObject as RawHandleObject, Heap, JS_AtomizeStringN, JS_ClearPendingException,
    JS_DeprecatedStringHasLatin1Chars, JS_GetLatin1StringCharsAndLength, JS_IsExceptionPending,
    JS_IsGlobalObject, JS_MayResolveStandardClass, JS_NewEnumerateStandardClasses,
    JS_ResolveStandardClass, JSAtom, JSAtomState, JSContext, JSJitInfo, JSObject, JSPROP_ENUMERATE,
    JSTracer, MutableHandleIdVector as RawMutableHandleIdVector,
    MutableHandleValue as RawMutableHandleValue, ObjectOpResult, PropertyKey, StringIsArrayIndex,
    jsid,
};
use js::jsid::StringId;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::{
    CallOriginalPromiseReject, JS_DefineProperty, JS_DeletePropertyById, JS_ForwardGetPropertyTo,
    JS_GetPendingException, JS_GetProperty, JS_GetPrototype, JS_HasOwnProperty, JS_HasProperty,
    JS_HasPropertyById, JS_SetPendingException, JS_SetProperty,
};
use js::rust::{
    HandleId, HandleObject, HandleValue, MutableHandleValue, Runtime, ToString, get_object_class,
};
use js::{JS_CALLEE, rooted};
use malloc_size_of::MallocSizeOfOps;

use crate::DomTypes;
use crate::codegen::Globals::Globals;
use crate::codegen::InheritTypes::TopTypeId;
use crate::codegen::PrototypeList::{self, MAX_PROTO_CHAIN_LENGTH, PROTO_OR_IFACE_LENGTH};
use crate::conversions::{PrototypeCheck, private_from_proto_check};
use crate::error::throw_invalid_this;
use crate::interfaces::DomHelpers;
use crate::proxyhandler::{is_cross_origin_object, report_cross_origin_denial};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::str::DOMString;
use crate::trace::trace_object;

/// The struct that holds inheritance information for DOM object reflectors.
#[derive(Clone, Copy)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The last valid index of `interface_chain`.
    pub depth: u8,

    /// The type ID of that interface.
    pub type_id: TopTypeId,

    /// The MallocSizeOf function wrapper for that interface.
    pub malloc_size_of: unsafe fn(ops: &mut MallocSizeOfOps, *const c_void) -> usize,

    /// The `Globals` flag for this global interface, if any.
    pub global: Globals,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
#[repr(C)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::jsapi::JSClass,
    /// Associated data for DOM object reflectors.
    pub dom_class: DOMClass,
}
impl Clone for DOMJSClass {
    fn clone(&self) -> DOMJSClass {
        *self
    }
}
unsafe impl Sync for DOMJSClass {}

/// The index of the slot where the object holder of that interface's
/// unforgeable members are defined.
pub(crate) const DOM_PROTO_UNFORGEABLE_HOLDER_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub(crate) const DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub(crate) const JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
///
/// # Safety
/// `global` must point to a valid, non-null JS object.
pub(crate) unsafe fn get_proto_or_iface_array(global: *mut JSObject) -> *mut ProtoOrIfaceArray {
    assert_ne!(((*get_object_class(global)).flags & JSCLASS_DOM_GLOBAL), 0);
    let mut slot = UndefinedValue();
    JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT, &mut slot);
    slot.to_private() as *mut ProtoOrIfaceArray
}

/// An array of *mut JSObject of size PROTO_OR_IFACE_LENGTH.
pub type ProtoOrIfaceArray = [*mut JSObject; PROTO_OR_IFACE_LENGTH];

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `found` must point to a valid, non-null bool.
pub(crate) unsafe fn get_property_on_prototype(
    cx: *mut JSContext,
    proxy: HandleObject,
    receiver: HandleValue,
    id: HandleId,
    found: *mut bool,
    vp: MutableHandleValue,
) -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut::<JSObject>());
    if !JS_GetPrototype(cx, proxy, proto.handle_mut()) || proto.is_null() {
        *found = false;
        return true;
    }
    let mut has_property = false;
    if !JS_HasPropertyById(cx, proto.handle(), id, &mut has_property) {
        return false;
    }
    *found = has_property;
    if !has_property {
        return true;
    }

    JS_ForwardGetPropertyTo(cx, proto.handle(), id, receiver, vp)
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(id: HandleId) -> Option<u32> {
    let raw_id = *id;
    if raw_id.is_int() {
        return Some(raw_id.to_int() as u32);
    }

    if raw_id.is_void() || !raw_id.is_string() {
        return None;
    }

    unsafe {
        let atom = raw_id.to_string() as *mut JSAtom;
        let s = AtomToLinearString(atom);
        if GetLinearStringLength(s) == 0 {
            return None;
        }

        let chars = [GetLinearStringCharAt(s, 0)];
        let first_char = char::decode_utf16(chars.iter().cloned())
            .next()
            .map_or('\0', |r| r.unwrap_or('\0'));
        if first_char.is_ascii_lowercase() {
            return None;
        }

        let mut i = 0;
        if StringIsArrayIndex(s, &mut i) {
            Some(i)
        } else {
            None
        }
    }

    /*let s = jsstr_to_string(cx, RUST_JSID_TO_STRING(raw_id));
    if s.len() == 0 {
        return None;
    }

    let first = s.chars().next().unwrap();
    if first.is_ascii_lowercase() {
        return None;
    }

    let mut i: u32 = 0;
    let is_array = if s.is_ascii() {
        let chars = s.as_bytes();
        StringIsArrayIndex1(chars.as_ptr() as *const _, chars.len() as u32, &mut i)
    } else {
        let chars = s.encode_utf16().collect::<Vec<u16>>();
        let slice = chars.as_slice();
        StringIsArrayIndex2(slice.as_ptr(), chars.len() as u32, &mut i)
    };

    if is_array {
        Some(i)
    } else {
        None
    }*/
}

/// Find the enum equivelent of a string given by `v` in `pairs`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok((None, value))` if there was no matching string.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
#[allow(clippy::result_unit_err)]
pub(crate) unsafe fn find_enum_value<'a, T>(
    cx: *mut JSContext,
    v: HandleValue,
    pairs: &'a [(&'static str, T)],
) -> Result<(Option<&'a T>, DOMString), ()> {
    match ptr::NonNull::new(ToString(cx, v)) {
        Some(jsstr) => {
            let search = jsstr_to_string(cx, jsstr).into();
            Ok((
                pairs
                    .iter()
                    .find(|&&(key, _)| search == key)
                    .map(|(_, ev)| ev),
                search,
            ))
        },
        None => Err(()),
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(false)` if there was no property with the given name.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
#[allow(clippy::result_unit_err)]
pub unsafe fn get_dictionary_property(
    cx: *mut JSContext,
    object: HandleObject,
    property: &CStr,
    rval: MutableHandleValue,
    _can_gc: CanGc,
) -> Result<bool, ()> {
    unsafe fn has_property(
        cx: *mut JSContext,
        object: HandleObject,
        property: &CStr,
        found: &mut bool,
    ) -> bool {
        JS_HasProperty(cx, object, property.as_ptr(), found)
    }
    unsafe fn get_property(
        cx: *mut JSContext,
        object: HandleObject,
        property: &CStr,
        value: MutableHandleValue,
    ) -> bool {
        JS_GetProperty(cx, object, property.as_ptr(), value)
    }

    if object.get().is_null() {
        return Ok(false);
    }

    let mut found = false;
    if !has_property(cx, object, property, &mut found) {
        return Err(());
    }

    if !found {
        return Ok(false);
    }

    if !get_property(cx, object, property, rval) {
        return Err(());
    }

    Ok(true)
}

/// Set the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure, or null object,
/// and Ok(()) otherwise
#[allow(clippy::result_unit_err)]
pub fn set_dictionary_property(
    cx: SafeJSContext,
    object: HandleObject,
    property: &CStr,
    value: HandleValue,
) -> Result<(), ()> {
    if object.get().is_null() {
        return Err(());
    }

    unsafe {
        if !JS_SetProperty(*cx, object, property.as_ptr(), value) {
            return Err(());
        }
    }

    Ok(())
}

/// Define an own enumerable data property with name `property` on `object`.
/// Returns `Err(())` on JSAPI failure, or null object,
/// and Ok(()) otherwise.
#[allow(clippy::result_unit_err)]
pub fn define_dictionary_property(
    cx: SafeJSContext,
    object: HandleObject,
    property: &CStr,
    value: HandleValue,
) -> Result<(), ()> {
    if object.get().is_null() {
        return Err(());
    }

    unsafe {
        if !JS_DefineProperty(
            *cx,
            object,
            property.as_ptr(),
            value,
            JSPROP_ENUMERATE as u32,
        ) {
            return Err(());
        }
    }

    Ok(())
}

/// Checks whether `object` has an own property named `property`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception),
/// and `Ok(false)` for null objects or when the property is not own.
#[allow(clippy::result_unit_err)]
pub fn has_own_property(
    cx: SafeJSContext,
    object: HandleObject,
    property: &CStr,
) -> Result<bool, ()> {
    if object.get().is_null() {
        return Ok(false);
    }

    let mut found = false;
    unsafe {
        if !JS_HasOwnProperty(*cx, object, property.as_ptr(), &mut found) {
            return Err(());
        }
    }

    Ok(found)
}

/// Computes whether `proxy` has a property `id` on its prototype and stores
/// the result in `found`.
///
/// Returns a boolean indicating whether the check succeeded.
/// If `false` is returned then the value of `found` is unspecified.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub unsafe fn has_property_on_prototype(
    cx: *mut JSContext,
    proxy: HandleObject,
    id: HandleId,
    found: &mut bool,
) -> bool {
    rooted!(in(cx) let mut proto = ptr::null_mut::<JSObject>());
    if !JS_GetPrototype(cx, proxy, proto.handle_mut()) {
        return false;
    }
    assert!(!proto.is_null());
    JS_HasPropertyById(cx, proto.handle(), id, found)
}

/// Deletes the property `id` from `object`.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub(crate) unsafe fn delete_property_by_id(
    cx: *mut JSContext,
    object: HandleObject,
    id: HandleId,
    bp: *mut ObjectOpResult,
) -> bool {
    JS_DeletePropertyById(cx, object, id, bp)
}

pub trait CallPolicy {
    const INFO: CallPolicyInfo;
}
pub mod call_policies {
    use super::*;
    pub struct Normal;
    pub struct TargetClassMaybeCrossOrigin;
    pub struct LenientThis;
    pub struct LenientThisTargetClassMaybeCrossOrigin;
    pub struct CrossOriginCallable;
    impl CallPolicy for Normal {
        const INFO: CallPolicyInfo = CallPolicyInfo {
            lenient_this: false,
            needs_security_check_on_interface_match: false,
        };
    }
    impl CallPolicy for TargetClassMaybeCrossOrigin {
        const INFO: CallPolicyInfo = CallPolicyInfo {
            lenient_this: false,
            needs_security_check_on_interface_match: true,
        };
    }
    impl CallPolicy for LenientThis {
        const INFO: CallPolicyInfo = CallPolicyInfo {
            lenient_this: true,
            needs_security_check_on_interface_match: false,
        };
    }
    impl CallPolicy for LenientThisTargetClassMaybeCrossOrigin {
        const INFO: CallPolicyInfo = CallPolicyInfo {
            lenient_this: true,
            needs_security_check_on_interface_match: true,
        };
    }
    impl CallPolicy for CrossOriginCallable {
        const INFO: CallPolicyInfo = CallPolicyInfo {
            lenient_this: false,
            needs_security_check_on_interface_match: false,
        };
    }
}
/// Controls various details of an IDL operation, such as whether a
/// `[`[`LegacyLenientThis`][1]`]` attribute is specified and preconditions that
/// affect the outcome of the "[perform a security check][2]" steps.
///
/// [1]: https://heycam.github.io/webidl/#LegacyLenientThis
/// [2]: https://html.spec.whatwg.org/multipage/#integration-with-idl
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CallPolicyInfo {
    /// Specifies whether a `[LegacyLenientThis]` attribute is specified on the
    /// interface member this operation is associated with.
    pub lenient_this: bool,
    /// Indicates whether a [security check][1] is required if the target object
    /// implements this operation's interface.
    ///
    /// Regardless of this value, performing a security check is always
    /// necessary if the target object doesn't implement this operation's
    /// interface.
    ///
    /// This field is `false` iff any of the following are true:
    ///
    ///  - The operation is not implemented by any cross-origin objects (i.e.,
    ///    any `Window` or `Location` objects).
    ///
    ///  - The operation is defined as cross origin (i.e., it's included in
    ///    [`CrossOriginProperties`][2]`(obj)`, given `obj` implementing the
    ///    operation's interface).
    ///
    /// [1]: https://html.spec.whatwg.org/multipage/#integration-with-idl
    /// [2]: https://html.spec.whatwg.org/multipage/#crossoriginproperties-(-o-)
    pub needs_security_check_on_interface_match: bool,
}

unsafe fn generic_call<D: DomTypes, const EXCEPTION_TO_REJECTION: bool>(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
    CallPolicyInfo {
        lenient_this,
        needs_security_check_on_interface_match,
    }: CallPolicyInfo,
    call: unsafe extern "C" fn(
        *const JSJitInfo,
        *mut JSContext,
        RawHandleObject,
        *mut libc::c_void,
        u32,
        *mut JSVal,
    ) -> bool,
    can_gc: CanGc,
) -> bool {
    let mut cx = unsafe { js::context::JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let args = CallArgs::from_vp(vp, argc);

    let info = RUST_FUNCTION_VALUE_TO_JITINFO(JS_CALLEE(cx.raw_cx_no_gc(), vp));
    let proto_id = (*info).__bindgen_anon_2.protoID;

    // <https://heycam.github.io/webidl/#es-operations>
    //
    // > To create an operation function, given an operation `op`, a namespace
    // > or interface `target`, and a Realm `realm`:
    // >
    // > 2. Let `steps` be the following series of steps, [...]
    // >
    // > 2.1.2.1. Let `esValue` be the `this` value, if it is not `null` or
    // >          `undefined`, or `realm`’s global object otherwise. [...]
    // >
    // > 2.1.2.2. If `esValue` is a platform object, then perform a security
    // >          check, passing `esValue`, `id`, and "method".
    // >
    // > 2.1.2.3. If `esValue` does not implement the interface `target`, throw
    // >          a `TypeError`. [...]
    //
    // <https://html.spec.whatwg.org/multipage/#integration-with-idl>
    //
    // > When perform a security check is invoked, with a `platformObject`,
    // > `identifier`, and `type`, run these steps:
    // >
    // > 1. If `platformObject` is not a `Window` or `Location` object, then
    // >    return.
    // >
    // > 2. For each `e` of `! CrossOriginProperties(platformObject)`:
    // >
    // > 2.1. If `SameValue(e.[[Property]], identifier)` is true, then:
    // >
    // > 2.1.1. If type is "method" and `e` has neither `[[NeedsGet]]` nor
    // >        `[[NeedsSet]]`, then return. [... ditto for other types]
    // >
    // > 3. If `! IsPlatformObjectSameOrigin(platformObject)` is false, then
    // >    throw a "SecurityError" `DOMException`.
    //
    // According to the above steps, the outcome of an IDL operation is
    // determined be the following boolean variables:
    //
    //  - `this_same_origin`: The current principals object subsumes that of
    //    `thisobj`
    //  - `this_class_cross_origin`: `thisobj`'s class provides a cross-origin
    //    member
    //  - `cross_origin_operation`: The tuple `(operation_name, operation_type)`
    //    (e.g., `("focus", "method")`) is a member of
    //    `CrossOriginProperties(thisobj)`
    //  - `this_implements_operation`: `thisobj`'s class implements the
    //    current operation.
    //
    // The Karnaugh-esque map of the expected outcome is shown below:
    //
    //                            this_same_origin
    //                                ,-------,
    //                            ,---+---+---+---,
    //                            | T | T | o | o |
    //                          ,-+---+---+---+---+
    //                          | | S | T | o | S |
    //  this_class_cross_origin | +---+---+---+---+-,
    //                          | | T | T | o | o | |
    //                          '-+---+---+---+---+ | cross_origin_operation
    //                            | T | T |   |   | |
    //                            '---+---+---+---+-'
    //                                    '-------'
    //                            this_implements_operation
    //
    //       T: TypeError (generated by WebIDL opration function step 2.1.2.3)
    //       S: SecurityError (generated by HTML security check step 3)
    //       o: OK
    //   blank: don't-care (impossible cases)
    //
    // Under some circumstances, we can rule out some cases from this map.
    // E.g., if the operation is known to be not implemented by any cross-origin
    // objects, `this_implements_operation → ¬this_class_cross_origin`, so in
    // this case, we don't have to perform the security check at all if
    // `thisobj`'s class implements the expected interface. `CallPolicyInfo::
    // needs_security_check_on_interface_match` indicates whether this applies.

    let thisobj = args.thisv();
    if !thisobj.get().is_null_or_undefined() && !thisobj.get().is_object() {
        // `thisobj` is not a platform object, so the security check is not
        // invoked in this case
        throw_invalid_this((&mut cx).into(), proto_id);
        return if EXCEPTION_TO_REJECTION {
            exception_to_promise(cx.raw_cx(), args.rval(), can_gc)
        } else {
            false
        };
    }

    rooted!(&in(cx) let obj = if thisobj.get().is_object() {
        thisobj.get().to_object()
    } else {
        GetNonCCWObjectGlobal(JS_CALLEE(cx.raw_cx_no_gc(), vp).to_object_or_null())
    });
    let depth = (*info).__bindgen_anon_3.depth as usize;
    let proto_check = PrototypeCheck::Depth { depth, proto_id };
    let this = match private_from_proto_check(obj.get(), cx.raw_cx_no_gc(), proto_check) {
        Ok(val) => val,
        Err(()) => {
            // [this_implements_operation == false]
            //
            // For now, We don't check the conditions for `SecurityError` in
            // this case, following WebKit's behavior.
            //
            // FIXME: Implement a different browser or the specification's
            //        behavior? (They all differ subtly.) Some behavior is more
            //        challenging to implement - for example, implementing the
            //        specification's behavior requires `generic_call`'s code to
            //        have access to the current IDL operation's name and type
            //        and the target object's `CrossOriginProperties`.
            if lenient_this {
                debug_assert!(!JS_IsExceptionPending(cx.raw_cx_no_gc()));
                *vp = UndefinedValue();
                return true;
            } else {
                throw_invalid_this((&mut cx).into(), proto_id);
                return if EXCEPTION_TO_REJECTION {
                    exception_to_promise(cx.raw_cx(), args.rval(), can_gc)
                } else {
                    false
                };
            }
        },
    };

    // [this_implements_operation == true]

    if needs_security_check_on_interface_match {
        let mut realm = js::realm::CurrentRealm::assert(&mut cx);
        // [cross_origin_operation == false]
        if is_cross_origin_object::<D>((&mut realm).into(), obj.handle().into()) &&
            !<D as DomHelpers<D>>::is_platform_object_same_origin(&realm, obj.handle().into())
        {
            // [this_class_cross_origin == true && this_same_origin == false]
            // Throw a `SecurityError` `DOMException`.
            // FIXME: `Handle<jsid>` could have a default constructor
            //        like `Handle<Value>::null`
            rooted!(&in(*realm) let mut void_jsid: js::jsapi::jsid);
            let result =
                report_cross_origin_denial::<D>(&mut realm, void_jsid.handle().into(), "call");
            return if EXCEPTION_TO_REJECTION {
                exception_to_promise(cx.raw_cx(), args.rval(), can_gc)
            } else {
                result
            };
        }
    } else {
        // [(cross_origin_operation == true && this_class_cross_origin == true)
        //  || cross_origin_operation == false && this_class_cross_origin == false]
    }

    call(
        info,
        cx.raw_cx(),
        obj.handle().into(),
        this as *mut libc::c_void,
        argc,
        vp,
    )
}

/// Generic method of IDL interface.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `vp` must point to a VALID, non-null JSVal.
pub(crate) unsafe extern "C" fn generic_method<
    D: DomTypes,
    Policy: CallPolicy,
    const EXCEPTION_TO_REJECTION: bool,
>(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call::<D, EXCEPTION_TO_REJECTION>(
        cx,
        argc,
        vp,
        Policy::INFO,
        CallJitMethodOp,
        CanGc::deprecated_note(),
    )
}

/// Generic getter of IDL interface.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `vp` must point to a VALID, non-null JSVal.
pub(crate) unsafe extern "C" fn generic_getter<
    D: DomTypes,
    Policy: CallPolicy,
    const EXCEPTION_TO_REJECTION: bool,
>(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call::<D, EXCEPTION_TO_REJECTION>(
        cx,
        argc,
        vp,
        Policy::INFO,
        CallJitGetterOp,
        CanGc::deprecated_note(),
    )
}

unsafe extern "C" fn call_setter(
    info: *const JSJitInfo,
    cx: *mut JSContext,
    handle: RawHandleObject,
    this: *mut libc::c_void,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    if !CallJitSetterOp(info, cx, handle, this, argc, vp) {
        return false;
    }
    *vp = UndefinedValue();
    true
}

/// Generic setter of IDL interface.
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
/// `vp` must point to a VALID, non-null JSVal.
pub(crate) unsafe extern "C" fn generic_setter<D: DomTypes, Policy: CallPolicy>(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    generic_call::<D, false>(
        cx,
        argc,
        vp,
        Policy::INFO,
        call_setter,
        CanGc::deprecated_note(),
    )
}

/// <https://searchfox.org/mozilla-central/rev/7279a1df13a819be254fd4649e07c4ff93e4bd45/dom/bindings/BindingUtils.cpp#3300>
/// # Safety
///
/// `cx` must point to a valid, non-null JSContext.
/// `vp` must point to a VALID, non-null JSVal.
pub(crate) unsafe extern "C" fn generic_static_promise_method(
    cx: *mut JSContext,
    argc: libc::c_uint,
    vp: *mut JSVal,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);

    let info = RUST_FUNCTION_VALUE_TO_JITINFO(JS_CALLEE(cx, vp));
    assert!(!info.is_null());
    // TODO: we need safe wrappers for this in mozjs!
    // assert_eq!((*info)._bitfield_1, JSJitInfo_OpType::StaticMethod as u8)
    let static_fn = (*info).__bindgen_anon_1.staticMethod.unwrap();
    if static_fn(cx, argc, vp) {
        return true;
    }
    exception_to_promise(cx, args.rval(), CanGc::deprecated_note())
}

/// Coverts exception to promise rejection
///
/// <https://searchfox.org/mozilla-central/rev/b220e40ff2ee3d10ce68e07d8a8a577d5558e2a2/dom/bindings/BindingUtils.cpp#3315>
///
/// # Safety
/// `cx` must point to a valid, non-null JSContext.
pub(crate) unsafe fn exception_to_promise(
    cx: *mut JSContext,
    rval: RawMutableHandleValue,
    _can_gc: CanGc,
) -> bool {
    rooted!(in(cx) let mut exception = UndefinedValue());
    if !JS_GetPendingException(cx, exception.handle_mut()) {
        return false;
    }
    JS_ClearPendingException(cx);
    if let Some(promise) = NonNull::new(CallOriginalPromiseReject(cx, exception.handle())) {
        promise.to_jsval(cx, MutableHandleValue::from_raw(rval));
        true
    } else {
        // We just give up.  Put the exception back.
        JS_SetPendingException(cx, exception.handle(), ExceptionStackBehavior::Capture);
        false
    }
}

/// Trace the resources held by reserved slots of a global object
///
/// # Safety
/// `tracer` must point to a valid, non-null JSTracer.
/// `obj` must point to a valid, non-null JSObject.
pub(crate) unsafe fn trace_global(tracer: *mut JSTracer, obj: *mut JSObject) {
    let array = get_proto_or_iface_array(obj);
    for proto in (*array).iter() {
        if !proto.is_null() {
            trace_object(
                tracer,
                "prototype",
                &*(proto as *const *mut JSObject as *const Heap<*mut JSObject>),
            );
        }
    }
}

/// Enumerate lazy properties of a global object.
/// Modeled after <https://github.com/mozilla/gecko-dev/blob/3fd619f47/dom/bindings/BindingUtils.cpp#L2814>
pub(crate) unsafe extern "C" fn enumerate_global(
    cx: *mut JSContext,
    obj: RawHandleObject,
    props: RawMutableHandleIdVector,
    enumerable_only: bool,
) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    JS_NewEnumerateStandardClasses(cx, obj, props, enumerable_only)
}

/// Enumerate lazy properties of a global object that is a Window.
/// <https://github.com/mozilla/gecko-dev/blob/3fd619f47/dom/base/nsGlobalWindowInner.cpp#3297>
pub(crate) unsafe extern "C" fn enumerate_window<D: DomTypes>(
    cx: *mut JSContext,
    obj: RawHandleObject,
    props: RawMutableHandleIdVector,
    enumerable_only: bool,
) -> bool {
    let mut cx = js::context::JSContext::from_ptr(NonNull::new(cx).unwrap());
    if !enumerate_global(cx.raw_cx(), obj, props, enumerable_only) {
        return false;
    }

    if enumerable_only {
        // All WebIDL interface names are defined as non-enumerable, so there's
        // no point in checking them if we're only returning enumerable names.
        return true;
    }

    let obj = Handle::from_raw(obj);
    for (name, interface) in <D as DomHelpers<D>>::interface_map() {
        if !(interface.enabled)(&mut cx, obj) {
            continue;
        }
        let s = JS_AtomizeStringN(cx.raw_cx(), name.as_ptr() as *const c_char, name.len());
        rooted!(&in(cx) let id = StringId(s));
        if s.is_null() || !AppendToIdVector(props, id.handle().into()) {
            return false;
        }
    }
    true
}

/// Returns true if the resolve hook for this global may resolve the provided id.
/// <https://searchfox.org/mozilla-central/rev/f3c8c63a097b61bb1f01e13629b9514e09395947/dom/bindings/BindingUtils.cpp#2809>
/// <https://searchfox.org/mozilla-central/rev/f3c8c63a097b61bb1f01e13629b9514e09395947/js/public/Class.h#283-291>
pub(crate) unsafe extern "C" fn may_resolve_global(
    names: *const JSAtomState,
    id: PropertyKey,
    maybe_obj: *mut JSObject,
) -> bool {
    JS_MayResolveStandardClass(names, id, maybe_obj)
}

/// Returns true if the resolve hook for this window may resolve the provided id.
/// <https://searchfox.org/mozilla-central/rev/f3c8c63a097b61bb1f01e13629b9514e09395947/dom/base/nsGlobalWindowInner.cpp#3275>
/// <https://searchfox.org/mozilla-central/rev/f3c8c63a097b61bb1f01e13629b9514e09395947/js/public/Class.h#283-291>
pub(crate) unsafe extern "C" fn may_resolve_window<D: DomTypes>(
    names: *const JSAtomState,
    id: PropertyKey,
    maybe_obj: *mut JSObject,
) -> bool {
    if may_resolve_global(names, id, maybe_obj) {
        return true;
    }

    let cx = Runtime::get()
        .expect("There must be a JSContext active")
        .as_ptr();
    let Ok(bytes) = latin1_bytes_from_id(cx, id) else {
        return false;
    };

    <D as DomHelpers<D>>::interface_map().contains_key(bytes)
}

/// Resolve a lazy global property, for interface objects and named constructors.
pub(crate) unsafe extern "C" fn resolve_global(
    cx: *mut JSContext,
    obj: RawHandleObject,
    id: RawHandleId,
    rval: *mut bool,
) -> bool {
    assert!(JS_IsGlobalObject(obj.get()));
    JS_ResolveStandardClass(cx, obj, id, rval)
}

/// Resolve a lazy global property for a Window global.
pub(crate) unsafe extern "C" fn resolve_window<D: DomTypes>(
    cx: *mut JSContext,
    obj: RawHandleObject,
    id: RawHandleId,
    rval: *mut bool,
) -> bool {
    let mut cx = js::context::JSContext::from_ptr(NonNull::new(cx).unwrap());
    if !resolve_global(cx.raw_cx(), obj, id, rval) {
        return false;
    }

    if *rval {
        return true;
    }
    let Ok(bytes) = latin1_bytes_from_id(cx.raw_cx(), *id) else {
        *rval = false;
        return true;
    };

    if let Some(interface) = <D as DomHelpers<D>>::interface_map().get(bytes) {
        (interface.define)(&mut cx, Handle::from_raw(obj));
        *rval = true;
    } else {
        *rval = false;
    }
    true
}

/// Returns a slice of bytes corresponding to the bytes in the provided string id.
/// Returns an error if the id is not a string, or the string contains non-latin1 characters.
/// # Safety
/// The slice is only valid as long as the original id is not garbage collected.
unsafe fn latin1_bytes_from_id(cx: *mut JSContext, id: jsid) -> Result<&'static [u8], ()> {
    if !id.is_string() {
        return Err(());
    }

    let string = id.to_string();
    if !JS_DeprecatedStringHasLatin1Chars(string) {
        return Err(());
    }
    let mut length = 0;
    let ptr = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
    assert!(!ptr.is_null());
    Ok(slice::from_raw_parts(ptr, length))
}
