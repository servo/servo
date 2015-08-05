/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data structure measurement.

use libc::{c_void, size_t};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, LinkedList, hash_state};
use std::hash::Hash;
use std::mem::{size_of, transmute};
use std::sync::Arc;
use std::rc::Rc;
use std::result::Result;

use azure::azure_hl::Color;
use cssparser::Color as CSSParserColor;
use cssparser::RGBA;
use cursor::Cursor;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Matrix2D, Matrix4};
use euclid::length::Length;
use euclid::scale_factor::ScaleFactor;
use geometry::{PagePx, ViewportPx, Au};
use html5ever::tree_builder::QuirksMode;
use layers::geometry::DevicePixel;
use js::jsapi::Heap;
use js::rust::GCMethods;
use js::jsval::JSVal;
use logical_geometry::WritingMode;
use range::Range;
use str::LengthOrPercentageOrAuto;
use string_cache::atom::Atom;
use string_cache::namespace::Namespace;
use url;
use hyper::method::Method;
use hyper::http::RawStatus;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use selectors::parser::{PseudoElement, Selector, CompoundSelector, SimpleSelector, Combinator};
use rand::OsRng;

extern {
    // Get the size of a heap block.
    //
    // Ideally Rust would expose a function like this in std::rt::heap, which would avoid the
    // jemalloc dependence.
    //
    // The C prototype is `je_malloc_usable_size(JEMALLOC_USABLE_SIZE_CONST void *ptr)`. On some
    // platforms `JEMALLOC_USABLE_SIZE_CONST` is `const` and on some it is empty. But in practice
    // this function doesn't modify the contents of the block that `ptr` points to, so we use
    // `*const c_void` here.
    fn je_malloc_usable_size(ptr: *const c_void) -> size_t;
}

// A wrapper for je_malloc_usable_size that handles `EMPTY` and returns `usize`.
pub fn heap_size_of(ptr: *const c_void) -> usize {
    if ptr == ::std::rt::heap::EMPTY as *const c_void {
        0
    } else {
        unsafe { je_malloc_usable_size(ptr) as usize }
    }
}

// The simplest trait for measuring the size of heap data structures. More complex traits that
// return multiple measurements -- e.g. measure text separately from images -- are also possible,
// and should be used when appropriate.
//
pub trait HeapSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this value, but not the
    /// space taken up by the value itself (i.e. what size_of::<T> measures, more or less); that
    /// space is handled by the implementation of HeapSizeOf for Box<T> below.
    fn heap_size_of_children(&self) -> usize;
}

// There are two possible ways to measure the size of `self` when it's on the heap: compute it
// (with `::std::rt::heap::usable_size(::std::mem::size_of::<T>(), 0)`) or measure it directly
// using the heap allocator (with `heap_size_of`). We do the latter, for the following reasons.
//
// * The heap allocator is the true authority for the sizes of heap blocks; its measurement is
//   guaranteed to be correct. In comparison, size computations are error-prone. (For example, the
//   `rt::heap::usable_size` function used in some of Rust's non-default allocator implementations
//   underestimate the true usable size of heap blocks, which is safe in general but would cause
//   under-measurement here.)
//
// * If we measure something that isn't a heap block, we'll get a crash. This keeps us honest,
//   which is important because unsafe code is involved and this can be gotten wrong.
//
// However, in the best case, the two approaches should give the same results.
//
impl<T: HeapSizeOf> HeapSizeOf for Box<T> {
    fn heap_size_of_children(&self) -> usize {
        // Measure size of `self`.
        heap_size_of(&**self as *const T as *const c_void) + (**self).heap_size_of_children()
    }
}

impl HeapSizeOf for String {
    fn heap_size_of_children(&self) -> usize {
        heap_size_of(self.as_ptr() as *const c_void)
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Option<T> {
    fn heap_size_of_children(&self) -> usize {
        match *self {
            None => 0,
            Some(ref x) => x.heap_size_of_children()
        }
    }
}

impl HeapSizeOf for url::Url {
    fn heap_size_of_children(&self) -> usize {
        // Using a struct pattern without `..` rather than `foo.bar` field access
        // makes sure this will be updated if a field is added.
        let &url::Url { ref scheme, ref scheme_data, ref query, ref fragment } = self;
        scheme.heap_size_of_children() +
        scheme_data.heap_size_of_children() +
        query.heap_size_of_children() +
        fragment.heap_size_of_children()
    }
}

impl HeapSizeOf for url::SchemeData {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &url::SchemeData::Relative(ref data) => data.heap_size_of_children(),
            &url::SchemeData::NonRelative(ref str) => str.heap_size_of_children()
        }
    }
}

