/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Implementation of `iterable<...>` and `iterable<..., ...>` WebIDL declarations.

use std::cell::Cell;
use std::ptr;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{HandleObject, HandleValue, MutableHandleObject};

use crate::dom::bindings::codegen::Bindings::IterableIteratorBinding::{
    IterableKeyAndValueResult, IterableKeyOrValueResult,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{
    reflect_dom_object, DomObjectIteratorWrap, DomObjectWrap, Reflector,
};
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

/// The values that an iterator will iterate over.
#[derive(JSTraceable, MallocSizeOf)]
pub enum IteratorType {
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

/// An iterator over the iterable entries of a given DOM interface.
//FIXME: #12811 prevents dom_struct with type parameters
#[dom_struct]
pub struct IterableIterator<T: DomObjectIteratorWrap + JSTraceable + Iterable> {
    reflector: Reflector,
    iterable: Dom<T>,
    type_: IteratorType,
    index: Cell<u32>,
}

impl<T: DomObjectIteratorWrap + JSTraceable + Iterable> IterableIterator<T> {
    /// Create a new iterator instance for the provided iterable DOM interface.
    pub fn new(iterable: &T, type_: IteratorType) -> DomRoot<Self> {
        let iterator = Box::new(IterableIterator {
            reflector: Reflector::new(),
            type_,
            iterable: Dom::from_ref(iterable),
            index: Cell::new(0),
        });
        reflect_dom_object(iterator, &*iterable.global())
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

impl<T: DomObjectIteratorWrap + JSTraceable + Iterable> DomObjectWrap for IterableIterator<T> {
    const WRAP: unsafe fn(
        JSContext,
        &GlobalScope,
        Option<HandleObject>,
        Box<Self>,
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
