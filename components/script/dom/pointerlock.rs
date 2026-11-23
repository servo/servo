/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::VecDeque;

use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::ElementBinding::PointerLockOptions;
use script_bindings::dom::MutNullableDom;
use script_bindings::refcounted::Trusted;

use crate::dom::RootedPromise;
use crate::dom::element::Element;

#[derive(Default)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct PointerLockState {
    pub target: MutNullableDom<Element>,
    pub options: Cell<Option<PointerLockOptions>>,
    pub requests: VecDeque<PointerLockRequest>,
}

pub struct PointerLockRequest {
    pub element: Trusted<Element>,
    pub promise: DomRefCell<RootedPromise>,
    pub options: PointerLockOptions,
}
