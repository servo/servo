/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ffi::c_void;
use std::ptr;

use js::JSCLASS_IS_GLOBAL;
use js::context::JSContext;
use js::glue::SetProxyReservedSlot;
use js::jsapi::{JS_SetReservedSlot, JSAutoRealm, JSClass, JSObject};
use js::jsval::{PrivateValue, UndefinedValue};
use js::rust::wrappers2::{
    JS_CopyOwnPropertiesAndPrivateFields, JS_InitializePropertiesFromCompatibleNativeObject,
    JS_NewObjectWithGivenProto, JS_WrapObject, NewProxyObject,
};
use js::rust::{Handle, get_context_realm, get_object_class, get_object_realm};

use crate::codegen::PrototypeList;
use crate::conversions::DOM_OBJECT_SLOT;
use crate::import::module::JS_GetReservedSlot;
use crate::proxyhandler::ensure_expando_object;
use crate::root::{DomRoot, MaybeUnreflectedDom, Root};
use crate::utils::DOM_PROTO_UNFORGEABLE_HOLDER_SLOT;
use crate::weakref::DOM_WEAK_SLOT;
use crate::{DomObject, DomTypes, MutDomObject};

type ProtoObjectFn = Box<
    dyn Fn(
        &mut js::context::JSContext,
        js::rust::Handle<*mut JSObject>,
        js::rust::MutableHandle<*mut JSObject>,
    ),
>;

/// TODO: unforgeable is missing
pub(crate) struct WrapConfig {
    pub(crate) is_maybe_cross_origin_object: bool,
    pub(crate) is_proxy: bool,
    pub(crate) weak_referenceable: bool,
    pub(crate) proxy_handler: Option<*const c_void>,
    pub(crate) prototype_id: PrototypeList::ID,
    pub(crate) class: Option<&'static JSClass>,
    // this function has to be more general because we do not have the correct type for globalscope.
    pub(crate) proto_object_fn: ProtoObjectFn,
    pub(crate) is_global: bool,
    pub(crate) has_legacy_unforgeable_members: bool,
}

/// SAFETY:
/// This function returns the first two objects as raw pointers that need to be rooted.
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) unsafe fn wrap<T: MutDomObject, D: DomTypes>(
    cx: &mut JSContext,
    scope: &D::GlobalScope,
    given_proto: Option<js::rust::Handle<*mut JSObject>>,
    object: Box<T>,
    config: WrapConfig,
) -> DomRoot<T> {
    unsafe {
        let raw = Root::new(MaybeUnreflectedDom::from_box(object));

        let scope = scope.reflector().get_jsobject();
        assert!(!scope.get().is_null());
        assert!(((*get_object_class(scope.get())).flags & JSCLASS_IS_GLOBAL) != 0);
        let _ac = JSAutoRealm::new(cx.raw_cx(), scope.get());

        rooted!(&in(cx) let mut canonical_proto = ptr::null_mut::<JSObject>());
        (config.proto_object_fn)(cx, scope, canonical_proto.handle_mut());
        assert!(!canonical_proto.is_null());

        rooted!(&in(cx) let mut obj = ptr::null_mut::<JSObject>());
        if config.is_proxy {
            let handler: *const libc::c_void = config.proxy_handler.unwrap();

            if config.is_maybe_cross_origin_object {
                obj.set(NewProxyObject(
                    cx,
                    handler,
                    Handle::undefined(),
                    ptr::null_mut(),
                    ptr::null(),
                    true,
                ));
            } else {
                obj.set(NewProxyObject(
                    cx,
                    handler,
                    Handle::undefined(),
                    canonical_proto.get(),
                    ptr::null(),
                    false,
                ));
            };

            assert!(!obj.is_null());
            SetProxyReservedSlot(
                obj.get(),
                0,
                &PrivateValue(raw.as_ptr() as *const libc::c_void),
            );
        } else {
            rooted!(&in(cx) let mut proto = ptr::null_mut::<JSObject>());
            if let Some(given) = given_proto {
                proto.set(*given);
                if get_context_realm(cx.raw_cx()) != get_object_realm(*given) {
                    assert!(JS_WrapObject(cx, proto.handle_mut()));
                }
            } else {
                proto.set(*canonical_proto);
            }
            obj.set(JS_NewObjectWithGivenProto(
                cx,
                config.class.unwrap(),
                proto.handle(),
            ));
            assert!(!obj.is_null());
            JS_SetReservedSlot(
                obj.get(),
                DOM_OBJECT_SLOT,
                &PrivateValue(raw.as_ptr() as *const libc::c_void),
            );
        };

        if config.weak_referenceable {
            let val = PrivateValue(ptr::null());
            JS_SetReservedSlot(obj.get(), DOM_WEAK_SLOT, &val);
        }

        let root = raw.reflect_with(obj.get());
        root.reflector().set_proto_id(config.prototype_id as u16);

        // CopyLegacyUnforgeablePropertiesToInstance
        if config.has_legacy_unforgeable_members {
            rooted!(&in(cx) let mut expando = ptr::null_mut::<JSObject>());
            if config.is_proxy {
                ensure_expando_object(cx.raw_cx(), obj.handle().into(), expando.handle_mut());
            }

            let copy_fn = if config.is_global {
                JS_CopyOwnPropertiesAndPrivateFields
            } else {
                JS_InitializePropertiesFromCompatibleNativeObject
            };

            let mut slot = UndefinedValue();
            JS_GetReservedSlot(
                canonical_proto.get(),
                DOM_PROTO_UNFORGEABLE_HOLDER_SLOT,
                &mut slot,
            );
            rooted!(&in(cx) let mut unforgeable_holder = ptr::null_mut::<JSObject>());
            unforgeable_holder.handle_mut().set(slot.to_object());
            if config.is_proxy {
                assert!(copy_fn(cx, expando.handle(), unforgeable_holder.handle()));
            } else {
                assert!(copy_fn(cx, obj.handle(), unforgeable_holder.handle()));
            }
        }

        DomRoot::from_ref(&*root)
    }
}
