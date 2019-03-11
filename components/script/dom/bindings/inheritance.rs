/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Castable` trait.

pub use crate::dom::bindings::codegen::InheritTypes::*;

use crate::dom::bindings::conversions::get_dom_class;
use crate::dom::bindings::conversions::{DerivedFrom, IDLInterface};
use crate::dom::bindings::reflector::DomObject;
use inert::{Inert, NeutralizeUnsafe};
use std::mem;

/// A trait to hold the cast functions of IDL interfaces that either derive
/// or are derived from other interfaces.
pub trait Castable: IDLInterface + DomObject + Sized {
    /// Check whether a DOM object implements one of its deriving interfaces.
    fn is<T>(&self) -> bool
    where
        T: DerivedFrom<Self>,
    {
        let class = unsafe { get_dom_class(self.reflector().get_jsobject().get()).unwrap() };
        T::derives(class)
    }

    /// Cast a DOM object upwards to one of the interfaces it derives from.
    fn upcast<T>(&self) -> &T
    where
        T: Castable,
        Self: DerivedFrom<T>,
    {
        unsafe { mem::transmute(self) }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    fn downcast<T>(&self) -> Option<&T>
    where
        T: DerivedFrom<Self>,
    {
        if self.is::<T>() {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }
}

pub trait CastableInert<T>
where
    T: Castable,
{
    fn is<U>(&self) -> bool
    where
        U: DerivedFrom<T>;

    fn upcast<U>(&self) -> &Inert<U>
    where
        T: DerivedFrom<U>,
        U: Castable + NeutralizeUnsafe;

    fn downcast<U>(&self) -> Option<&Inert<U>>
    where
        U: DerivedFrom<T> + NeutralizeUnsafe;
}

impl<T> CastableInert<T> for Inert<T>
where
    T: Castable + NeutralizeUnsafe,
{
    #[inline]
    fn is<U>(&self) -> bool
    where
        U: DerivedFrom<T>,
    {
        unsafe { Inert::get_ref_unchecked(self).is::<U>() }
    }

    #[inline]
    fn upcast<U>(&self) -> &Inert<U>
    where
        T: DerivedFrom<U>,
        U: Castable + NeutralizeUnsafe,
    {
        unsafe { Inert::new_unchecked(Inert::get_ref_unchecked(self).upcast()) }
    }

    #[inline]
    fn downcast<U>(&self) -> Option<&Inert<U>>
    where
        U: DerivedFrom<T> + NeutralizeUnsafe,
    {
        unsafe {
            Inert::get_ref_unchecked(self)
                .downcast()
                .map(|v| Inert::new_unchecked(v))
        }
    }
}

pub trait HasParent {
    type Parent;
    fn as_parent(&self) -> &Self::Parent;
}
