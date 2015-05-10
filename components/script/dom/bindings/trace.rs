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
//!    This is typically derived via a `#[dom_struct]` (implies `#[jstraceable]`) annotation.
//!    Non-JS-managed types have an empty inline `trace()` method,
//!    achieved via `no_jsmanaged_fields!` or similar.
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 5. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.
//! 6. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `no_jsmanaged_fields!()` macro adds an empty implementation of `JSTraceable` to
//! a datatype.

use dom::bindings::js::JS;
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflectable, Reflector, WindowProxyHandler};
use script_task::ScriptChan;

use canvas::canvas_paint_task::{CanvasGradientStop, LinearGradientStyle, RadialGradientStyle};
use canvas::canvas_paint_task::{LineCapStyle, LineJoinStyle, CompositionOrBlending};
use cssparser::RGBA;
use encoding::types::EncodingRef;
use geom::matrix2d::Matrix2D;
use geom::rect::Rect;
use html5ever::tree_builder::QuirksMode;
use hyper::header::Headers;
use hyper::method::Method;
use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSGCTraceKind};
use js::jsval::JSVal;
use js::rust::Runtime;
use layout_interface::{LayoutRPC, LayoutChan};
use libc;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData, WorkerId};
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask};
use net_traits::storage_task::StorageType;
use script_traits::ScriptControlChan;
use script_traits::UntrustedNodeAddress;
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::ConstellationChan;
use net_traits::image::base::Image;
use util::smallvec::SmallVec1;
use util::str::{LengthOrPercentageOrAuto};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_state::HashState;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::intrinsics::return_address;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use string_cache::{Atom, Namespace};
use style::properties::PropertyDeclarationBlock;
use url::Url;


/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    /// Trace `self`.
    fn trace(&self, trc: *mut JSTracer);
}

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", self.reflector());
    }
}

no_jsmanaged_fields!(EncodingRef);

no_jsmanaged_fields!(Reflector);

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: JSVal) {
    if !val.is_markable() {
        return;
    }

    unsafe {
        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = !0;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing value {}", description);
        JS_CallTracer(tracer, val.to_gcthing(), val.trace_kind());
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace_object(tracer, description, reflector.get_jsobject())
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: *mut JSObject) {
    unsafe {
        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = !0;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing {}", description);
        JS_CallTracer(tracer, obj as *mut libc::c_void, JSGCTraceKind::JSTRACE_OBJECT);
    }
}

impl<T: JSTraceable> JSTraceable for RefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.borrow().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Rc<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Box<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for *const T {
    fn trace(&self, trc: *mut JSTracer) {
        if !self.is_null() {
            unsafe {
                (**self).trace(trc)
            }
        }
    }
}

impl<T: JSTraceable> JSTraceable for *mut T {
    fn trace(&self, trc: *mut JSTracer) {
        if !self.is_null() {
            unsafe {
                (**self).trace(trc)
            }
        }
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Cell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

impl JSTraceable for *mut JSObject {
    fn trace(&self, trc: *mut JSTracer) {
        trace_object(trc, "object", *self);
    }
}

impl JSTraceable for JSVal {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "val", *self);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable + 'static> JSTraceable for SmallVec1<T> {
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

impl<T: JSTraceable, U: JSTraceable> JSTraceable for Result<T, U> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        match *self {
            Ok(ref inner) => inner.trace(trc),
            Err(ref inner) => inner.trace(trc),
        }
    }
}

impl<K,V,S> JSTraceable for HashMap<K, V, S>
    where K: Hash + Eq + JSTraceable,
          V: JSTraceable,
          S: HashState,
          <S as HashState>::Hasher: Hasher,
{
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in self.iter() {
            k.trace(trc);
            v.trace(trc);
        }
    }
}

impl<A: JSTraceable, B: JSTraceable> JSTraceable for (A, B) {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b) = *self;
        a.trace(trc);
        b.trace(trc);
    }
}


no_jsmanaged_fields!(bool, f32, f64, String, Url);
no_jsmanaged_fields!(usize, u8, u16, u32, u64);
no_jsmanaged_fields!(isize, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(Arc<T>);
no_jsmanaged_fields!(Image, ImageCacheChan, ImageCacheTask, ScriptControlChan);
no_jsmanaged_fields!(Atom, Namespace);
no_jsmanaged_fields!(Trusted<T>);
no_jsmanaged_fields!(PropertyDeclarationBlock);
no_jsmanaged_fields!(HashSet<T>);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId);
no_jsmanaged_fields!(WorkerId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Runtime);
no_jsmanaged_fields!(Headers, Method);
no_jsmanaged_fields!(ConstellationChan);
no_jsmanaged_fields!(LayoutChan);
no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);
no_jsmanaged_fields!(Matrix2D<T>);
no_jsmanaged_fields!(StorageType);
no_jsmanaged_fields!(CanvasGradientStop, LinearGradientStyle, RadialGradientStyle);
no_jsmanaged_fields!(LineCapStyle, LineJoinStyle, CompositionOrBlending);

