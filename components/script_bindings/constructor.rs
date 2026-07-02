/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;
use std::ptr;

use js::gc::{MutableHandle, RootedGuard};
use js::jsapi::{CallArgs, JSClass, JSFunctionSpec, JSObject, JSPropertySpec};
use js::rooted;
use js::rust::HandleObject;
use js::rust::wrappers2::{
    GetRealmErrorPrototype, GetRealmIteratorPrototype, GetRealmObjectPrototype, JS_NewPlainObject,
};

use crate::DomTypes;
use crate::codegen::PrototypeList::{self};
use crate::constant::ConstantSpec;
use crate::error::throw_constructor_without_new;
use crate::guard::Guard;
use crate::interface::{
    create_callback_interface_object, create_interface_prototype_object, get_desired_proto,
};
use crate::js::rust::GCMethods;
use crate::namespace::{NamespaceObjectClass, create_namespace_object};
use crate::utils::ProtoOrIfaceArray;

pub(crate) unsafe fn call_default_constructor<D: crate::DomTypes>(
    cx: &mut js::context::JSContext,
    args: &CallArgs,
    global: &D::GlobalScope,
    proto_id: PrototypeList::ID,
    ctor_name: &str,
    creator: unsafe fn(&mut js::context::JSContext, HandleObject, *mut ProtoOrIfaceArray),
    constructor: impl FnOnce(
        &mut js::context::JSContext,
        &CallArgs,
        &D::GlobalScope,
        HandleObject,
    ) -> bool,
) -> bool {
    if !args.is_constructing() {
        throw_constructor_without_new(cx.into(), ctor_name);
        return false;
    }

    rooted!(&in(cx) let mut desired_proto = ptr::null_mut::<JSObject>());
    let proto_result = get_desired_proto(cx, args, proto_id, creator, desired_proto.handle_mut());
    if proto_result.is_err() {
        return false;
    }

    constructor(cx, args, global, desired_proto.handle())
}

/// SAFETY: cache is a non-null pointer to a valid ProtoOrIfaceArray object.
unsafe fn post_barrier(
    constructor: usize,
    cache: *mut ProtoOrIfaceArray,
    object: RootedGuard<'_, *mut JSObject>,
) -> RootedGuard<'_, *mut JSObject> {
    unsafe {
        assert!((*cache)[constructor].is_null());
        (*cache)[constructor] = object.get();
        <*mut JSObject>::post_barrier(
            (*cache).as_mut_ptr().add(constructor),
            ptr::null_mut(),
            object.get(),
        );
    }
    object
}

pub(crate) struct NamespaceInit {
    pub(crate) is_proto_hack: bool,
    pub(crate) static_methods: &'static [Guard<&'static [JSFunctionSpec]>],
    pub(crate) namespace_object_class: &'static NamespaceObjectClass,
    pub(crate) constructor_name: PrototypeList::Constructor,
    pub(crate) constants: &'static [Guard<&'static [ConstantSpec]>],
    pub(crate) attributes: &'static [Guard<&'static [JSPropertySpec]>],
    pub(crate) name: &'static CStr,
}

pub(crate) struct CallbackInit {
    pub(crate) constants: &'static [Guard<&'static [ConstantSpec]>],
    pub(crate) constructor_name: PrototypeList::Constructor,
    pub(crate) name: &'static CStr,
}

pub(crate) enum InitType {
    RealmErrorPrototype,
    RealmIteratorPrototype,
    RealmObjectPrototype,
    Parent(ParentFn),
}

type HasPropertyFn =
    fn(&mut js::context::JSContext, HandleObject, MutableHandle<'_, *mut JSObject>);

type ParentFn = fn(&mut js::context::JSContext, HandleObject, MutableHandle<'_, *mut JSObject>);

pub(crate) struct InterfaceInit {
    pub(crate) init_type: InitType,
    pub(crate) has_named_properties_object: Option<HasPropertyFn>,
    pub(crate) prototype_class: &'static JSClass,
    pub(crate) methods: &'static [Guard<&'static [JSFunctionSpec]>],
    pub(crate) attrs: &'static [Guard<&'static [JSPropertySpec]>],
    pub(crate) consts: &'static [Guard<&'static [ConstantSpec]>],
    pub(crate) unscopables: &'static [&'static CStr],
    pub(crate) prototype_id: PrototypeList::ID,
}

