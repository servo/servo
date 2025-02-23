/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

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
use script_bindings::conversions::IDLInterface;
pub(crate) use script_bindings::iterable::*;
use script_bindings::utils::DOMClass;

use crate::dom::bindings::codegen::Bindings::IterableIteratorBinding::{
    IterableKeyAndValueResult, IterableKeyOrValueResult,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{
    reflect_dom_object, DomGlobalGeneric, DomObjectIteratorWrap, DomObjectWrap, Reflector,
};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::{JSTraceable, NoTrace, RootedTraceableBox};
use crate::script_runtime::{CanGc, JSContext};
use crate::DomTypes;

/// An iterator over the iterable entries of a given DOM interface.
#[dom_struct]
pub(crate) struct IterableIterator<
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
    pub fn global(&self) -> DomRoot<D::GlobalScope> {
        <Self as DomGlobalGeneric<D>>::global(self)
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
    pub(crate) fn new(iterable: &T, type_: IteratorType) -> DomRoot<Self> {
        let iterator = Box::new(IterableIterator {
            reflector: Reflector::new(),
            type_,
            iterable: Dom::from_ref(iterable),
            index: Cell::new(0),
            _marker: NoTrace(PhantomData),
        });
        reflect_dom_object(iterator, &*iterable.global(), CanGc::note())
    }

    /// Return the next value from the iterable object.
    #[allow(non_snake_case)]
    pub(crate) fn Next(&self, cx: JSContext) -> Fallible<NonNull<JSObject>> {
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
