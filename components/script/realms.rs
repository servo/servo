/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use js::realm::AutoRealm;
use script_bindings::reflector::DomObject;

pub(crate) fn enter_auto_realm<'cx>(
    cx: &'cx mut JSContext,
    object: &impl DomObject,
) -> AutoRealm<'cx> {
    script_bindings::realms::enter_auto_realm::<crate::DomTypeHolder>(cx, object)
}
