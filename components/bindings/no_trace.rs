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
//!    This is typically derived via a `#[dom_struct]`
//!    (implies `#[derive(JSTraceable)]`) annotation.
//!    Non-JS-managed types have an empty inline `trace()` method,
//!    achieved via `no_jsmanaged_fields!` or similar.
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `JS_CallUnbarrieredObjectTracer()` with a
//!    pointer to the `JSObject` for the reflector. This notifies the GC, which
//!    will add the object to the graph, and will trace that object as well.
//! 5. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `no_jsmanaged_fields!()` macro adds an empty implementation of `JSTraceable` to
//! a datatype.

use trace::JSTraceable;

use canvas_traits::WebGLError;
use canvas_traits::{CanvasGradientStop, LinearGradientStyle, RadialGradientStyle};
use canvas_traits::{CompositionOrBlending, LineCapStyle, LineJoinStyle, RepetitionStyle};
use cssparser::RGBA;
use pointers::{JS, Root};
//use dom::bindings::refcounted::Trusted;
use reflector::{Reflectable, Reflector};
//use dom::bindings::utils::WindowProxyHandler;
use encoding::types::EncodingRef;
use euclid::length::Length as EuclidLength;
use euclid::matrix2d::Matrix2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use html5ever::tree_builder::QuirksMode;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::mime::Mime;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use js::jsapi::JS_CallUnbarrieredObjectTracer;
use js::jsapi::{GCTraceKindToAscii, Heap, JSGCTraceKind, JSObject, JSTracer, JS_CallObjectTracer, JS_CallValueTracer};
use js::jsval::JSVal;
use js::rust::Runtime;
use libc;
use msg::constellation_msg::{ConstellationChan, ScriptMsg};
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData, WorkerId};
use msg::constellation_msg::{Key};
use net_traits::Metadata;
use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask};
use net_traits::storage_task::StorageType;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
//use script_task::ScriptChan;
use script_traits::{TimerEventId, TimerSource, UntrustedNodeAddress};
use selectors::parser::PseudoElement;
use selectors::states::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::boxed::FnBox;
use std::cell::{Cell, UnsafeCell};
use std::collections::hash_state::HashState;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::intrinsics::return_address;
use std::iter::{FromIterator, IntoIterator};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use string_cache::{Atom, Namespace, QualName};
use style::attr::{AttrIdentifier, AttrValue};
use style::properties::PropertyDeclarationBlock;
use style::restyle_hints::ElementSnapshot;
use style::values::specified::Length;
use url::Url;
use util::str::{DOMString, LengthOrPercentageOrAuto};
use uuid::Uuid;
use websocket::client::sender::Sender as WSSender;
use rand::{OsRng, Rng};
use websocket::stream::WebSocketStream;
use style::stylesheets::{Stylesheet};

/// For use on non-jsmanaged types
/// Use #[derive(JSTraceable)] on JS managed types
macro_rules! no_jsmanaged_fields(
    ($($ty:ident),+) => (
        $(
            impl $crate::trace::JSTraceable for $ty {
                #[inline]
                fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                    // Do nothing
                }
            }
        )+
    );
    ($ty:ident<$($gen:ident),+>) => (
        impl<$($gen),+> $crate::trace::JSTraceable for $ty<$($gen),+> {
            #[inline]
            fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                // Do nothing
            }
        }
    );
    ($ty:ident<$($gen:ident: $bound:ident),+>) => (
        impl<$($gen: $bound),+> $crate::trace::JSTraceable for $ty<$($gen),+> {
            #[inline]
            fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                // Do nothing
            }
        }
    );
);


no_jsmanaged_fields!(EncodingRef);
no_jsmanaged_fields!(Reflector);
no_jsmanaged_fields!(bool, f32, f64, String, Url, AtomicBool, Uuid);
no_jsmanaged_fields!(usize, u8, u16, u32, u64);
no_jsmanaged_fields!(isize, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(Size2D<T>);
no_jsmanaged_fields!(Arc<T>);
no_jsmanaged_fields!(Image, ImageCacheChan, ImageCacheTask);
no_jsmanaged_fields!(Metadata);
no_jsmanaged_fields!(Atom, Namespace, QualName);
//no_jsmanaged_fields!(Trusted<T: Reflectable>);
no_jsmanaged_fields!(PropertyDeclarationBlock);
no_jsmanaged_fields!(HashSet<T>);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId);
no_jsmanaged_fields!(TimerEventId, TimerSource);
no_jsmanaged_fields!(WorkerId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Runtime);
no_jsmanaged_fields!(Headers, Method);
//no_jsmanaged_fields!(LayoutChan);
//no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);
no_jsmanaged_fields!(EuclidLength<Unit, T>);
no_jsmanaged_fields!(Matrix2D<T>);
no_jsmanaged_fields!(StorageType);
no_jsmanaged_fields!(CanvasGradientStop, LinearGradientStyle, RadialGradientStyle);
no_jsmanaged_fields!(LineCapStyle, LineJoinStyle, CompositionOrBlending);
no_jsmanaged_fields!(RepetitionStyle);
no_jsmanaged_fields!(WebGLError);
no_jsmanaged_fields!(TimeProfilerChan);
no_jsmanaged_fields!(MemProfilerChan);
no_jsmanaged_fields!(PseudoElement);
no_jsmanaged_fields!(Length);
no_jsmanaged_fields!(ElementState);
no_jsmanaged_fields!(DOMString);
no_jsmanaged_fields!(Mime);
no_jsmanaged_fields!(AttrIdentifier);
no_jsmanaged_fields!(AttrValue);
no_jsmanaged_fields!(ElementSnapshot);
no_jsmanaged_fields!(WSSender<WebSocketStream>);
no_jsmanaged_fields!(OsRng);
no_jsmanaged_fields!(Key);
no_jsmanaged_fields!(Stylesheet);

impl JSTraceable for ConstellationChan<ScriptMsg> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<FnBox(f64, )> {
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

impl<A, B> JSTraceable for fn(A) -> B {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<T> JSTraceable for IpcSender<T> where T: Deserialize + Serialize {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for () {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<T> JSTraceable for IpcReceiver<T> where T: Deserialize + Serialize {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}