impl JSTraceable for Box<ScriptChan+Send> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<Fn(f64, )> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl<'a> JSTraceable for &'a str {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<A,B> JSTraceable for fn(A) -> B {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<ScriptListener+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<LayoutRPC+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for () {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
    }
}

/// Holds a set of vectors that need to be rooted
pub struct RootedCollectionSet {
    set: Vec<HashSet<*const RootedVec<Void>>>
}

/// TLV Holds a set of vectors that need to be rooted
thread_local!(pub static ROOTED_COLLECTIONS: Rc<RefCell<RootedCollectionSet>> =
              Rc::new(RefCell::new(RootedCollectionSet::new())));

/// Type of `RootedVec`
pub enum CollectionType {
    /// DOM objects
    DOMObjects,
    /// `JSVal`s
    JSVals,
    /// `*mut JSObject`s
    JSObjects,
}


impl RootedCollectionSet {
    fn new() -> RootedCollectionSet {
        RootedCollectionSet {
            set: vec!(HashSet::new(), HashSet::new(), HashSet::new())
        }
    }

    fn remove<T: VecRootableType>(collection: &RootedVec<T>) {
        ROOTED_COLLECTIONS.with(|ref collections| {
            let type_ = VecRootableType::tag(None::<T>);
            let mut collections = collections.borrow_mut();
            assert!(collections.set[type_ as usize].remove(&(collection as *const _ as *const _)));
        });
    }

    fn add<T: VecRootableType>(collection: &RootedVec<T>) {
        ROOTED_COLLECTIONS.with(|ref collections| {
            let type_ = VecRootableType::tag(None::<T>);
            let mut collections = collections.borrow_mut();
            collections.set[type_ as usize].insert(collection as *const _ as *const _);
        })
    }

    unsafe fn trace(&self, tracer: *mut JSTracer) {
        fn trace_collection_type<T>(tracer: *mut JSTracer,
                                    collections: &HashSet<*const RootedVec<Void>>)
            where T: JSTraceable + VecRootableType
        {
            for collection in collections {
                let collection: *const RootedVec<Void> = *collection;
                let collection = collection as *const RootedVec<T>;
                unsafe {
                    let _ = (*collection).trace(tracer);
                }
            }
        }

        let dom_collections = &self.set[CollectionType::DOMObjects as usize] as *const _ as *const HashSet<*const RootedVec<JS<Void>>>;
        for dom_collection in (*dom_collections).iter() {
            for reflector in (**dom_collection).iter() {
                trace_reflector(tracer, "", reflector.reflector());
            }
        }

        trace_collection_type::<JSVal>(tracer, &self.set[CollectionType::JSVals as usize]);
        trace_collection_type::<*mut JSObject>(tracer, &self.set[CollectionType::JSObjects as usize]);
    }
}


/// Trait implemented by all types that can be used with RootedVec
pub trait VecRootableType {
    /// Return the type tag used to determine how to trace RootedVec
    fn tag(_a: Option<Self>) -> CollectionType;
}

impl<T: Reflectable> VecRootableType for JS<T> {
    fn tag(_a: Option<JS<T>>) -> CollectionType { CollectionType::DOMObjects }
}

impl VecRootableType for JSVal {
    fn tag(_a: Option<JSVal>) -> CollectionType { CollectionType::JSVals }
}

impl VecRootableType for *mut JSObject {
    fn tag(_a: Option<*mut JSObject>) -> CollectionType { CollectionType::JSObjects }
}

enum Void {}

impl VecRootableType for Void {
    fn tag(_a: Option<Void>) -> CollectionType { unreachable!() }
}

impl Reflectable for Void {
    fn reflector<'a>(&'a self) -> &'a Reflector { unreachable!() }
}

/// A vector of items that are rooted for the lifetime
/// of this struct
#[allow(unrooted_must_root)]
#[no_move]
pub struct RootedVec<T: VecRootableType> {
    v: Vec<T>
}


impl<T: VecRootableType> RootedVec<T> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new() -> RootedVec<T> {
        let addr = unsafe {
            return_address() as *const libc::c_void
        };

        RootedVec::new_with_destination_address(addr)
    }

    /// Create a vector of items of type T. This constructor is specific
    /// for RootCollection.
    pub fn new_with_destination_address(addr: *const libc::c_void) -> RootedVec<T> {
        unsafe {
            RootedCollectionSet::add::<T>(&*(addr as *const _));
        }
        RootedVec::<T> { v: vec!() }
    }
}

impl<T: VecRootableType> Drop for RootedVec<T> {
    fn drop(&mut self) {
        RootedCollectionSet::remove(self);
    }
}

impl<T: VecRootableType> Deref for RootedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.v
    }
}

impl<T: VecRootableType> DerefMut for RootedVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}


/// SM Callback that traces the rooted collections
pub unsafe fn trace_collections(tracer: *mut JSTracer) {
    ROOTED_COLLECTIONS.with(|ref collections| {
        let collections = collections.borrow();
        collections.trace(tracer);
    });
}
