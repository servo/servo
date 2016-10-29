/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::validitystate::ValidationFlags;

pub trait Validatable {
    fn is_instance_validatable(&self) -> bool { true }
    fn validate(&self, _validate_flags: ValidationFlags) -> bool { true }
}
