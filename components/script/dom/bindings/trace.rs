/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for tracing JS-managed values.
//!
//! The lifetime of DOM objects is managed by the SpiderMonkey Garbage
//! Collector. A rooted DOM object implementing the interface `Foo` is traced
//! as follows:
//!
//! 1. The GC calls `_trace` defined in `FooBinding` during the marking
//!    phase. (This happens through `JSClass.trace` for non-proxy bindings, and
//!    through `ProxyTraps.trace` otherwise.)
//! 2. `_trace` calls `Foo::trace()` (an implementation of `JSTraceable`).
//!     This is typically derived via a #[jstraceable] annotation
//! 3. For all fields (except those wrapped in `Untraceable`), `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 5. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.
//!
//! The untraceable!() macro adds an empty implementation of JSTraceable to
//! a datatype.

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::EventTarget;

use js::jsapi::{JSObject, JSTracer, JS_CallValueTracer, JS_CallObjectTracer};
use js::jsval::JSVal;

use libc;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use url::Url;
use servo_util::atom::Atom;
use servo_util::namespace::Namespace;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData};
use net::image_cache_task::ImageCacheTask;
use script_traits::ScriptControlChan;
use std::collections::hashmap::HashMap;
use collections::hash::Hash;
use style::PropertyDeclarationBlock;

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", self.reflector());
    }
}

untraceable!(Reflector)

/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    fn trace(&self, trc: *mut JSTracer);
}

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, orig_val: JSVal) {
    if !orig_val.is_markable() {
        return;
    }

    debug!("tracing value {:s}", description);
    let name = description.to_c_str();
    let mut val = orig_val;
    unsafe {
        JS_CallValueTracer(tracer, &mut val, name.as_ptr());
    }
    assert!(val == orig_val);
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace_object(tracer, description, reflector.get_jsobject())
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, orig_obj: *mut JSObject) {
    debug!("tracing {:s}", description);
    let name = description.to_c_str();
    let mut obj = orig_obj;
    unsafe {
        JS_CallObjectTracer(tracer, &mut obj, name.as_ptr());
    }
    // FIXME: JS_CallObjectTracer could theoretically do something that can
    //        cause pointers to shuffle around. We need to pass a *mut *mut JSObject
    //        to JS_CallObjectTracer, but the JSTraceable trait doesn't allow us
    //        to obtain a mutable reference to self (and thereby self.cb);
    //        All we can do right now is scream loudly if this actually causes
    //        a problem in practice.
    assert!(obj == orig_obj);
}

/// Encapsulates a type that cannot easily have `Encodable` derived automagically,
/// but also does not need to be made known to the SpiderMonkey garbage collector.
///
/// Use only with types that are not associated with a JS reflector and do not contain
/// fields of types associated with JS reflectors.
///
/// This should really only be used for types that are from other crates,
/// so we can't implement `Encodable`. See more details: mozilla#2662.
pub struct Untraceable<T> {
    inner: T,
}

impl<T> Untraceable<T> {
    pub fn new(val: T) -> Untraceable<T> {
        Untraceable {
            inner: val
        }
    }
}

impl<T> Deref<T> for Untraceable<T> {
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<T> DerefMut<T> for Untraceable<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.inner
    }
}

/// Encapsulates a type that can be traced but is boxed in a type we don't
/// control (such as `RefCell`).
///
/// Wrap a field in Traceable and implement the `Encodable` trait
/// for that new concrete type to achieve magic compiler-derived trace hooks.
///
/// We always prefer this, in case the contained type ever changes to something that should be traced.
/// See more details: mozilla#2662.
#[deriving(PartialEq, Clone)]
pub struct Traceable<T> {
    inner: T
}

impl<T> Traceable<T> {
    pub fn new(val: T) -> Traceable<T> {
        Traceable {
            inner: val
        }
    }
}

impl<T> Deref<T> for Traceable<T> {
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<T: JSTraceable> JSTraceable for RefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.borrow().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Rc<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.deref().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Box<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Traceable<Cell<T>> {
    fn trace(&self, trc: *mut JSTracer) {
        self.deref().get().trace(trc)
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Cell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

impl JSTraceable for Traceable<*mut JSObject> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_object(trc, "object", **self);
    }
}

impl JSTraceable for JSVal {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "val", *self);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an untraceable type)
impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