impl HeapSizeOf for url::RelativeSchemeData {
    fn heap_size_of_children(&self) -> usize {
        // Using a struct pattern without `..` rather than `foo.bar` field access
        // makes sure this will be updated if a field is added.
        let &url::RelativeSchemeData { ref username, ref password, ref host,
                                       ref port, ref default_port, ref path } = self;
        username.heap_size_of_children() +
        password.heap_size_of_children() +
        host.heap_size_of_children() +
        port.heap_size_of_children() +
        default_port.heap_size_of_children() +
        path.heap_size_of_children()
    }
}

impl HeapSizeOf for url::Host {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &url::Host::Domain(ref str) => str.heap_size_of_children(),
            &url::Host::Ipv6(_) => 0
        }
    }
}

impl<T: HeapSizeOf, U: HeapSizeOf> HeapSizeOf for (T, U) {
    fn heap_size_of_children(&self) -> usize {
        self.0.heap_size_of_children() + self.1.heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Arc<T> {
    fn heap_size_of_children(&self) -> usize {
        (**self).heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for RefCell<T> {
    fn heap_size_of_children(&self) -> usize {
        self.borrow().heap_size_of_children()
    }
}

impl<T: HeapSizeOf + Copy> HeapSizeOf for Cell<T> {
    fn heap_size_of_children(&self) -> usize {
        self.get().heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Vec<T> {
    fn heap_size_of_children(&self) -> usize {
        heap_size_of(self.as_ptr() as *const c_void) +
            self.iter().fold(0, |n, elem| n + elem.heap_size_of_children())
    }
}

impl<T> HeapSizeOf for Vec<Rc<T>> {
    fn heap_size_of_children(&self) -> usize {
        // The fate of measuring Rc<T> is still undecided, but we still want to measure
        // the space used for storing them.
        heap_size_of(self.as_ptr() as *const c_void)
    }
}

impl<K: HeapSizeOf, V: HeapSizeOf, S> HeapSizeOf for HashMap<K, V, S>
    where K: Eq + Hash, S: hash_state::HashState {
    fn heap_size_of_children(&self) -> usize {
        //TODO(#6908) measure actual bucket memory usage instead of approximating
        let size = self.capacity() * (size_of::<V>() + size_of::<K>());
        self.iter().fold(size, |n, (key, value)| {
            n + key.heap_size_of_children() + value.heap_size_of_children()
        })
    }
}

// FIXME(njn): We can't implement HeapSizeOf accurately for LinkedList because it requires access
// to the private Node type. Eventually we'll want to add HeapSizeOf (or equivalent) to Rust
// itself. In the meantime, we use the dirty hack of transmuting LinkedList into an identical type
// (LinkedList2) and measuring that.
impl<T: HeapSizeOf> HeapSizeOf for LinkedList<T> {
    fn heap_size_of_children(&self) -> usize {
        let list2: &LinkedList2<T> = unsafe { transmute(self) };
        list2.heap_size_of_children()
    }
}

struct LinkedList2<T> {
    _length: usize,
    list_head: Link<T>,
    _list_tail: Rawlink<Node<T>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Rawlink<T> {
    _p: *mut T,
}

struct Node<T> {
    next: Link<T>,
    _prev: Rawlink<Node<T>>,
    value: T,
}

impl<T: HeapSizeOf> HeapSizeOf for Node<T> {
    // Unlike most heap_size_of_children() functions, this one does *not* measure descendents.
    // Instead, LinkedList2<T>::heap_size_of_children() handles that, so that it can use iteration
    // instead of recursion, which avoids potentially blowing the stack.
    fn heap_size_of_children(&self) -> usize {
        self.value.heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for LinkedList2<T> {
    fn heap_size_of_children(&self) -> usize {
        let mut size = 0;
        let mut curr: &Link<T> = &self.list_head;
        while curr.is_some() {
            size += (*curr).heap_size_of_children();
            curr = &curr.as_ref().unwrap().next;
        }
        size
    }
}

// This is a basic sanity check. If the representation of LinkedList changes such that it becomes a
// different size to LinkedList2, this will fail at compile-time.
#[allow(dead_code)]
unsafe fn linked_list2_check() {
    transmute::<LinkedList<i32>, LinkedList2<i32>>(panic!());
}

// Currently, types that implement the Drop type are larger than those that don't. Because
// LinkedList implements Drop, LinkedList2 must also so that linked_list2_check() doesn't fail.
impl<T> Drop for LinkedList2<T> {
    fn drop(&mut self) {}
}

/// For use on types defined in external crates
/// with known heap sizes.
#[macro_export]
macro_rules! known_heap_size(
    ($size:expr, $($ty:ident),+) => (
        $(
            impl $crate::mem::HeapSizeOf for $ty {
                #[inline(always)]
                fn heap_size_of_children(&self) -> usize {
                    $size
                }
            }
        )+
    );
    ($size: expr, $($ty:ident<$($gen:ident),+>),+) => (
        $(
        impl<$($gen: $crate::mem::HeapSizeOf),+> $crate::mem::HeapSizeOf for $ty<$($gen),+> {
            #[inline(always)]
            fn heap_size_of_children(&self) -> usize {
                $size
            }
        }
        )+
    );
);

// This is measured properly by the heap measurement implemented in SpiderMonkey.
impl<T: Copy + GCMethods<T>> HeapSizeOf for Heap<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl HeapSizeOf for Method {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &Method::Extension(ref str) => str.heap_size_of_children(),
            _ => 0
        }
    }
}

impl<T: HeapSizeOf, U: HeapSizeOf> HeapSizeOf for Result<T, U> {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &Result::Ok(ref ok) => ok.heap_size_of_children(),
            &Result::Err(ref err) => err.heap_size_of_children()
        }
    }
}

impl HeapSizeOf for () {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl HeapSizeOf for Selector {
    fn heap_size_of_children(&self) -> usize {
        let &Selector { ref compound_selectors, ref pseudo_element, ref specificity } = self;
        compound_selectors.heap_size_of_children() + pseudo_element.heap_size_of_children() +
        specificity.heap_size_of_children()
    }
}

impl HeapSizeOf for CompoundSelector {
    fn heap_size_of_children(&self) -> usize {
        let &CompoundSelector { ref simple_selectors, ref next } = self;
        simple_selectors.heap_size_of_children() + next.heap_size_of_children()
    }
}

impl HeapSizeOf for SimpleSelector {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &SimpleSelector::Negation(ref vec) => vec.heap_size_of_children(),
            &SimpleSelector::AttrIncludes(_, ref str) | &SimpleSelector::AttrPrefixMatch(_, ref str) |
            &SimpleSelector::AttrSubstringMatch(_, ref str) | &SimpleSelector::AttrSuffixMatch(_, ref str)
            => str.heap_size_of_children(),
            &SimpleSelector::AttrEqual(_, ref str, _) => str.heap_size_of_children(),
            &SimpleSelector::AttrDashMatch(_, ref first, ref second) => first.heap_size_of_children()
            + second.heap_size_of_children(),
            // All other types come down to Atom, enum or i32, all 0
            _ => 0
        }
    }
}

impl HeapSizeOf for ContentType {
    fn heap_size_of_children(&self) -> usize {
        let &ContentType(ref mime) = self;
        mime.heap_size_of_children()
    }
}

impl HeapSizeOf for Mime {
    fn heap_size_of_children(&self) -> usize {
        let &Mime(ref top_level, ref sub_level, ref vec) = self;
        top_level.heap_size_of_children() + sub_level.heap_size_of_children() +
        vec.heap_size_of_children()
    }
}

impl HeapSizeOf for TopLevel {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &TopLevel::Ext(ref str) => str.heap_size_of_children(),
            _ => 0
        }
    }
}

impl HeapSizeOf for SubLevel {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &SubLevel::Ext(ref str) => str.heap_size_of_children(),
            _ => 0
        }
    }
}

impl HeapSizeOf for Attr {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &Attr::Ext(ref str) => str.heap_size_of_children(),
            _ => 0
        }
    }
}

impl HeapSizeOf for Value {
    fn heap_size_of_children(&self) -> usize {
        match self {
            &Value::Ext(ref str) => str.heap_size_of_children(),
            _ => 0
        }
    }
}

known_heap_size!(0, u8, u16, u32, u64, usize);
known_heap_size!(0, i8, i16, i32, i64, isize);
known_heap_size!(0, bool, f32, f64);

known_heap_size!(0, Rect<T>, Point2D<T>, Size2D<T>, Matrix2D<T>, SideOffsets2D<T>, Range<T>);
known_heap_size!(0, Length<T, U>, ScaleFactor<T, U, V>);

known_heap_size!(0, Au, WritingMode, CSSParserColor, Color, RGBA, Cursor, Matrix4, Atom, Namespace);
known_heap_size!(0, JSVal, PagePx, ViewportPx, DevicePixel, QuirksMode, OsRng, RawStatus, LengthOrPercentageOrAuto);

known_heap_size!(0, PseudoElement, Combinator, str);
