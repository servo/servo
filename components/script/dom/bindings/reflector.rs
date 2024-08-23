/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use std::default::Default;

use js::jsapi::{Heap, JSObject};
use js::rust::HandleObject;
pub use script_bindings::reflector::*;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::iterable::{Iterable, IterableIterator};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::realms::AlreadyInRealm;
use crate::script_runtime::{CanGc, JSContext};

pub trait DomGlobal {
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: script_bindings::DomGlobal<crate::DomTypeHolder>> DomGlobal for T {
    fn global(&self) -> DomRoot<GlobalScope> {
        <Self as script_bindings::DomGlobal<crate::DomTypeHolder>>::global(self)
    }
}
