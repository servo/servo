/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::JSAutoRealm;
use script_bindings::reflector::DomObject;

pub(crate) use script_bindings::realms::{AlreadyInRealm, InRealm};

pub(crate) fn enter_realm(object: &impl DomObject) -> JSAutoRealm {
    script_bindings::realms::enter_realm::<crate::DomTypeHolder>(object)
}
