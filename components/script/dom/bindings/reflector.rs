/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use js::rust::HandleObject;
use script_bindings::interfaces::GlobalScopeHelpers;

use crate::DomTypes;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub(crate) fn reflect_dom_object<D, T, U>(obj: Box<T>, global: &U, can_gc: CanGc) -> DomRoot<T>
where
    D: DomTypes,
    T: DomObject + DomObjectWrap<D>,
    U: DerivedFrom<D::GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(D::GlobalScope::get_cx(), global_scope, None, obj, can_gc) }
}

pub(crate) fn reflect_dom_object_with_proto<D, T, U>(
    obj: Box<T>,
    global: &U,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> DomRoot<T>
where
    D: DomTypes,
    T: DomObject + DomObjectWrap<D>,
    U: DerivedFrom<D::GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(D::GlobalScope::get_cx(), global_scope, proto, obj, can_gc) }
}

pub(crate) trait DomGlobal {
    fn global_(&self, realm: InRealm) -> DomRoot<GlobalScope>;
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: DomGlobalGeneric<crate::DomTypeHolder>> DomGlobal for T {
    fn global_(&self, realm: InRealm) -> DomRoot<GlobalScope> {
        <Self as DomGlobalGeneric<crate::DomTypeHolder>>::global_(self, realm)
    }
    fn global(&self) -> DomRoot<GlobalScope> {
        <Self as DomGlobalGeneric<crate::DomTypeHolder>>::global(self)
    }
}

pub(crate) use script_bindings::reflector::*;
