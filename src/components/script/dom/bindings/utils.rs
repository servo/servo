/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::MAX_PROTO_CHAIN_LENGTH;
use dom::bindings::js::JS;
use dom::window;
use servo_util::str::DOMString;

use collections::hashmap::HashMap;
use std::libc::c_uint;
use std::cast;
use std::cmp::Eq;
use std::libc;
use std::ptr;
use std::ptr::null;
use std::str;
use std::vec;
use js::glue::*;
use js::glue::{js_IsObjectProxyClass, js_IsFunctionProxyClass, IsProxyHandlerFamily};
use js::jsapi::{JS_AlreadyHasOwnProperty, JS_NewFunction};
use js::jsapi::{JS_DefineProperties, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_GetClass, JS_LinkConstructorAndPrototype, JS_GetStringCharsAndLength};
use js::jsapi::{JS_ObjectIsRegExp, JS_ObjectIsDate};
use js::jsapi::{JS_InternString, JS_GetFunctionObject};
use js::jsapi::{JS_HasPropertyById, JS_GetPrototype, JS_GetGlobalForObject};
use js::jsapi::{JS_DefineFunctions, JS_DefineProperty};
use js::jsapi::{JS_ValueToString, JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::{JSContext, JSObject, JSBool, jsid, JSClass, JSNative};
use js::jsapi::{JSFunctionSpec, JSPropertySpec, JSPropertyDescriptor};
use js::jsapi::{JS_NewGlobalObject, JS_InitStandardClasses};
use js::jsapi::{JSString};
use js::jsapi::{JS_AllowGC, JS_InhibitGC};
use js::jsfriendapi::bindgen::JS_NewObjectWithUniqueType;
use js::jsval::JSVal;
use js::jsval::{PrivateValue, ObjectValue, NullValue, Int32Value};
use js::jsval::{UInt32Value, DoubleValue, BooleanValue, UndefinedValue};
use js::{JSPROP_ENUMERATE, JSCLASS_IS_GLOBAL, JSCLASS_IS_DOMJSCLASS};
use js::{JSPROP_PERMANENT, JSID_VOID, JSPROP_NATIVE_ACCESSORS, JSPROP_GETTER};
use js::JSPROP_SETTER;
use js::{JSFUN_CONSTRUCTOR, JSPROP_READONLY};
use js;

pub struct GlobalStaticData {
    proxy_handlers: HashMap<uint, *libc::c_void>,
    attribute_ids: HashMap<uint, ~[jsid]>,
    method_ids: HashMap<uint, ~[jsid]>,
    constant_ids: HashMap<uint, ~[jsid]>
}

pub fn GlobalStaticData() -> GlobalStaticData {
    GlobalStaticData {
        proxy_handlers: HashMap::new(),
        attribute_ids: HashMap::new(),
        method_ids: HashMap::new(),
        constant_ids: HashMap::new()
    }
}

fn is_dom_class(clasp: *JSClass) -> bool {
    unsafe {
        ((*clasp).flags & js::JSCLASS_IS_DOMJSCLASS) != 0
    }
}

pub fn is_dom_proxy(obj: *JSObject) -> bool {
    unsafe {
        (js_IsObjectProxyClass(obj) || js_IsFunctionProxyClass(obj)) &&
            IsProxyHandlerFamily(obj)
    }
}

pub unsafe fn dom_object_slot(obj: *JSObject) -> u32 {
    let clasp = JS_GetClass(obj);
    if is_dom_class(clasp) {
        DOM_OBJECT_SLOT as u32
    } else {
        assert!(is_dom_proxy(obj));
        DOM_PROXY_OBJECT_SLOT as u32
    }
}

pub unsafe fn unwrap<T>(obj: *JSObject) -> T {
    let slot = dom_object_slot(obj);
    let val = JS_GetReservedSlot(obj, slot);
    cast::transmute(val.to_private())
}

pub unsafe fn get_dom_class(obj: *JSObject) -> Result<DOMClass, ()> {
    let clasp = JS_GetClass(obj);
    if is_dom_class(clasp) {
        debug!("plain old dom object");
        let domjsclass: *DOMJSClass = cast::transmute(clasp);
        return Ok((*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        debug!("proxy dom object");
        let dom_class: *DOMClass = cast::transmute(GetProxyHandlerExtra(obj));
        return Ok(*dom_class);
    }
    debug!("not a dom object");
    return Err(());
}

pub fn unwrap_object<T>(obj: *JSObject, proto_id: PrototypeList::id::ID, proto_depth: uint) -> Result<T, ()> {
    unsafe {
        get_dom_class(obj).and_then(|dom_class| {
            if dom_class.interface_chain[proto_depth] == proto_id {
                debug!("good prototype");
                Ok(unwrap(obj))
            } else {
                debug!("bad prototype");
                Err(())
            }
        })
    }
}

pub fn unwrap_jsmanaged<T: Reflectable>(obj: *JSObject,
                                        proto_id: PrototypeList::id::ID,
                                        proto_depth: uint) -> Result<JS<T>, ()> {
    let result: Result<*mut T, ()> = unwrap_object(obj, proto_id, proto_depth);
    result.map(|unwrapped| {
        unsafe {
            JS::from_raw(unwrapped)
        }
    })
}

pub fn unwrap_value<T>(val: *JSVal, proto_id: PrototypeList::id::ID, proto_depth: uint) -> Result<T, ()> {
    unsafe {
        let obj = (*val).to_object();
        unwrap_object(obj, proto_id, proto_depth)
    }
}

pub unsafe fn squirrel_away_unboxed<T>(x: ~T) -> *T {
    cast::transmute(x)
}

pub fn jsstring_to_str(cx: *JSContext, s: *JSString) -> DOMString {
    unsafe {
        let length = 0;
        let chars = JS_GetStringCharsAndLength(cx, s, &length);
        vec::raw::buf_as_slice(chars, length as uint, |char_vec| {
            str::from_utf16(char_vec).unwrap()
        })
    }
}

pub fn jsid_to_str(cx: *JSContext, id: jsid) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id) != 0);
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

// We use slot 0 for holding the raw object.  This is safe for both
// globals and non-globals.
pub static DOM_OBJECT_SLOT: uint = 0;
static DOM_PROXY_OBJECT_SLOT: uint = js::JSSLOT_PROXY_PRIVATE as uint;

// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
static DOM_PROTO_INSTANCE_CLASS_SLOT: u32 = 0;

// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub static DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub static JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

pub struct NativeProperties {
    staticMethods: *JSFunctionSpec,
    staticMethodIds: *jsid,
    staticMethodsSpecs: *JSFunctionSpec,
    staticAttributes: *JSPropertySpec,
    staticAttributeIds: *jsid,
    staticAttributeSpecs: *JSPropertySpec,
    methods: *JSFunctionSpec,
    methodIds: *jsid,
    methodsSpecs: *JSFunctionSpec,
    attributes: *JSPropertySpec,
    attributeIds: *jsid,
    attributeSpecs: *JSPropertySpec,
    unforgeableAttributes: *JSPropertySpec,
    unforgeableAttributeIds: *jsid,
    unforgeableAttributeSpecs: *JSPropertySpec,
    constants: *ConstantSpec,
    constantIds: *jsid,
    constantSpecs: *ConstantSpec
}

pub struct NativePropertyHooks {
    resolve_own_property: *u8,
    resolve_property: extern "C" fn(*JSContext, *JSObject, jsid, bool, *mut JSPropertyDescriptor) -> bool,
    enumerate_own_properties: *u8,
    enumerate_properties: *u8,
    proto_hooks: *NativePropertyHooks
}

pub struct JSNativeHolder {
    native: js::jsapi::JSNative,
    propertyHooks: *NativePropertyHooks
}

#[deriving(Clone)]
pub enum ConstantVal {
    IntVal(i32),
    UintVal(u32),
    DoubleVal(f64),
    BoolVal(bool),
    NullVal,
    VoidVal
}

#[deriving(Clone)]
pub struct ConstantSpec {
    name: *libc::c_char,
    value: ConstantVal
}

pub struct DOMClass {
    // A list of interfaces that this object implements, in order of decreasing
    // derivedness.
    interface_chain: [PrototypeList::id::ID, ..MAX_PROTO_CHAIN_LENGTH],

    unused: bool, // DOMObjectIsISupports (always false)
    native_hooks: *NativePropertyHooks
}

pub struct DOMJSClass {
    base: JSClass,
    dom_class: DOMClass
}

pub fn GetProtoOrIfaceArray(global: *JSObject) -> **JSObject {
    unsafe {
        /*assert ((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0;*/
        cast::transmute(JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private())
    }
}

pub fn CreateInterfaceObjects2(cx: *JSContext, global: *JSObject, receiver: *JSObject,
                               protoProto: *JSObject, protoClass: *JSClass,
                               constructor: Option<JSNative>,
                               ctorNargs: u32,
                               domClass: *DOMClass,
                               methods: *JSFunctionSpec,
                               properties: *JSPropertySpec,
                               constants: *ConstantSpec,
                               staticMethods: *JSFunctionSpec,
                               name: &str) -> *JSObject {
    let mut proto = ptr::null();
    if protoClass.is_not_null() {
        proto = CreateInterfacePrototypeObject(cx, global, protoProto,
                                               protoClass, methods,
                                               properties, constants);
        if proto.is_null() {
            return ptr::null();
        }

        unsafe {
            JS_SetReservedSlot(proto, DOM_PROTO_INSTANCE_CLASS_SLOT,
                               PrivateValue(domClass as *libc::c_void));
        }
    }

    let mut interface = ptr::null();
    if constructor.is_some() {
        interface = name.to_c_str().with_ref(|s| {
            CreateInterfaceObject(cx, global, receiver,
                                  constructor, ctorNargs, proto,
                                  staticMethods, constants, s)
        });
        if interface.is_null() {
            return ptr::null();
        }
    }

    if protoClass.is_not_null() {
        proto
    } else {
        interface
    }
}

fn CreateInterfaceObject(cx: *JSContext, global: *JSObject, receiver: *JSObject,
                         constructorNative: Option<JSNative>,
                         ctorNargs: u32, proto: *JSObject,
                         staticMethods: *JSFunctionSpec,
                         constants: *ConstantSpec,
                         name: *libc::c_char) -> *JSObject {
    unsafe {
        let fun = JS_NewFunction(cx, constructorNative, ctorNargs,
                                 JSFUN_CONSTRUCTOR, global, name);
        if fun.is_null() {
            return ptr::null();
        }

        let constructor = JS_GetFunctionObject(fun);
        assert!(constructor.is_not_null());

        if staticMethods.is_not_null() &&
            !DefineMethods(cx, constructor, staticMethods) {
            return ptr::null();
        }

        if constants.is_not_null() &&
            !DefineConstants(cx, constructor, constants) {
            return ptr::null();
        }

        if proto.is_not_null() && JS_LinkConstructorAndPrototype(cx, constructor, proto) == 0 {
            return ptr::null();
        }

        let alreadyDefined = 0;
        if JS_AlreadyHasOwnProperty(cx, receiver, name, &alreadyDefined) == 0 {
            return ptr::null();
        }

        if alreadyDefined == 0 &&
            JS_DefineProperty(cx, receiver, name, ObjectValue(&*constructor),
                              None, None, 0) == 0 {
            return ptr::null();
        }

        return constructor;
    }
}

fn DefineConstants(cx: *JSContext, obj: *JSObject, constants: *ConstantSpec) -> bool {
    let mut i = 0;
    loop {
        unsafe {
            let spec = *constants.offset(i);
            if spec.name.is_null() {
                return true;
            }
            let jsval = match spec.value {
                NullVal => NullValue(),
                IntVal(i) => Int32Value(i),
                UintVal(u) => UInt32Value(u),
                DoubleVal(d) => DoubleValue(d),
                BoolVal(b) => BooleanValue(b),
                VoidVal => UndefinedValue(),
            };
            if JS_DefineProperty(cx, obj, spec.name,
                                 jsval, None,
                                 None,
                                 JSPROP_ENUMERATE | JSPROP_READONLY |
                                 JSPROP_PERMANENT) == 0 {
                return false;
            }
        }
        i += 1;
    }
}

fn DefineMethods(cx: *JSContext, obj: *JSObject, methods: *JSFunctionSpec) -> bool {
    unsafe {
        JS_DefineFunctions(cx, obj, methods) != 0
    }
}

fn DefineProperties(cx: *JSContext, obj: *JSObject, properties: *JSPropertySpec) -> bool {
    unsafe {
        JS_DefineProperties(cx, obj, properties) != 0
    }
}

fn CreateInterfacePrototypeObject(cx: *JSContext, global: *JSObject,
                                  parentProto: *JSObject, protoClass: *JSClass,
                                  methods: *JSFunctionSpec,
                                  properties: *JSPropertySpec,
                                  constants: *ConstantSpec) -> *JSObject {
    unsafe {
        let ourProto = JS_NewObjectWithUniqueType(cx, protoClass, parentProto, global);
        if ourProto.is_null() {
            return ptr::null();
        }

        if methods.is_not_null() && !DefineMethods(cx, ourProto, methods) {
            return ptr::null();
        }

        if properties.is_not_null() && !DefineProperties(cx, ourProto, properties) {
            return ptr::null();
        }

        if constants.is_not_null() && !DefineConstants(cx, ourProto, constants) {
            return ptr::null();
        }

        return ourProto;
    }
}

pub extern fn ThrowingConstructor(_cx: *JSContext, _argc: c_uint, _vp: *mut JSVal) -> JSBool {
    //XXX should trigger exception here
    return 0;
}

pub fn initialize_global(global: *JSObject) {
    let protoArray = ~([0 as *JSObject, ..PrototypeList::id::IDCount as uint]);
    unsafe {
        let box_ = squirrel_away_unboxed(protoArray);
        JS_SetReservedSlot(global,
                           DOM_PROTOTYPE_SLOT,
                           PrivateValue(box_ as *libc::c_void));
    }
}

pub trait Reflectable {
    fn reflector<'a>(&'a self) -> &'a Reflector;
    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector;
}

