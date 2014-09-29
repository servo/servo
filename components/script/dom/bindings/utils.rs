/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::MAX_PROTO_CHAIN_LENGTH;
use dom::bindings::conversions::IDLInterface;
use dom::bindings::error::throw_type_error;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Temporary, Root};
use dom::browsercontext;
use dom::window;
use servo_util::str::DOMString;

use libc;
use libc::c_uint;
use std::cell::Cell;
use std::mem;
use std::cmp::PartialEq;
use std::ptr;
use std::slice;
use js::glue::{js_IsObjectProxyClass, js_IsFunctionProxyClass, IsProxyHandlerFamily};
use js::glue::{UnwrapObject, GetProxyHandlerExtra};
use js::glue::{IsWrapper, RUST_JSID_TO_STRING, RUST_JSID_IS_INT};
use js::glue::{RUST_JSID_IS_STRING, RUST_JSID_TO_INT};
use js::jsapi::{JS_AlreadyHasOwnProperty, JS_NewFunction};
use js::jsapi::{JS_DefineProperties, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_GetClass, JS_LinkConstructorAndPrototype, JS_GetStringCharsAndLength};
use js::jsapi::{JS_ObjectIsRegExp, JS_ObjectIsDate, JSHandleObject};
use js::jsapi::JS_GetFunctionObject;
use js::jsapi::{JS_HasPropertyById, JS_GetPrototype};
use js::jsapi::{JS_GetProperty, JS_HasProperty};
use js::jsapi::{JS_DefineFunctions, JS_DefineProperty};
use js::jsapi::{JS_ValueToString, JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::{JSContext, JSObject, JSBool, jsid, JSClass};
use js::jsapi::{JSFunctionSpec, JSPropertySpec};
use js::jsapi::{JS_NewGlobalObject, JS_InitStandardClasses};
use js::jsapi::{JSString};
use js::jsapi::JS_DeletePropertyById2;
use js::jsfriendapi::JS_ObjectToOuterObject;
use js::jsfriendapi::bindgen::JS_NewObjectWithUniqueType;
use js::jsval::JSVal;
use js::jsval::{PrivateValue, ObjectValue, NullValue};
use js::jsval::{Int32Value, UInt32Value, DoubleValue, BooleanValue, UndefinedValue};
use js::rust::with_compartment;
use js::{JSPROP_ENUMERATE, JSPROP_READONLY, JSPROP_PERMANENT};
use js::JSFUN_CONSTRUCTOR;
use js;

pub struct WindowProxyHandler(pub *const libc::c_void);

#[allow(raw_pointer_deriving)]
#[jstraceable]
pub struct GlobalStaticData {
    pub windowproxy_handler: WindowProxyHandler,
}

pub fn GlobalStaticData() -> GlobalStaticData {
    GlobalStaticData {
        windowproxy_handler: browsercontext::new_window_proxy_handler(),
    }
}

/// Returns whether the given `clasp` is one for a DOM object.
fn is_dom_class(clasp: *const JSClass) -> bool {
    unsafe {
        ((*clasp).flags & js::JSCLASS_IS_DOMJSCLASS) != 0
    }
}

/// Returns whether `obj` is a DOM object implemented as a proxy.
pub fn is_dom_proxy(obj: *mut JSObject) -> bool {
    unsafe {
        (js_IsObjectProxyClass(obj) || js_IsFunctionProxyClass(obj)) &&
            IsProxyHandlerFamily(obj)
    }
}

/// Returns the index of the slot wherein a pointer to the reflected DOM object
/// is stored.
///
/// Fails if `obj` is not a DOM object.
pub unsafe fn dom_object_slot(obj: *mut JSObject) -> u32 {
    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        DOM_OBJECT_SLOT as u32
    } else {
        assert!(is_dom_proxy(obj));
        DOM_PROXY_OBJECT_SLOT as u32
    }
}

/// Get the DOM object from the given reflector.
pub unsafe fn unwrap<T>(obj: *mut JSObject) -> *const T {
    let slot = dom_object_slot(obj);
    let val = JS_GetReservedSlot(obj, slot);
    val.to_private() as *const T
}

