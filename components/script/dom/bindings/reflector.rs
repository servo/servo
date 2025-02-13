/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use js::rust::HandleObject;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::iterable::{Iterable, IterableIterator};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::{GlobalScope, GlobalScopeHelpers};
use crate::realms::AlreadyInRealm;
use crate::script_runtime::{CanGc, JSContext};
use crate::DomTypes;

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

pub(crate) trait DomGlobalGeneric<D: DomTypes>: DomObject {
    /// Returns the [`GlobalScope`] of the realm that the [`DomObject`] was created in.  If this
    /// object is a `Node`, this will be different from it's owning `Document` if adopted by. For
    /// `Node`s it's almost always better to use `NodeTraits::owning_global`.
    fn global(&self) -> DomRoot<D::GlobalScope>
    where
        Self: Sized,
    {
        let realm = AlreadyInRealm::assert_for_cx(D::GlobalScope::get_cx());
        D::GlobalScope::from_reflector(self, &realm)
    }
}

impl<D: DomTypes, T: DomObject> DomGlobalGeneric<D> for T {}

pub(crate) trait DomGlobal {
    fn global(&self) -> DomRoot<GlobalScope>;
}

impl<T: DomGlobalGeneric<crate::DomTypeHolder>> DomGlobal for T {
    fn global(&self) -> DomRoot<GlobalScope> {
        <Self as DomGlobalGeneric<crate::DomTypeHolder>>::global(self)
    }
}

pub(crate) use script_bindings::reflector::{DomObject, MutDomObject, Reflector};

/// A trait to provide a function pointer to wrap function for DOM objects.
pub(crate) trait DomObjectWrap<D: DomTypes>:
    Sized + DomObject + DomGlobalGeneric<D>
{
    /// Function pointer to the general wrap function type
    #[allow(clippy::type_complexity)]
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub(crate) trait DomObjectIteratorWrap<D: DomTypes>:
    DomObjectWrap<D> + JSTraceable + Iterable
{
    /// Function pointer to the wrap function for `IterableIterator<T>`
    #[allow(clippy::type_complexity)]
    const ITER_WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<IterableIterator<D, Self>>,
        CanGc,
    ) -> Root<Dom<IterableIterator<D, Self>>>;
}