/// SAFETY: cache is a non-null pointer to a valid ProtoOrIfaceArray object.
pub(crate) unsafe fn create_namespace_interface_objects<D: DomTypes>(
    cx: &mut js::context::JSContext,
    init: NamespaceInit,
    global: HandleObject,
    cache: *mut ProtoOrIfaceArray,
) {
    rooted!(&in(cx) let mut proto: *mut JSObject = std::ptr::null_mut());
    unsafe {
        if init.is_proto_hack {
            proto.set(GetRealmObjectPrototype(cx))
        } else {
            proto.set(JS_NewPlainObject(cx))
        };
    }

    assert!(!proto.is_null());
    rooted!(&in(cx) let mut namespace = ptr::null_mut::<JSObject>());
    create_namespace_object::<D>(
        cx,
        global,
        proto.handle(),
        init.namespace_object_class,
        init.static_methods,
        init.attributes,
        init.constants,
        init.name,
        namespace.handle_mut(),
    );
    assert!(!namespace.is_null());

    unsafe {
        post_barrier(init.constructor_name as usize, cache, namespace);
    }
}

/// SAFETY: cache is a non-null pointer to a valid ProtoOrIfaceArray object.
pub(crate) unsafe fn create_callback_interface_objects<D: DomTypes>(
    cx: &mut js::context::JSContext,
    init: CallbackInit,
    global: HandleObject,
    cache: *mut ProtoOrIfaceArray,
) {
    rooted!(&in(cx) let mut interface = ptr::null_mut::<JSObject>());
    create_callback_interface_object::<D>(
        cx,
        global,
        init.constants,
        init.name,
        interface.handle_mut(),
    );
    unsafe {
        post_barrier(init.constructor_name as usize, cache, interface);
    }
}

/// SAFETY: cache is a non-null pointer to a valid ProtoOrIfaceArray object.
/// The returned object needs to be rooted.
pub(crate) unsafe fn create_interface<D: DomTypes>(
    cx: &mut js::context::JSContext,
    init: InterfaceInit,
    global: HandleObject,
    cache: *mut ProtoOrIfaceArray,
) -> *mut JSObject {
    rooted!(&in(cx) let mut prototype_proto = ptr::null_mut::<JSObject>());
    match init.init_type {
        InitType::RealmErrorPrototype => unsafe {
            prototype_proto.set(GetRealmErrorPrototype(cx));
        },
        InitType::RealmIteratorPrototype => unsafe {
            prototype_proto.set(GetRealmIteratorPrototype(cx));
        },
        InitType::RealmObjectPrototype => unsafe {
            prototype_proto.set(GetRealmObjectPrototype(cx));
        },
        InitType::Parent(f) => f(cx, global, prototype_proto.handle_mut()),
    };
    assert!(!prototype_proto.is_null());

    if let Some(f) = init.has_named_properties_object {
        rooted!(&in(cx) let mut prototype_proto_proto = prototype_proto.get());
        f(
            cx,
            prototype_proto_proto.handle(),
            prototype_proto.handle_mut(),
        );
        assert!(!prototype_proto_proto.is_null());
    }

    rooted!(&in(cx) let mut prototype = ptr::null_mut::<JSObject>());
    create_interface_prototype_object::<D>(
        cx,
        global,
        prototype_proto.handle(),
        init.prototype_class,
        init.methods,
        init.attrs,
        init.consts,
        init.unscopables,
        prototype.handle_mut(),
    );
    assert!(!prototype.is_null());
    unsafe { *post_barrier(init.prototype_id as usize, cache, prototype) }
}
