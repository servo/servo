/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of `iterable<...>` and `iterable<..., ...>` WebIDL declarations.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{HandleObject, HandleValue, MutableHandleObject};

use crate::codegen::GenericBindings::IterableIteratorBinding::{
    IterableKeyAndValueResult, IterableKeyOrValueResult,
};
use crate::conversions::IDLInterface;
use crate::error::Fallible;
use crate::interfaces::DomHelpers;
use crate::realms::InRealm;
use crate::reflector::{DomGlobalGeneric, DomObjectIteratorWrap, DomObjectWrap, Reflector};
use crate::root::{Dom, DomRoot, Root};
use crate::script_runtime::{CanGc, JSContext};
use crate::trace::{NoTrace, RootedTraceableBox};
use crate::utils::DOMClass;
use crate::{DomTypes, JSTraceable};

/// The values that an iterator will iterate over.
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum IteratorType {
    /// The keys of the iterable object.
    Keys,
    /// The values of the iterable object.
    Values,
    /// The keys and values of the iterable object combined.
    Entries,
}

/// A DOM object that can be iterated over using a pair value iterator.
pub trait Iterable {
    /// The type of the key of the iterator pair.
    type Key: ToJSValConvertible;
    /// The type of the value of the iterator pair.
    type Value: ToJSValConvertible;
    /// Return the number of entries that can be iterated over.
    fn get_iterable_length(&self) -> u32;
    /// Return the value at the provided index.
    fn get_value_at_index(&self, index: u32) -> Self::Value;
    /// Return the key at the provided index.
    fn get_key_at_index(&self, index: u32) -> Self::Key;
}

/// A version of the [IDLInterface] trait that is specific to types that have
/// iterators defined for them. This allows the `script` crate to define the
/// derives check for the concrete interface type, while the [IteratableIterator]
/// type defined in this module can be parameterized over an unknown generic.
pub trait IteratorDerives {
    fn derives(class: &'static DOMClass) -> bool;
}

/// An iterator over the iterable entries of a given DOM interface.
#[dom_struct]
pub struct IterableIterator<
    D: DomTypes,
    T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>,
> {
    reflector: Reflector,
    iterable: Dom<T>,
    type_: IteratorType,
    index: Cell<u32>,
    _marker: NoTrace<PhantomData<D>>,
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable> IterableIterator<D, T> {
    pub fn global_(&self, realm: InRealm) -> DomRoot<D::GlobalScope> {
        <Self as DomGlobalGeneric<D>>::global_(self, realm)
    }
}

impl<
    D: DomTypes,
    T: DomObjectIteratorWrap<D>
        + JSTraceable
        + Iterable
        + DomGlobalGeneric<D>
        + IDLInterface
        + IteratorDerives,
> IDLInterface for IterableIterator<D, T>
{
    fn derives(class: &'static DOMClass) -> bool {
        <T as IteratorDerives>::derives(class)
    }
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>>
    IterableIterator<D, T>
{
    /// Create a new iterator instance for the provided iterable DOM interface.
    pub(crate) fn new(iterable: &T, type_: IteratorType, realm: InRealm) -> DomRoot<Self> {
        let iterator = Box::new(IterableIterator {
            reflector: Reflector::new(),
            type_,
            iterable: Dom::from_ref(iterable),
            index: Cell::new(0),
            _marker: NoTrace(PhantomData),
        });
        <D as DomHelpers<D>>::reflect_dom_object(iterator, &*iterable.global_(realm), CanGc::note())
    }

    /// Return the next value from the iterable object.
    #[allow(non_snake_case)]
    pub fn Next(&self, cx: JSContext) -> Fallible<NonNull<JSObject>> {
        let index = self.index.get();
        rooted!(in(*cx) let mut value = UndefinedValue());
        rooted!(in(*cx) let mut rval = ptr::null_mut::<JSObject>());
        let result = if index >= self.iterable.get_iterable_length() {
            dict_return(cx, rval.handle_mut(), true, value.handle())
        } else {
            match self.type_ {
                IteratorType::Keys => {
                    unsafe {
                        self.iterable
                            .get_key_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    dict_return(cx, rval.handle_mut(), false, value.handle())
                },
                IteratorType::Values => {
                    unsafe {
                        self.iterable
                            .get_value_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    dict_return(cx, rval.handle_mut(), false, value.handle())
                },
                IteratorType::Entries => {
                    rooted!(in(*cx) let mut key = UndefinedValue());
                    unsafe {
                        self.iterable
                            .get_key_at_index(index)
                            .to_jsval(*cx, key.handle_mut());
                        self.iterable
                            .get_value_at_index(index)
                            .to_jsval(*cx, value.handle_mut());
                    }
                    key_and_value_return(cx, rval.handle_mut(), key.handle(), value.handle())
                },
            }
        };
        self.index.set(index + 1);
        result.map(|_| NonNull::new(rval.get()).expect("got a null pointer"))
    }
}

impl<D: DomTypes, T: DomObjectIteratorWrap<D> + JSTraceable + Iterable + DomGlobalGeneric<D>>
    DomObjectWrap<D> for IterableIterator<D, T>
{
    const WRAP: unsafe fn(
        JSContext,
        &D::GlobalScope,
        Option<HandleObject>,
        Box<Self>,
        CanGc,
    ) -> Root<Dom<Self>> = T::ITER_WRAP;
}

fn dict_return(
    cx: JSContext,
    mut result: MutableHandleObject,
    done: bool,
    value: HandleValue,
) -> Fallible<()> {
    let mut dict = IterableKeyOrValueResult::empty();
    dict.done = done;
    dict.value.set(value.get());
    rooted!(in(*cx) let mut dict_value = UndefinedValue());
    unsafe {
        dict.to_jsval(*cx, dict_value.handle_mut());
    }
    result.set(dict_value.to_object());
    Ok(())
}

fn key_and_value_return(
    cx: JSContext,
    mut result: MutableHandleObject,
    key: HandleValue,
    value: HandleValue,
) -> Fallible<()> {
    let mut dict = IterableKeyAndValueResult::empty();
    dict.done = false;
    dict.value = Some(
        vec![key, value]
            .into_iter()
            .map(|handle| RootedTraceableBox::from_box(Heap::boxed(handle.get())))
            .collect(),
    );
    rooted!(in(*cx) let mut dict_value = UndefinedValue());
    unsafe {
        dict.to_jsval(*cx, dict_value.handle_mut());
    }
    result.set(dict_value.to_object());
    Ok(())
}
