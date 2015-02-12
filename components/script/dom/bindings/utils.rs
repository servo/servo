/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various utilities to glue JavaScript and the DOM implementation together.

use dom::bindings::codegen::PrototypeList;
use dom::bindings::codegen::PrototypeList::MAX_PROTO_CHAIN_LENGTH;
use dom::bindings::conversions::{unwrap_jsmanaged, is_dom_class};
use dom::bindings::error::throw_type_error;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, Root};
use dom::browsercontext;
use dom::window;

use libc;
use libc::c_uint;
use std::boxed;
use std::cell::Cell;
use std::ffi::CString;
use std::ptr;
use js::glue::UnwrapObject;
use js::glue::{IsWrapper, RUST_JSID_IS_INT, RUST_JSID_TO_INT};
use js::jsapi::{JS_AlreadyHasOwnProperty, JS_NewFunction};
use js::jsapi::{JS_DefineProperties, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_GetClass, JS_LinkConstructorAndPrototype, JS_GetStringCharsAndLength};
use js::jsapi::JSHandleObject;
use js::jsapi::JS_GetFunctionObject;
use js::jsapi::{JS_HasPropertyById, JS_GetPrototype};
use js::jsapi::{JS_GetProperty, JS_HasProperty};
use js::jsapi::{JS_DefineFunctions, JS_DefineProperty};
use js::jsapi::{JS_ValueToString, JS_GetReservedSlot, JS_SetReservedSlot};
use js::jsapi::{JSContext, JSObject, JSBool, jsid, JSClass};
use js::jsapi::{JSFunctionSpec, JSPropertySpec};
use js::jsapi::{JS_NewGlobalObject, JS_InitStandardClasses};
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

/// Proxy handler for a WindowProxy.
pub struct WindowProxyHandler(pub *const libc::c_void);

#[allow(raw_pointer_derive)]
#[jstraceable]
/// Static data associated with a global object.
pub struct GlobalStaticData {
    /// The WindowProxy proxy handler for this global.
    pub windowproxy_handler: WindowProxyHandler,
}

impl GlobalStaticData {
    /// Creates a new GlobalStaticData.
    pub fn new() -> GlobalStaticData {
        GlobalStaticData {
            windowproxy_handler: browsercontext::new_window_proxy_handler(),
        }
    }
}

// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
const DOM_PROTO_INSTANCE_CLASS_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub const DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub const JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

/// Representation of an IDL constant value.
#[derive(Clone)]
pub enum ConstantVal {
    /// `long` constant.
    IntVal(i32),
    /// `unsigned long` constant.
    UintVal(u32),
    /// `double` constant.
    DoubleVal(f64),
    /// `boolean` constant.
    BoolVal(bool),
    /// `null` constant.
    NullVal,
}

/// Representation of an IDL constant.
#[derive(Clone)]
pub struct ConstantSpec {
    /// name of the constant.
    pub name: &'static [u8],
    /// value of the constant.
    pub value: ConstantVal
}

impl ConstantSpec {
    /// Returns a `JSVal` that represents the value of this `ConstantSpec`.
    pub fn get_value(&self) -> JSVal {
        match self.value {
            ConstantVal::NullVal => NullValue(),
            ConstantVal::IntVal(i) => Int32Value(i),
            ConstantVal::UintVal(u) => UInt32Value(u),
            ConstantVal::DoubleVal(d) => DoubleValue(d),
            ConstantVal::BoolVal(b) => BooleanValue(b),
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
#[derive(Copy)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The NativePropertyHooks for the interface associated with this class.
    pub native_hooks: &'static NativePropertyHooks,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::Class,
    /// Associated data for DOM object reflectors.
    pub dom_class: DOMClass
}
unsafe impl Sync for DOMJSClass {}

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
pub fn get_proto_or_iface_array(global: *mut JSObject) -> *mut *mut JSObject {
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT).to_private() as *mut *mut JSObject
    }
}