impl<T: JSTraceable> JSTraceable for Option<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        self.as_ref().map(|e| e.trace(trc));
    }
}

impl<K: Eq+Hash+JSTraceable, V: JSTraceable> JSTraceable for HashMap<K, V> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.val0().trace(trc);
            e.val1().trace(trc);
        }
    }
}

untraceable!(bool, f32, f64, String, Url)
untraceable!(uint, u8, u16, u32, u64)
untraceable!(int, i8, i16, i32, i64)
untraceable!(Untraceable<T>)
untraceable!(ImageCacheTask, ScriptControlChan)
untraceable!(Atom, Namespace)
untraceable!(PropertyDeclarationBlock)
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
untraceable!(SubpageId, WindowSizeData, PipelineId)

impl<'a> JSTraceable for &'a str {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

#[allow(unrooted_must_root)]
#[jstraceable]
pub struct RootedVec<T: Reflectable> {
    v: Vec<JS<T>>
}

#[jstraceable]
pub struct RootedJSValVec {
    v: Vec<JSVal>
}

local_data_key!(pub RootedCollections: RefCell<Vec<HashSet<*const ()>>>)

enum CollectionType {
    DOMObjects,
    JSVals,
}

pub fn init_collections() {
    assert!(RootedCollections.get().is_none());
    RootedCollections.replace(Some(RefCell::new(vec!(HashSet::new(), HashSet::new()))));
}

fn drop_rooted_collection<T>(collection: &mut T, type_: CollectionType) {
    let collections = RootedCollections.get();
    let mut collections = collections.as_ref().unwrap().borrow_mut();
    assert!((*collections).get_mut(type_ as uint).remove(&(collection as *mut _ as *const _)));
}

fn init_rooted_collection<T>(collection: &T, type_: CollectionType) {
        let collections = RootedCollections.get();
        let mut collections = collections.as_ref().unwrap().borrow_mut();
        (*collections).get_mut(type_ as uint).insert(collection as *const _ as *const _);
}

impl Drop for RootedJSValVec {
    fn drop(&mut self) {
        drop_rooted_collection(self, JSVals);
    }
}

#[unsafe_destructor]
impl<T: Reflectable> Drop for RootedVec<T> {
    fn drop(&mut self) {
        drop_rooted_collection(self, DOMObjects);
    }
}

impl<T: Reflectable> RootedVec<T> {
    pub fn new() -> RootedVec<T> {
        RootedVec {
            v: vec!()
        }
    }

    pub fn init(&self) {
        init_rooted_collection(self, DOMObjects);
    }
}

impl<T: Reflectable> Deref<Vec<JS<T>>> for RootedVec<T> {
    fn deref<'a>(&'a self) -> &'a Vec<JS<T>> {
        &self.v
    }
}

impl<T: Reflectable> DerefMut<Vec<JS<T>>> for RootedVec<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Vec<JS<T>> {
        &mut self.v
    }
}

impl RootedJSValVec {
    pub fn new() -> RootedJSValVec {
        RootedJSValVec {
            v: vec!()
        }
    }

    pub fn init(&self) {
        init_rooted_collection(self, JSVals);
    }
}

impl Deref<Vec<JSVal>> for RootedJSValVec {
    fn deref<'a>(&'a self) -> &'a Vec<JSVal> {
        &self.v
    }
}

impl DerefMut<Vec<JSVal>> for RootedJSValVec {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Vec<JSVal> {
        &mut self.v
    }
}

fn trace_collection_type<T: JSTraceable>(tracer: *mut JSTracer,
                                         collections: *const HashSet<*const T>) {
    unsafe {
        for collection in (*collections).iter() {
            let traceable = &(**collection) as &JSTraceable;
            let _ = (**collection).trace(tracer);
        }
    }
}

pub extern fn trace_collections(tracer: *mut JSTracer, data: *mut libc::c_void) {
    let collections = data as *const RefCell<Vec<HashSet<*const ()>>>;
    let collections = unsafe { (*collections).borrow() };
    trace_collection_type(tracer,
                          &(*collections)[DOMObjects as uint] as *const _ as *const HashSet<*const RootedVec<EventTarget>>);
    trace_collection_type(tracer,
                          &(*collections)[JSVals as uint] as *const _ as *const HashSet<*const RootedJSValVec>);
}
