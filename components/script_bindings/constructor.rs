/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use js::jsapi::{CallArgs, JSObject};
use js::rust::HandleObject;

use crate::codegen::PrototypeList;
use crate::error::throw_constructor_without_new;
use crate::interface::get_desired_proto;
use crate::script_runtime::JSContext;
use crate::utils::ProtoOrIfaceArray;

pub(crate) unsafe fn call_default_constructor<D: crate::DomTypes>(
    cx: JSContext,
    args: &CallArgs,
    global: &D::GlobalScope,
    proto_id: PrototypeList::ID,
    ctor_name: &str,
    creator: unsafe fn(JSContext, HandleObject, *mut ProtoOrIfaceArray),
    constructor: impl FnOnce(JSContext, &CallArgs, &D::GlobalScope, HandleObject) -> bool,
) -> bool {
    if !args.is_constructing() {
        throw_constructor_without_new(cx, ctor_name);
        return false;
    }

    rooted!(in(*cx) let mut desired_proto = ptr::null_mut::<JSObject>());
    let proto_result = get_desired_proto(cx, args, proto_id, creator, desired_proto.handle_mut());
    if proto_result.is_err() {
        return false;
    }

    constructor(cx, args, global, desired_proto.handle())
}