/// Get the `DOMClass` from `obj`, or `Err(())` if `obj` is not a DOM object.
pub unsafe fn get_dom_class(obj: *mut JSObject) -> Result<DOMClass, ()> {
    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        debug!("plain old dom object");
        let domjsclass: *const DOMJSClass = clasp as *const DOMJSClass;
        return Ok((*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        debug!("proxy dom object");
        let dom_class: *const DOMClass = GetProxyHandlerExtra(obj) as *const DOMClass;
        return Ok(*dom_class);
    }
    debug!("not a dom object");
    return Err(());
}

/// Get a `JS<T>` for the given DOM object, unwrapping any wrapper around it
/// first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub fn unwrap_jsmanaged<T: Reflectable>(mut obj: *mut JSObject,
                                        proto_id: PrototypeList::id::ID,
                                        proto_depth: uint) -> Result<JS<T>, ()> {
    unsafe {
        let dom_class = get_dom_class(obj).or_else(|_| {
            if IsWrapper(obj) == 1 {
                debug!("found wrapper");
                obj = UnwrapObject(obj, /* stopAtOuter = */ 0, ptr::null_mut());
                if obj.is_null() {
                    debug!("unwrapping security wrapper failed");
                    Err(())
                } else {
                    assert!(IsWrapper(obj) == 0);
                    debug!("unwrapped successfully");
                    get_dom_class(obj)
                }
            } else {
                debug!("not a dom wrapper");
                Err(())
            }
        });

        dom_class.and_then(|dom_class| {
            if dom_class.interface_chain[proto_depth] == proto_id {
                debug!("good prototype");
                Ok(JS::from_raw(unwrap(obj)))
            } else {
                debug!("bad prototype");
                Err(())
            }
        })
    }
}

/// Leak the given pointer.
pub unsafe fn squirrel_away_unique<T>(x: Box<T>) -> *const T {
    mem::transmute(x)
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
pub fn jsstring_to_str(cx: *mut JSContext, s: *mut JSString) -> DOMString {
    unsafe {
        let mut length = 0;
        let chars = JS_GetStringCharsAndLength(cx, s, &mut length);
        slice::raw::buf_as_slice(chars, length as uint, |char_vec| {
            String::from_utf16(char_vec).unwrap()
        })
    }
}

/// Convert the given `jsid` to a `DOMString`. Fails if the `jsid` is not a
/// string, or if the string does not contain valid UTF-16.
pub fn jsid_to_str(cx: *mut JSContext, id: jsid) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id) != 0);
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

/// The index of the slot wherein a pointer to the reflected DOM object is
/// stored for non-proxy bindings.
// We use slot 0 for holding the raw object.  This is safe for both
// globals and non-globals.
pub static DOM_OBJECT_SLOT: uint = 0;
static DOM_PROXY_OBJECT_SLOT: uint = js::JSSLOT_PROXY_PRIVATE as uint;

// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
static DOM_PROTO_INSTANCE_CLASS_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub static DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub static JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

/// Representation of an IDL constant value.
#[deriving(Clone)]
pub enum ConstantVal {
    IntVal(i32),
    UintVal(u32),
    DoubleVal(f64),
    BoolVal(bool),
    NullVal,
    VoidVal
}

/// Representation of an IDL constant.
#[deriving(Clone)]
pub struct ConstantSpec {
    pub name: &'static [u8],
    pub value: ConstantVal
}

impl ConstantSpec {
    /// Returns a `JSVal` that represents the value of this `ConstantSpec`.
    pub fn get_value(&self) -> JSVal {
        match self.value {
            NullVal => NullValue(),
            IntVal(i) => Int32Value(i),
            UintVal(u) => UInt32Value(u),
            DoubleVal(d) => DoubleValue(d),
            BoolVal(b) => BooleanValue(b),
            VoidVal => UndefinedValue(),
        }
    }
}

/// Helper structure for cross-origin wrappers for DOM binding objects.
pub struct NativePropertyHooks {
    /// The property arrays for this interface.
    pub native_properties: &'static NativeProperties,

