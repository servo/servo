/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use js::context::JSContext;
use js::realm::AutoRealm;

use crate::DomTypes;
use crate::reflector::DomObject;

pub fn enter_auto_realm<'cx, D: DomTypes>(
    cx: &'cx mut JSContext,
    object: &impl DomObject,
) -> AutoRealm<'cx> {
    AutoRealm::new(
        cx,
        NonNull::new(object.reflector().get_jsobject().get()).unwrap(),
    )
}
