/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Reflector` struct.

use js::jsapi::JSObject;
use js::rust::HandleObject;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::iterable::{Iterable, IterableIterator};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::realms::AlreadyInRealm;
use crate::script_runtime::{CanGc, JSContext};

/// Create the reflector for a new DOM object and yield ownership to the
/// reflector.
pub(crate) fn reflect_dom_object<T, U>(obj: Box<T>, global: &U, can_gc: CanGc) -> DomRoot<T>
where
    T: DomObject + DomObjectWrap,
    U: DerivedFrom<GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(GlobalScope::get_cx(), global_scope, None, obj, can_gc) }
}

pub(crate) fn reflect_dom_object_with_proto<T, U>(
    obj: Box<T>,
    global: &U,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> DomRoot<T>
where
    T: DomObject + DomObjectWrap,
    U: DerivedFrom<GlobalScope>,
{
    let global_scope = global.upcast();
    unsafe { T::WRAP(GlobalScope::get_cx(), global_scope, proto, obj, can_gc) }
}

pub(crate) use script_bindings::reflector::Reflector;

/// A trait to provide access to the `Reflector` for a DOM object.
pub(crate) trait DomObject: JSTraceable + 'static {
    /// Returns the receiver's reflector.
    fn reflector(&self) -> &Reflector;

    /// Returns the [`GlobalScope`] of the realm that the [`DomObject`] was created in.  If this
    /// object is a `Node`, this will be different from it's owning `Document` if adopted by. For
    /// `Node`s it's almost always better to use `NodeTraits::owning_global`.
    fn global(&self) -> DomRoot<GlobalScope>
    where
        Self: Sized,
    {
        let realm = AlreadyInRealm::assert_for_cx(GlobalScope::get_cx());
        GlobalScope::from_reflector(self, &realm)
    }
}

impl DomObject for Reflector {
    fn reflector(&self) -> &Self {
        self
    }
}

/// A trait to initialize the `Reflector` for a DOM object.
pub(crate) trait MutDomObject: DomObject {
    /// Initializes the Reflector
    ///
    /// # Safety
    ///
    /// The provided [`JSObject`] pointer must point to a valid [`JSObject`].
    unsafe fn init_reflector(&self, obj: *mut JSObject);
}

impl MutDomObject for Reflector {
    unsafe fn init_reflector(&self, obj: *mut JSObject) {
        self.set_jsobject(obj)
    }
}

/// A trait to provide a function pointer to wrap function for DOM objects.
pub(crate) trait DomObjectWrap: Sized + DomObject {
    /// Function pointer to the general wrap function type
    #[allow(clippy::type_complexity)]
    const WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>>;
}

/// A trait to provide a function pointer to wrap function for
/// DOM iterator interfaces.
pub(crate) trait DomObjectIteratorWrap: DomObjectWrap + JSTraceable + Iterable {
    /// Function pointer to the wrap function for `IterableIterator<T>`
    #[allow(clippy::type_complexity)]
    const ITER_WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Option<HandleObject>,
        Box<IterableIterator<Self>>,
        CanGc,
    ) -> Root<Dom<IterableIterator<Self>>>;
}