    /// The NativePropertyHooks instance for the parent interface, if any.
    pub proto_hooks: Option<&'static NativePropertyHooks>,
}

/// The struct that holds inheritance information for DOM object reflectors.
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::id::ID, ..MAX_PROTO_CHAIN_LENGTH],

    /// The NativePropertyHooks for the interface associated with this class.
    pub native_hooks: &'static NativePropertyHooks,
}

/// The JSClass used for DOM object reflectors.
pub struct DOMJSClass {
    pub base: js::Class,
    pub dom_class: DOMClass
}

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
pub fn GetProtoOrIfaceArray(global: *mut JSObject) -> *mut *mut JSObject {
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private() as *mut *mut JSObject
    }
}

/// Contains references to lists of methods, attributes, and constants for a
/// given interface.
pub struct NativeProperties {
    pub methods: Option<&'static [JSFunctionSpec]>,
    pub attrs: Option<&'static [JSPropertySpec]>,
    pub consts: Option<&'static [ConstantSpec]>,
    pub staticMethods: Option<&'static [JSFunctionSpec]>,
    pub staticAttrs: Option<&'static [JSPropertySpec]>,
}

/// A JSNative that cannot be null.
pub type NonNullJSNative =
    unsafe extern "C" fn (arg1: *mut JSContext, arg2: c_uint, arg3: *mut JSVal) -> JSBool;

