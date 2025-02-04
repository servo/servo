/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use script_bindings::codegen::InheritTypes::*;
pub(crate) use script_bindings::inheritance::Castable;

#[allow(missing_docs)]
pub(crate) trait HasParent {
    #[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
    type Parent;
    fn as_parent(&self) -> &Self::Parent;
}