pub fn reflect_dom_object<T: Reflectable>
        (obj:     ~T,
         window:  &JS<window::Window>,
         wrap_fn: extern "Rust" fn(*JSContext, &JS<window::Window>, ~T) -> *JSObject)
         ->       JS<T> {
    JS::new(obj, window, wrap_fn)
}

#[deriving(Eq)]
pub struct Reflector {
    object: *JSObject,
}

impl Reflector {
    #[inline]
    pub fn get_jsobject(&self) -> *JSObject {
        self.object
    }

    pub fn set_jsobject(&mut self, object: *JSObject) {
        assert!(self.object.is_null());
        assert!(object.is_not_null());
        self.object = object;
    }

    pub fn new() -> Reflector {
        Reflector {
            object: ptr::null(),
        }
    }
}

pub fn GetPropertyOnPrototype(cx: *JSContext, proxy: *JSObject, id: jsid, found: *mut bool,
                              vp: *JSVal) -> bool {
    unsafe {
      //let proto = GetObjectProto(proxy);
      let proto = JS_GetPrototype(proxy);
      if proto.is_null() {
          *found = false;
          return true;
      }
      let hasProp = 0;
      if JS_HasPropertyById(cx, proto, id, &hasProp) == 0 {
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

pub fn GetArrayIndexFromId(_cx: *JSContext, id: jsid) -> Option<u32> {
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

pub fn XrayResolveProperty(cx: *JSContext,
                           wrapper: *JSObject,
                           id: jsid,
                           desc: *mut JSPropertyDescriptor,
                           _methods: Option<~[(JSFunctionSpec, jsid)]>,
                           attributes: Option<~[(JSPropertySpec, jsid)]>,
                           _constants: Option<~[(ConstantSpec, jsid)]>) -> bool
{
  unsafe {
    match attributes {
        Some(attrs) => {
            for &elem in attrs.iter() {
                let (attr, attr_id) = elem;
                if attr_id == JSID_VOID || attr_id != id {
                    continue;
                }

                (*desc).attrs = (attr.flags & !(JSPROP_NATIVE_ACCESSORS as u8)) as u32;
                let global = JS_GetGlobalForObject(cx, wrapper);
                let fun = JS_NewFunction(cx, attr.getter.op, 0, 0, global, ptr::null());
                if fun.is_null() {
                    return false;
                }

                RUST_SET_JITINFO(fun, attr.getter.info);
                let funobj = JS_GetFunctionObject(fun);
                (*desc).getter = Some(cast::transmute(funobj));
                (*desc).attrs |= JSPROP_GETTER;
                if attr.setter.op.is_some() {
                    let fun = JS_NewFunction(cx, attr.setter.op, 1, 0, global, ptr::null());
                    if fun.is_null() {
                        return false
                    }

                    RUST_SET_JITINFO(fun, attr.setter.info);
                    let funobj = JS_GetFunctionObject(fun);
                    (*desc).setter = Some(cast::transmute(funobj));
                    (*desc).attrs |= JSPROP_SETTER;
                } else {
                    (*desc).setter = None;
                }
            }
        }
        None => ()
    }
    return true;
  }
}

fn InternJSString(cx: *JSContext, chars: *libc::c_char) -> Option<jsid> {
    unsafe {
        let s = JS_InternString(cx, chars);
        if s.is_not_null() {
            Some(RUST_INTERNED_STRING_TO_JSID(cx, s))
        } else {
            None
        }
    }
}

pub fn InitIds(cx: *JSContext, specs: &[JSPropertySpec], ids: &mut [jsid]) -> bool {
    for (i, spec) in specs.iter().enumerate() {
        if spec.name.is_null() == true {
            return true;
        }
        match InternJSString(cx, spec.name) {
            Some(id) => ids[i] = id,
            None => {
                return false;
            }
        }
    }
    true
}

pub struct EnumEntry {
    value: &'static str,
    length: uint
}

pub fn FindEnumStringIndex(cx: *JSContext,
                           v: JSVal,
                           values: &[EnumEntry]) -> Result<uint, ()> {
    unsafe {
        let jsstr = JS_ValueToString(cx, v);
        if jsstr.is_null() {
            return Err(());
        }
        let length = 0;
        let chars = JS_GetStringCharsAndLength(cx, jsstr, &length);
        if chars.is_null() {
            return Err(());
        }
        for (i, value) in values.iter().enumerate() {
            if value.length != length as uint {
                continue;
            }
            let mut equal = true;
            for j in range(0, length as int) {
                if value.value[j] as u16 != *chars.offset(j) {
                    equal = false;
                    break;
                }
            };

            if equal {
                return Ok(i);
            }
        }

        return Err(()); //XXX pass in behaviour for value not found
    }
}

pub fn HasPropertyOnPrototype(cx: *JSContext, proxy: *JSObject, id: jsid) -> bool {
    //  MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);
    let mut found = false;
    return !GetPropertyOnPrototype(cx, proxy, id, &mut found, ptr::null()) || found;
}

pub fn IsConvertibleToCallbackInterface(cx: *JSContext, obj: *JSObject) -> bool {
    unsafe {
        JS_ObjectIsDate(cx, obj) == 0 && JS_ObjectIsRegExp(cx, obj) == 0
    }
}

pub fn CreateDOMGlobal(cx: *JSContext, class: *JSClass) -> *JSObject {
    unsafe {
        let obj = JS_NewGlobalObject(cx, class, ptr::null());
        if obj.is_null() {
            return ptr::null();
        }
        JS_InitStandardClasses(cx, obj);
        initialize_global(obj);
        obj
    }
}

/// Returns the global object of the realm that the given JS object was created in.
pub fn global_object_for_js_object(obj: *JSObject) -> JS<window::Window> {
    unsafe {
        let global = GetGlobalForObjectCrossCompartment(obj);
        let clasp = JS_GetClass(global);
        assert!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)) != 0);
        // FIXME(jdm): Either don't hardcode or sanity assert prototype stuff.
        match unwrap_object::<*mut window::Window>(global, PrototypeList::id::Window, 1) {
            Ok(win) => JS::from_raw(win),
            Err(_) => fail!("found DOM global that doesn't unwrap to Window"),
        }
    }
}

fn cx_for_dom_reflector(obj: *JSObject) -> *JSContext {
    let win = global_object_for_js_object(obj);
    let js_info = win.get().page().js_info();
    match *js_info.get() {
        Some(ref info) => info.js_context.borrow().ptr,
        None => fail!("no JS context for DOM global")
    }
}

/// Returns the global object of the realm that the given DOM object was created in.
pub fn global_object_for_dom_object<T: Reflectable>(obj: &T) -> JS<window::Window> {
    global_object_for_js_object(obj.reflector().get_jsobject())
}

pub fn cx_for_dom_object<T: Reflectable>(obj: &T) -> *JSContext {
    cx_for_dom_reflector(obj.reflector().get_jsobject())
}

/// Execute arbitrary code with the JS GC enabled, then disable it afterwards.
pub fn with_gc_enabled<R>(cx: *JSContext, f: || -> R) -> R {
    unsafe {
        JS_AllowGC(cx);
        let rv = f();
        JS_InhibitGC(cx);
        rv
    }
}

/// Execute arbitrary code with the JS GC disabled, then enable it afterwards.
pub fn with_gc_disabled<R>(cx: *JSContext, f: || -> R) -> R {
    unsafe {
        JS_InhibitGC(cx);
        let rv = f();
        JS_AllowGC(cx);
        rv
    }
}

/// Check if an element name is valid. See http://www.w3.org/TR/xml/#NT-Name
/// for details.
#[deriving(Eq)]
pub enum XMLName {
    QName,
    Name,
    InvalidXMLName
}

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