/// Creates the *interface prototype object* and the *interface object* (if
/// needed).
/// Fails on JSAPI failure.
pub fn CreateInterfaceObjects2(cx: *mut JSContext, global: *mut JSObject, receiver: *mut JSObject,
                               protoProto: *mut JSObject,
                               protoClass: &'static JSClass,
                               constructor: Option<(NonNullJSNative, &'static str, u32)>,
                               domClass: *const DOMClass,
                               members: &'static NativeProperties) -> *mut JSObject {
    let proto = CreateInterfacePrototypeObject(cx, global, protoProto,
                                               protoClass, members);

    unsafe {
        JS_SetReservedSlot(proto, DOM_PROTO_INSTANCE_CLASS_SLOT,
                           PrivateValue(domClass as *const libc::c_void));
    }

    match constructor {
        Some((native, name, nargs)) => {
            let s = name.to_c_str();
            CreateInterfaceObject(cx, global, receiver,
                                  native, nargs, proto,
                                  members, s.as_ptr())
        },
        None => (),
    }

    proto
}

/// Creates the *interface object*.
/// Fails on JSAPI failure.
fn CreateInterfaceObject(cx: *mut JSContext, global: *mut JSObject, receiver: *mut JSObject,
                         constructorNative: NonNullJSNative,
                         ctorNargs: u32, proto: *mut JSObject,
                         members: &'static NativeProperties,
                         name: *const libc::c_char) {
    unsafe {
        let fun = JS_NewFunction(cx, Some(constructorNative), ctorNargs,
                                 JSFUN_CONSTRUCTOR, global, name);
        assert!(fun.is_not_null());

        let constructor = JS_GetFunctionObject(fun);
        assert!(constructor.is_not_null());

        match members.staticMethods {
            Some(staticMethods) => DefineMethods(cx, constructor, staticMethods),
            _ => (),
        }

        match members.staticAttrs {
            Some(staticProperties) => DefineProperties(cx, constructor, staticProperties),
            _ => (),
        }

        match members.consts {
            Some(constants) => DefineConstants(cx, constructor, constants),
            _ => (),
        }

        if proto.is_not_null() {
            assert!(JS_LinkConstructorAndPrototype(cx, constructor, proto) != 0);
        }

        let mut alreadyDefined = 0;
        assert!(JS_AlreadyHasOwnProperty(cx, receiver, name, &mut alreadyDefined) != 0);

        if alreadyDefined == 0 {
            assert!(JS_DefineProperty(cx, receiver, name,
                                      ObjectValue(&*constructor),
                                      None, None, 0) != 0);
        }
    }
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
fn DefineConstants(cx: *mut JSContext, obj: *mut JSObject, constants: &'static [ConstantSpec]) {
    for spec in constants.iter() {
        unsafe {
            assert!(JS_DefineProperty(cx, obj, spec.name.as_ptr() as *const libc::c_char,
                                      spec.get_value(), None, None,
                                      JSPROP_ENUMERATE | JSPROP_READONLY |
                                      JSPROP_PERMANENT) != 0);
        }
    }
}

/// Defines methods on `obj`. The last entry of `methods` must contain zeroed
/// memory.
/// Fails on JSAPI failure.
fn DefineMethods(cx: *mut JSContext, obj: *mut JSObject, methods: &'static [JSFunctionSpec]) {
    unsafe {
        assert!(JS_DefineFunctions(cx, obj, methods.as_ptr()) != 0);
    }
}

/// Defines attributes on `obj`. The last entry of `properties` must contain
/// zeroed memory.
/// Fails on JSAPI failure.
fn DefineProperties(cx: *mut JSContext, obj: *mut JSObject, properties: &'static [JSPropertySpec]) {
    unsafe {
        assert!(JS_DefineProperties(cx, obj, properties.as_ptr()) != 0);
    }
}

/// Creates the *interface prototype object*.
/// Fails on JSAPI failure.
fn CreateInterfacePrototypeObject(cx: *mut JSContext, global: *mut JSObject,
                                  parentProto: *mut JSObject,
                                  protoClass: &'static JSClass,
                                  members: &'static NativeProperties) -> *mut JSObject {
    unsafe {
        let ourProto = JS_NewObjectWithUniqueType(cx, protoClass, &*parentProto, &*global);
        assert!(ourProto.is_not_null());

        match members.methods {
            Some(methods) => DefineMethods(cx, ourProto, methods),
            _ => (),
        }

        match members.attrs {
            Some(properties) => DefineProperties(cx, ourProto, properties),
            _ => (),
        }

        match members.consts {
            Some(constants) => DefineConstants(cx, ourProto, constants),
            _ => (),
        }

        return ourProto;
    }
}

/// A throwing constructor, for those interfaces that have neither
/// `NoInterfaceObject` nor `Constructor`.
pub unsafe extern fn ThrowingConstructor(cx: *mut JSContext, _argc: c_uint, _vp: *mut JSVal) -> JSBool {
    throw_type_error(cx, "Illegal constructor.");
    return 0;
}

/// Construct and cache the ProtoOrIfaceArray for the given global.
/// Fails if the argument is not a DOM global.
pub fn initialize_global(global: *mut JSObject) {
    let protoArray = box () ([0 as *mut JSObject, ..PrototypeList::id::IDCount as uint]);
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        let box_ = squirrel_away_unique(protoArray);
        JS_SetReservedSlot(global,
                           DOM_PROTOTYPE_SLOT,
                           PrivateValue(box_ as *const libc::c_void));
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait Reflectable {
    fn reflector<'a>(&'a self) -> &'a Reflector;
}

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T: Reflectable>
        (obj:     Box<T>,
         global:  &GlobalRef,
         wrap_fn: extern "Rust" fn(*mut JSContext, &GlobalRef, Box<T>) -> Temporary<T>)
         -> Temporary<T> {
    wrap_fn(global.get_cx(), global, obj)
}

/// A struct to store a reference to the reflector of a DOM object.
#[allow(raw_pointer_deriving, unrooted_must_root)]
#[deriving(PartialEq)]
#[must_root]
pub struct Reflector {
    object: Cell<*mut JSObject>,
}

impl Reflector {
    /// Get the reflector.
    #[inline]
    pub fn get_jsobject(&self) -> *mut JSObject {
        self.object.get()
    }

    /// Initialize the reflector. (May be called only once.)
    pub fn set_jsobject(&self, object: *mut JSObject) {
        assert!(self.object.get().is_null());
        assert!(object.is_not_null());
        self.object.set(object);
    }

    /// Return a pointer to the memory location at which the JS reflector object is stored.
    /// Used by Temporary values to root the reflector, as required by the JSAPI rooting
    /// APIs.
    pub fn rootable(&self) -> *mut *mut JSObject {
        &self.object as *const Cell<*mut JSObject>
                     as *mut Cell<*mut JSObject>
                     as *mut *mut JSObject
    }

    /// Create an uninitialized `Reflector`.
    pub fn new() -> Reflector {
        Reflector {
            object: Cell::new(ptr::null_mut()),
        }
    }
}

pub fn GetPropertyOnPrototype(cx: *mut JSContext, proxy: *mut JSObject, id: jsid, found: *mut bool,
                              vp: *mut JSVal) -> bool {
    unsafe {
      //let proto = GetObjectProto(proxy);
      let proto = JS_GetPrototype(proxy);
      if proto.is_null() {
          *found = false;
          return true;
      }
      let mut hasProp = 0;
      if JS_HasPropertyById(cx, proto, id, &mut hasProp) == 0 {
          return false;
      }
      *found = hasProp != 0;
      let no_output = vp.is_null();
      if hasProp == 0 || no_output {
          return true;
      }

      JS_ForwardGetPropertyTo(cx, proto, id, proxy, vp) != 0
  }
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn GetArrayIndexFromId(_cx: *mut JSContext, id: jsid) -> Option<u32> {
    unsafe {
        if RUST_JSID_IS_INT(id) != 0 {
            return Some(RUST_JSID_TO_INT(id) as u32);
        }
        return None;
    }
    // if id is length atom, -1, otherwise
    /*return if JSID_IS_ATOM(id) {
        let atom = JSID_TO_ATOM(id);
        //let s = *GetAtomChars(id);
        if s > 'a' && s < 'z' {
            return -1;
        }

        let i = 0;
        let str = AtomToLinearString(JSID_TO_ATOM(id));
        return if StringIsArray(str, &mut i) != 0 { i } else { -1 }
    } else {
        IdToInt32(cx, id);
    }*/
}

/// Find the index of a string given by `v` in `values`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(None)` if there was no matching string.
pub fn FindEnumStringIndex(cx: *mut JSContext,
                           v: JSVal,
                           values: &[&'static str]) -> Result<Option<uint>, ()> {
    unsafe {
        let jsstr = JS_ValueToString(cx, v);
        if jsstr.is_null() {
            return Err(());
        }

        let mut length = 0;
        let chars = JS_GetStringCharsAndLength(cx, jsstr, &mut length);
        if chars.is_null() {
            return Err(());
        }

        Ok(values.iter().position(|value| {
            value.len() == length as uint &&
            range(0, length as uint).all(|j| {
                value.as_bytes()[j] as u16 == *chars.offset(j as int)
            })
        }))
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(None)` if there was no property with the given name.
pub fn get_dictionary_property(cx: *mut JSContext,
                               object: *mut JSObject,
                               property: &str) -> Result<Option<JSVal>, ()> {
    use std::c_str::CString;
    fn has_property(cx: *mut JSContext, object: *mut JSObject, property: &CString,
                    found: &mut JSBool) -> bool {
        unsafe {
            JS_HasProperty(cx, object, property.as_ptr(), found) != 0
        }
    }
    fn get_property(cx: *mut JSContext, object: *mut JSObject, property: &CString,
                    value: &mut JSVal) -> bool {
        unsafe {
            JS_GetProperty(cx, object, property.as_ptr(), value) != 0
        }
    }

    let property = property.to_c_str();
    if object.is_null() {
        return Ok(None);
    }

    let mut found: JSBool = 0;
    if !has_property(cx, object, &property, &mut found) {
        return Err(());
    }

    if found == 0 {
        return Ok(None);
    }

    let mut value = NullValue();
    if !get_property(cx, object, &property, &mut value) {
        return Err(());
    }

    Ok(Some(value))
}

pub fn HasPropertyOnPrototype(cx: *mut JSContext, proxy: *mut JSObject, id: jsid) -> bool {
    //  MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);
    let mut found = false;
    return !GetPropertyOnPrototype(cx, proxy, id, &mut found, ptr::null_mut()) || found;
}

/// Returns whether `obj` can be converted to a callback interface per IDL.
pub fn IsConvertibleToCallbackInterface(cx: *mut JSContext, obj: *mut JSObject) -> bool {
    unsafe {
        JS_ObjectIsDate(cx, obj) == 0 && JS_ObjectIsRegExp(cx, obj) == 0
    }
}

/// Create a DOM global object with the given class.
pub fn CreateDOMGlobal(cx: *mut JSContext, class: *const JSClass) -> *mut JSObject {
    unsafe {
        let obj = JS_NewGlobalObject(cx, class, ptr::null_mut());
        if obj.is_null() {
            return ptr::null_mut();
        }
        with_compartment(cx, obj, || {
            JS_InitStandardClasses(cx, obj);
        });
        initialize_global(obj);
        obj
    }
}

/// Callback to outerize windows when wrapping.
pub unsafe extern fn wrap_for_same_compartment(cx: *mut JSContext, obj: *mut JSObject) -> *mut JSObject {
    JS_ObjectToOuterObject(cx, obj)
}

/// Callback to outerize windows before wrapping.
pub unsafe extern fn pre_wrap(cx: *mut JSContext, _scope: *mut JSObject,
                       obj: *mut JSObject, _flags: c_uint) -> *mut JSObject {
    JS_ObjectToOuterObject(cx, obj)
}

/// Callback to outerize windows.
pub extern fn outerize_global(_cx: *mut JSContext, obj: JSHandleObject) -> *mut JSObject {
    unsafe {
        debug!("outerizing");
        let obj = *obj.unnamed_field1;
        let win: Root<window::Window> =
            unwrap_jsmanaged(obj,
                             IDLInterface::get_prototype_id(None::<window::Window>),
                             IDLInterface::get_prototype_depth(None::<window::Window>))
            .unwrap()
            .root();
        win.deref().browser_context.borrow().as_ref().unwrap().window_proxy()
    }
}

pub unsafe fn delete_property_by_id(cx: *mut JSContext, object: *mut JSObject,
                                    id: jsid, bp: &mut bool) -> bool {
    let mut value = UndefinedValue();
    if JS_DeletePropertyById2(cx, object, id, &mut value) == 0 {
        return false;
    }

    *bp = value.to_boolean();
    return true;
}

/// Results of `xml_name_type`.
#[deriving(PartialEq)]
pub enum XMLName {
    QName,
    Name,
    InvalidXMLName
}

/// Check if an element name is valid. See http://www.w3.org/TR/xml/#NT-Name
/// for details.
pub fn xml_name_type(name: &str) -> XMLName {
    fn is_valid_start(c: char) -> bool {
        match c {
            ':' |
            'A' .. 'Z' |
            '_' |
            'a' .. 'z' |
            '\xC0' .. '\xD6' |
            '\xD8' .. '\xF6' |
            '\xF8' .. '\u02FF' |
            '\u0370' .. '\u037D' |
            '\u037F' .. '\u1FFF' |
            '\u200C' .. '\u200D' |
            '\u2070' .. '\u218F' |
            '\u2C00' .. '\u2FEF' |
            '\u3001' .. '\uD7FF' |
            '\uF900' .. '\uFDCF' |
            '\uFDF0' .. '\uFFFD' |
            '\U00010000' .. '\U000EFFFF' => true,
            _ => false,
        }
    }

    fn is_valid_continuation(c: char) -> bool {
        is_valid_start(c) || match c {
            '-' |
            '.' |
            '0' .. '9' |
            '\xB7' |
            '\u0300' .. '\u036F' |
            '\u203F' .. '\u2040' => true,
            _ => false,
        }
    }

    let mut iter = name.chars();
    let mut non_qname_colons = false;
    let mut seen_colon = false;
    match iter.next() {
        None => return InvalidXMLName,
        Some(c) => {
            if !is_valid_start(c) {
                return InvalidXMLName;
            }
            if c == ':' {
                non_qname_colons = true;
            }
        }
    }

    for c in name.chars() {
        if !is_valid_continuation(c) {
            return InvalidXMLName;
        }
        if c == ':' {
            match seen_colon {
                true => non_qname_colons = true,
                false => seen_colon = true
            }
        }
    }

    match non_qname_colons {
        false => QName,
        true => Name
    }
}