/// Contains references to lists of methods, attributes, and constants for a
/// given interface.
pub struct NativeProperties {
    /// Instance methods for the interface.
    pub methods: Option<&'static [JSFunctionSpec]>,
    /// Instance attributes for the interface.
    pub attrs: Option<&'static [JSPropertySpec]>,
    /// Constants for the interface.
    pub consts: Option<&'static [ConstantSpec]>,
    /// Static methods for the interface.
    pub static_methods: Option<&'static [JSFunctionSpec]>,
    /// Static attributes for the interface.
    pub static_attrs: Option<&'static [JSPropertySpec]>,
}
unsafe impl Sync for NativeProperties {}

/// A JSNative that cannot be null.
pub type NonNullJSNative =
    unsafe extern "C" fn (arg1: *mut JSContext, arg2: c_uint, arg3: *mut JSVal) -> JSBool;

/// Creates the *interface prototype object* and the *interface object* (if
/// needed).
/// Fails on JSAPI failure.
pub fn do_create_interface_objects(cx: *mut JSContext, global: *mut JSObject,
                                   receiver: *mut JSObject,
                                   proto_proto: *mut JSObject,
                                   proto_class: &'static JSClass,
                                   constructor: Option<(NonNullJSNative, &'static str, u32)>,
                                   dom_class: *const DOMClass,
                                   members: &'static NativeProperties)
                                   -> *mut JSObject {
    let proto = create_interface_prototype_object(cx, global, proto_proto,
                                                  proto_class, members);

    unsafe {
        JS_SetReservedSlot(proto, DOM_PROTO_INSTANCE_CLASS_SLOT,
                           PrivateValue(dom_class as *const libc::c_void));
    }

    match constructor {
        Some((native, name, nargs)) => {
            let s = CString::from_slice(name.as_bytes());
            create_interface_object(cx, global, receiver,
                                    native, nargs, proto,
                                    members, s.as_ptr())
        },
        None => (),
    }

    proto
}

/// Creates the *interface object*.
/// Fails on JSAPI failure.
fn create_interface_object(cx: *mut JSContext, global: *mut JSObject,
                           receiver: *mut JSObject,
                           constructor_native: NonNullJSNative,
                           ctor_nargs: u32, proto: *mut JSObject,
                           members: &'static NativeProperties,
                           name: *const libc::c_char) {
    unsafe {
        let fun = JS_NewFunction(cx, Some(constructor_native), ctor_nargs,
                                 JSFUN_CONSTRUCTOR, global, name);
        assert!(!fun.is_null());

        let constructor = JS_GetFunctionObject(fun);
        assert!(!constructor.is_null());

        match members.static_methods {
            Some(static_methods) => {
                define_methods(cx, constructor, static_methods)
            },
            _ => (),
        }

        match members.static_attrs {
            Some(static_properties) => {
                define_properties(cx, constructor, static_properties)
            },
            _ => (),
        }

        match members.consts {
            Some(constants) => define_constants(cx, constructor, constants),
            _ => (),
        }

        if !proto.is_null() {
            assert!(JS_LinkConstructorAndPrototype(cx, constructor, proto) != 0);
        }

        let mut already_defined = 0;
        assert!(JS_AlreadyHasOwnProperty(cx, receiver, name, &mut already_defined) != 0);

        if already_defined == 0 {
            assert!(JS_DefineProperty(cx, receiver, name,
                                      ObjectValue(&*constructor),
                                      None, None, 0) != 0);
        }
    }
}

/// Defines constants on `obj`.
/// Fails on JSAPI failure.
fn define_constants(cx: *mut JSContext, obj: *mut JSObject,
                    constants: &'static [ConstantSpec]) {
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
fn define_methods(cx: *mut JSContext, obj: *mut JSObject,
                  methods: &'static [JSFunctionSpec]) {
    unsafe {
        assert!(JS_DefineFunctions(cx, obj, methods.as_ptr()) != 0);
    }
}

/// Defines attributes on `obj`. The last entry of `properties` must contain
/// zeroed memory.
/// Fails on JSAPI failure.
fn define_properties(cx: *mut JSContext, obj: *mut JSObject,
                     properties: &'static [JSPropertySpec]) {
    unsafe {
        assert!(JS_DefineProperties(cx, obj, properties.as_ptr()) != 0);
    }
}

/// Creates the *interface prototype object*.
/// Fails on JSAPI failure.
fn create_interface_prototype_object(cx: *mut JSContext, global: *mut JSObject,
                                     parent_proto: *mut JSObject,
                                     proto_class: &'static JSClass,
                                     members: &'static NativeProperties)
                                     -> *mut JSObject {
    unsafe {
        let our_proto = JS_NewObjectWithUniqueType(cx, proto_class,
                                                   &*parent_proto, &*global);
        assert!(!our_proto.is_null());

        match members.methods {
            Some(methods) => define_methods(cx, our_proto, methods),
            _ => (),
        }

        match members.attrs {
            Some(properties) => define_properties(cx, our_proto, properties),
            _ => (),
        }

        match members.consts {
            Some(constants) => define_constants(cx, our_proto, constants),
            _ => (),
        }

        return our_proto;
    }
}

/// A throwing constructor, for those interfaces that have neither
/// `NoInterfaceObject` nor `Constructor`.
pub unsafe extern fn throwing_constructor(cx: *mut JSContext, _argc: c_uint,
                                          _vp: *mut JSVal) -> JSBool {
    throw_type_error(cx, "Illegal constructor.");
    return 0;
}

/// Construct and cache the ProtoOrIfaceArray for the given global.
/// Fails if the argument is not a DOM global.
pub fn initialize_global(global: *mut JSObject) {
    let proto_array = box ()
        ([0 as *mut JSObject; PrototypeList::ID::Count as uint]);
    unsafe {
        assert!(((*JS_GetClass(global)).flags & JSCLASS_DOM_GLOBAL) != 0);
        let box_ = boxed::into_raw(proto_array);
        JS_SetReservedSlot(global,
                           DOM_PROTOTYPE_SLOT,
                           PrivateValue(box_ as *const libc::c_void));
    }
}

/// A trait to provide access to the `Reflector` for a DOM object.
pub trait Reflectable {
    /// Returns the receiver's reflector.
    fn reflector<'a>(&'a self) -> &'a Reflector;
}

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub fn reflect_dom_object<T: Reflectable>
        (obj:     Box<T>,
         global:  GlobalRef,
         wrap_fn: extern "Rust" fn(*mut JSContext, GlobalRef, Box<T>) -> Temporary<T>)
         -> Temporary<T> {
    wrap_fn(global.get_cx(), global, obj)
}

/// A struct to store a reference to the reflector of a DOM object.
// Allowing unused_attribute because the lint sometimes doesn't run in order
#[allow(raw_pointer_derive, unrooted_must_root, unused_attributes)]
#[derive(PartialEq)]
#[must_root]
#[servo_lang = "reflector"]
// If you're renaming or moving this field, update the path in plugins::reflector as well
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
        assert!(!object.is_null());
        self.object.set(object);
    }

    /// Return a pointer to the memory location at which the JS reflector
    /// object is stored. Used by Temporary values to root the reflector, as
    /// required by the JSAPI rooting APIs.
    pub unsafe fn rootable(&self) -> *mut *mut JSObject {
        self.object.as_unsafe_cell().get()
    }

    /// Create an uninitialized `Reflector`.
    pub fn new() -> Reflector {
        Reflector {
            object: Cell::new(ptr::null_mut()),
        }
    }
}

/// Gets the property `id` on  `proxy`'s prototype. If it exists, `*found` is
/// set to true and `*vp` to the value, otherwise `*found` is set to false.
///
/// Returns false on JSAPI failure.
pub fn get_property_on_prototype(cx: *mut JSContext, proxy: *mut JSObject,
                                 id: jsid, found: *mut bool, vp: *mut JSVal)
                                 -> bool {
    unsafe {
      //let proto = GetObjectProto(proxy);
      let proto = JS_GetPrototype(proxy);
      if proto.is_null() {
          *found = false;
          return true;
      }
      let mut has_property = 0;
      if JS_HasPropertyById(cx, proto, id, &mut has_property) == 0 {
          return false;
      }
      *found = has_property != 0;
      let no_output = vp.is_null();
      if has_property == 0 || no_output {
          return true;
      }

      JS_ForwardGetPropertyTo(cx, proto, id, proxy, vp) != 0
  }
}

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(_cx: *mut JSContext, id: jsid) -> Option<u32> {
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
pub fn find_enum_string_index(cx: *mut JSContext,
                              v: JSVal,
                              values: &[&'static str])
                              -> Result<Option<uint>, ()> {
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

/// Returns wether `obj` is a platform object
/// http://heycam.github.io/webidl/#dfn-platform-object
pub fn is_platform_object(obj: *mut JSObject) -> bool {
    unsafe {
        // Fast-path the common case
        let mut clasp = JS_GetClass(obj);
        if is_dom_class(&*clasp) {
            return true;
        }
        // Now for simplicity check for security wrappers before anything else
        if IsWrapper(obj) == 1 {
            let unwrapped_obj = UnwrapObject(obj, /* stopAtOuter = */ 0, ptr::null_mut());
            if unwrapped_obj.is_null() {
                return false;
            }
            clasp = js::jsapi::JS_GetClass(obj);
        }
        // TODO also check if JS_IsArrayBufferObject
        return is_dom_class(&*clasp);
    }
}

/// Get the property with name `property` from `object`.
/// Returns `Err(())` on JSAPI failure (there is a pending exception), and
/// `Ok(None)` if there was no property with the given name.
pub fn get_dictionary_property(cx: *mut JSContext,
                               object: *mut JSObject,
                               property: &str) -> Result<Option<JSVal>, ()> {
    use std::ffi::CString;
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

    let property = CString::from_slice(property.as_bytes());
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

/// Returns whether `proxy` has a property `id` on its prototype.
pub fn has_property_on_prototype(cx: *mut JSContext, proxy: *mut JSObject,
                                 id: jsid) -> bool {
    //  MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);
    let mut found = false;
    return !get_property_on_prototype(cx, proxy, id, &mut found, ptr::null_mut()) || found;
}

/// Create a DOM global object with the given class.
pub fn create_dom_global(cx: *mut JSContext, class: *const JSClass)
                         -> *mut JSObject {
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
        let win: Root<window::Window> = unwrap_jsmanaged(obj).unwrap().root();
        win.r().browser_context().as_ref().unwrap().window_proxy()
    }
}

/// Deletes the property `id` from `object`.
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
#[derive(PartialEq)]
#[allow(missing_docs)]
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
            'A' ... 'Z' |
            '_' |
            'a' ... 'z' |
            '\u{C0}' ... '\u{D6}' |
            '\u{D8}' ... '\u{F6}' |
            '\u{F8}' ... '\u{2FF}' |
            '\u{370}' ... '\u{37D}' |
            '\u{37F}' ... '\u{1FFF}' |
            '\u{200C}' ... '\u{200D}' |
            '\u{2070}' ... '\u{218F}' |
            '\u{2C00}' ... '\u{2FEF}' |
            '\u{3001}' ... '\u{D7FF}' |
            '\u{F900}' ... '\u{FDCF}' |
            '\u{FDF0}' ... '\u{FFFD}' |
            '\u{10000}' ... '\u{EFFFF}' => true,
            _ => false,
        }
    }

    fn is_valid_continuation(c: char) -> bool {
        is_valid_start(c) || match c {
            '-' |
            '.' |
            '0' ... '9' |
            '\u{B7}' |
            '\u{300}' ... '\u{36F}' |
            '\u{203F}' ... '\u{2040}' => true,
            _ => false,
        }
    }

    let mut iter = name.chars();
    let mut non_qname_colons = false;
    let mut seen_colon = false;
    match iter.next() {
        None => return XMLName::InvalidXMLName,
        Some(c) => {
            if !is_valid_start(c) {
                return XMLName::InvalidXMLName;
            }
            if c == ':' {
                non_qname_colons = true;
            }
        }
    }

    for c in name.chars() {
        if !is_valid_continuation(c) {
            return XMLName::InvalidXMLName;
        }
        if c == ':' {
            match seen_colon {
                true => non_qname_colons = true,
                false => seen_colon = true
            }
        }
    }

    match non_qname_colons {
        false => XMLName::QName,
        true => XMLName::Name
    }
}
