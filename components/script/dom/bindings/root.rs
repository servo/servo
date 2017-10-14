/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Smart pointers for the JS-managed DOM objects.
//!
//! The DOM is made up of DOM objects whose lifetime is entirely controlled by
//! the whims of the SpiderMonkey garbage collector. The types in this module
//! are designed to ensure that any interactions with said Rust types only
//! occur on values that will remain alive the entire time.
//!
//! Here is a brief overview of the important types:
//!
//! - `Root<T>`: a stack-based rooted value.
//! - `DomRoot<T>`: a stack-based reference to a rooted DOM object.
//! - `Dom<T>`: a reference to a DOM object that can automatically be traced by
//!   the GC when encountered as a field of a Rust structure.
//!
//! `Dom<T>` does not allow access to their inner value without explicitly
//! creating a stack-based root via the `root` method. This returns a `DomRoot<T>`,
//! which causes the JS-owned value to be uncollectable for the duration of the
//! `Root` object's lifetime. A reference to the object can then be obtained
//! from the `Root` object. These references are not allowed to outlive their
//! originating `DomRoot<T>`.
//!

use dom::bindings::conversions::DerivedFrom;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::trace::JSTraceable;
use dom::bindings::trace::trace_reflector;
use dom::node::Node;
use heapsize::HeapSizeOf;
use js::jsapi::{JSObject, JSTracer, Heap};
use js::rust::GCMethods;
use mitochondria::OnceCell;
use nonzero::NonZero;
use script_layout_interface::TrustedNodeAddress;
use std::cell::{Cell, UnsafeCell};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;
use style::thread_state;

/// A rooted value.
#[allow(unrooted_must_root)]
#[allow_unrooted_interior]
pub struct Root<T: StableTraceObject> {
    /// The value to root.
    value: T,
    /// List that ensures correct dynamic root ordering
    root_list: *const RootCollection,
}

impl<T> Root<T>
where
    T: StableTraceObject + 'static,
{
    /// Create a new stack-bounded root for the provided value.
    /// It cannot outlive its associated `RootCollection`, and it gives
    /// out references which cannot outlive this new `Root`.
    #[allow(unrooted_must_root)]
    unsafe fn new(value: T) -> Self {
        debug_assert!(thread_state::get().is_script());
        STACK_ROOTS.with(|ref root_list| {
            let root_list = &*root_list.get().unwrap();
            root_list.root(value.stable_trace_object());
            Root { value, root_list }
        })
    }
}

/// Represents values that can be rooted through a stable address that will
/// not change for their whole lifetime.
pub unsafe trait StableTraceObject {
    /// Returns a stable trace object which address won't change for the whole
    /// lifetime of the value.
    fn stable_trace_object(&self) -> *const JSTraceable;
}

unsafe impl<T> StableTraceObject for Dom<T>
where
    T: DomObject,
{
    fn stable_trace_object<'a>(&'a self) -> *const JSTraceable {
        // The JSTraceable impl for Reflector doesn't actually do anything,
        // so we need this shenanigan to actually trace the reflector of the
        // T pointer in Dom<T>.
        #[allow(unrooted_must_root)]
        struct ReflectorStackRoot(Reflector);
        unsafe impl JSTraceable for ReflectorStackRoot {
            unsafe fn trace(&self, tracer: *mut JSTracer) {
                trace_reflector(tracer, "on stack", &self.0);
            }
        }
        unsafe {
            &*(self.reflector() as *const Reflector as *const ReflectorStackRoot)
        }
    }
}

impl<T> Deref for Root<T>
where
    T: Deref + StableTraceObject,
{
    type Target = <T as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        debug_assert!(thread_state::get().is_script());
        &self.value
    }
}

impl<T> Drop for Root<T>
where
    T: StableTraceObject,
{
    fn drop(&mut self) {
        unsafe {
            (*self.root_list).unroot(self.value.stable_trace_object());
        }
    }
}

/// A rooted reference to a DOM object.
pub type DomRoot<T> = Root<Dom<T>>;

impl<T: Castable> DomRoot<T> {
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(root: DomRoot<T>) -> DomRoot<U>
        where U: Castable,
              T: DerivedFrom<U>
    {
        unsafe { mem::transmute(root) }
    }

    /// Cast a DOM object root downwards to one of the interfaces it might implement.
    pub fn downcast<U>(root: DomRoot<T>) -> Option<DomRoot<U>>
        where U: DerivedFrom<T>
    {
        if root.is::<U>() {
            Some(unsafe { mem::transmute(root) })
        } else {
            None
        }
    }
}

impl<T: DomObject> DomRoot<T> {
    /// Generate a new root from a reference
    pub fn from_ref(unrooted: &T) -> DomRoot<T> {
        unsafe { DomRoot::new(Dom::from_ref(unrooted)) }
    }
}

impl<T> HeapSizeOf for DomRoot<T>
where
    T: DomObject + HeapSizeOf,
{
    fn heap_size_of_children(&self) -> usize {
        (**self).heap_size_of_children()
    }
}

impl<T> PartialEq for DomRoot<T>
where
    T: DomObject,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Clone for DomRoot<T>
where
    T: DomObject,
{
    fn clone(&self) -> DomRoot<T> {
        DomRoot::from_ref(&*self)
    }
}

unsafe impl<T> JSTraceable for DomRoot<T>
where
    T: DomObject,
{
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Already traced.
    }
}

/// A rooting mechanism for reflectors on the stack.
/// LIFO is not required.
///
/// See also [*Exact Stack Rooting - Storing a GCPointer on the CStack*]
/// (https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting).
pub struct RootCollection {
    roots: UnsafeCell<Vec<*const JSTraceable>>,
}

thread_local!(static STACK_ROOTS: Cell<Option<*const RootCollection>> = Cell::new(None));

pub struct ThreadLocalStackRoots<'a>(PhantomData<&'a u32>);

impl<'a> ThreadLocalStackRoots<'a> {
    pub fn new(roots: &'a RootCollection) -> Self {
        STACK_ROOTS.with(|ref r| {
            r.set(Some(roots))
        });
        ThreadLocalStackRoots(PhantomData)
    }
}

impl<'a> Drop for ThreadLocalStackRoots<'a> {
    fn drop(&mut self) {
        STACK_ROOTS.with(|ref r| r.set(None));
    }
}

impl RootCollection {
    /// Create an empty collection of roots
    pub fn new() -> RootCollection {
        debug_assert!(thread_state::get().is_script());
        RootCollection {
            roots: UnsafeCell::new(vec![]),
        }
    }

    /// Starts tracking a trace object.
    unsafe fn root(&self, object: *const JSTraceable) {
        debug_assert!(thread_state::get().is_script());
        (*self.roots.get()).push(object);
    }

    /// Stops tracking a trace object, asserting if it isn't found.
    unsafe fn unroot(&self, object: *const JSTraceable) {
        debug_assert!(thread_state::get().is_script());
        let roots = &mut *self.roots.get();
        match roots.iter().rposition(|r| *r == object) {
            Some(idx) => {
                roots.remove(idx);
            },
            None => panic!("Can't remove a root that was never rooted!"),
        }
    }
}

/// SM Callback that traces the rooted reflectors
pub unsafe fn trace_roots(tracer: *mut JSTracer) {
    debug!("tracing stack roots");
    STACK_ROOTS.with(|ref collection| {
        let collection = &*(*collection.get().unwrap()).roots.get();
        for root in collection {
            (**root).trace(tracer);
        }
    });
}

/// Get a reference out of a rooted value.
pub trait RootedReference<'root> {
    /// The type of the reference.
    type Ref: 'root;
    /// Obtain a reference out of the rooted value.
    fn r(&'root self) -> Self::Ref;
}

impl<'root, T: DomObject + 'root> RootedReference<'root> for DomRoot<T> {
    type Ref = &'root T;
    fn r(&'root self) -> &'root T {
        self
    }
}

impl<'root, T: DomObject + 'root> RootedReference<'root> for Dom<T> {
    type Ref = &'root T;
    fn r(&'root self) -> &'root T {
        &self
    }
}

impl<'root, T: JSTraceable + DomObject + 'root> RootedReference<'root> for [Dom<T>] {
    type Ref = &'root [&'root T];
    fn r(&'root self) -> &'root [&'root T] {
        unsafe { mem::transmute(self) }
    }
}

impl<'root, T: DomObject + 'root> RootedReference<'root> for Rc<T> {
    type Ref = &'root T;
    fn r(&'root self) -> &'root T {
        self
    }
}

impl<'root, T: RootedReference<'root> + 'root> RootedReference<'root> for Option<T> {
    type Ref = Option<T::Ref>;
    fn r(&'root self) -> Option<T::Ref> {
        self.as_ref().map(RootedReference::r)
    }
}

/// A traced reference to a DOM object
///
/// This type is critical to making garbage collection work with the DOM,
/// but it is very dangerous; if garbage collection happens with a `Dom<T>`
/// on the stack, the `Dom<T>` can point to freed memory.
///
/// This should only be used as a field in other DOM objects.
#[must_root]
pub struct Dom<T> {
    ptr: NonZero<*const T>,
}

// Dom<T> is similar to Rc<T>, in that it's not always clear how to avoid double-counting.
// For now, we choose not to follow any such pointers.
impl<T> HeapSizeOf for Dom<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T> Dom<T> {
    /// Returns `LayoutDom<T>` containing the same pointer.
    pub unsafe fn to_layout(&self) -> LayoutDom<T> {
        debug_assert!(thread_state::get().is_layout());
        LayoutDom {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T: DomObject> Dom<T> {
    /// Create a Dom<T> from a &T
    #[allow(unrooted_must_root)]
    pub fn from_ref(obj: &T) -> Dom<T> {
        debug_assert!(thread_state::get().is_script());
        Dom {
            ptr: unsafe { NonZero::new_unchecked(&*obj) },
        }
    }
}

impl<T: DomObject> Deref for Dom<T> {
    type Target = T;

    fn deref(&self) -> &T {
        debug_assert!(thread_state::get().is_script());
        // We can only have &Dom<T> from a rooted thing, so it's safe to deref
        // it to &T.
        unsafe { &*self.ptr.get() }
    }
}

unsafe impl<T: DomObject> JSTraceable for Dom<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        #[cfg(all(feature = "unstable", debug_assertions))]
        let trace_str = format!("for {} on heap", ::std::intrinsics::type_name::<T>());
        #[cfg(all(feature = "unstable", debug_assertions))]
        let trace_info = &trace_str[..];
        #[cfg(not(all(feature = "unstable", debug_assertions)))]
        let trace_info = "for DOM object on heap";

        trace_reflector(trc,
                        trace_info,
                        (*self.ptr.get()).reflector());
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
#[allow_unrooted_interior]
pub struct LayoutDom<T> {
    ptr: NonZero<*const T>,
}

impl<T: Castable> LayoutDom<T> {
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(&self) -> LayoutDom<U>
        where U: Castable,
              T: DerivedFrom<U>
    {
        debug_assert!(thread_state::get().is_layout());
        let ptr: *const T = self.ptr.get();
        LayoutDom {
            ptr: unsafe { NonZero::new_unchecked(ptr as *const U) },
        }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    pub fn downcast<U>(&self) -> Option<LayoutDom<U>>
        where U: DerivedFrom<T>
    {
        debug_assert!(thread_state::get().is_layout());
        unsafe {
            if (*self.unsafe_get()).is::<U>() {
                let ptr: *const T = self.ptr.get();
                Some(LayoutDom {
                    ptr: NonZero::new_unchecked(ptr as *const U),
                })
            } else {
                None
            }
        }
    }
}

impl<T: DomObject> LayoutDom<T> {
    /// Get the reflector.
    pub unsafe fn get_jsobject(&self) -> *mut JSObject {
        debug_assert!(thread_state::get().is_layout());
        (*self.ptr.get()).reflector().get_jsobject().get()
    }
}

impl<T> Copy for LayoutDom<T> {}

impl<T> PartialEq for Dom<T> {
    fn eq(&self, other: &Dom<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for Dom<T> {}

impl<T> PartialEq for LayoutDom<T> {
    fn eq(&self, other: &LayoutDom<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for LayoutDom<T> {}

impl<T> Hash for Dom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state)
    }
}

impl<T> Hash for LayoutDom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state)
    }
}

impl <T> Clone for Dom<T> {
    #[inline]
    #[allow(unrooted_must_root)]
    fn clone(&self) -> Dom<T> {
        debug_assert!(thread_state::get().is_script());
        Dom {
            ptr: self.ptr.clone(),
        }
    }
}

impl <T> Clone for LayoutDom<T> {
    #[inline]
    fn clone(&self) -> LayoutDom<T> {
        debug_assert!(thread_state::get().is_layout());
        LayoutDom {
            ptr: self.ptr.clone(),
        }
    }
}

impl LayoutDom<Node> {
    /// Create a new JS-owned value wrapped from an address known to be a
    /// `Node` pointer.
    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> LayoutDom<Node> {
        debug_assert!(thread_state::get().is_layout());
        let TrustedNodeAddress(addr) = inner;
        LayoutDom {
            ptr: NonZero::new_unchecked(addr as *const Node),
        }
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`.  Essentially a `Cell<Dom<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[must_root]
#[derive(JSTraceable)]
pub struct MutDom<T: DomObject> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject> MutDom<T> {
    /// Create a new `MutDom`.
    pub fn new(initial: &T) -> MutDom<T> {
        debug_assert!(thread_state::get().is_script());
        MutDom {
            val: UnsafeCell::new(Dom::from_ref(initial)),
        }
    }

    /// Set this `MutDom` to the given value.
    pub fn set(&self, val: &T) {
        debug_assert!(thread_state::get().is_script());
        unsafe {
            *self.val.get() = Dom::from_ref(val);
        }
    }

    /// Get the value in this `MutDom`.
    pub fn get(&self) -> DomRoot<T> {
        debug_assert!(thread_state::get().is_script());
        unsafe {
            DomRoot::from_ref(&*ptr::read(self.val.get()))
        }
    }
}

impl<T: DomObject> HeapSizeOf for MutDom<T> {
    fn heap_size_of_children(&self) -> usize {
        // See comment on HeapSizeOf for Dom<T>.
        0
    }
}

impl<T: DomObject> PartialEq for MutDom<T> {
   fn eq(&self, other: &Self) -> bool {
        unsafe {
            *self.val.get() == *other.val.get()
        }
    }
}

impl<T: DomObject + PartialEq> PartialEq<T> for MutDom<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe {
            **self.val.get() == *other
        }
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<Dom<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[must_root]
#[derive(JSTraceable)]
pub struct MutNullableDom<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> MutNullableDom<T> {
    /// Create a new `MutNullableDom`.
    pub fn new(initial: Option<&T>) -> MutNullableDom<T> {
        debug_assert!(thread_state::get().is_script());
        MutNullableDom {
            ptr: UnsafeCell::new(initial.map(Dom::from_ref)),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> DomRoot<T>
        where F: FnOnce() -> DomRoot<T>
    {
        debug_assert!(thread_state::get().is_script());
        match self.get() {
            Some(inner) => inner,
            None => {
                let inner = cb();
                self.set(Some(&inner));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `Dom<T>` as `LayoutDom<T>`.
    /// For use by layout, which can't use safe types like Temporary.
    #[allow(unrooted_must_root)]
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutDom<T>> {
        debug_assert!(thread_state::get().is_layout());
        ptr::read(self.ptr.get()).map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    #[allow(unrooted_must_root)]
    pub fn get(&self) -> Option<DomRoot<T>> {
        debug_assert!(thread_state::get().is_script());
        unsafe {
            ptr::read(self.ptr.get()).map(|o| DomRoot::from_ref(&*o))
        }
    }

    /// Set this `MutNullableDom` to the given value.
    pub fn set(&self, val: Option<&T>) {
        debug_assert!(thread_state::get().is_script());
        unsafe {
            *self.ptr.get() = val.map(|p| Dom::from_ref(p));
        }
    }

    /// Gets the current value out of this object and sets it to `None`.
    pub fn take(&self) -> Option<DomRoot<T>> {
        let value = self.get();
        self.set(None);
        value
    }
}

impl<T: DomObject> PartialEq for MutNullableDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            *self.ptr.get() == *other.ptr.get()
        }
    }
}

impl<'a, T: DomObject> PartialEq<Option<&'a T>> for MutNullableDom<T> {
    fn eq(&self, other: &Option<&T>) -> bool {
        unsafe {
            *self.ptr.get() == other.map(Dom::from_ref)
        }
    }
}

impl<T: DomObject> Default for MutNullableDom<T> {
    #[allow(unrooted_must_root)]
    fn default() -> MutNullableDom<T> {
        debug_assert!(thread_state::get().is_script());
        MutNullableDom {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl<T: DomObject> HeapSizeOf for MutNullableDom<T> {
    fn heap_size_of_children(&self) -> usize {
        // See comment on HeapSizeOf for Dom<T>.
        0
    }
}

/// A holder that allows to lazily initialize the value only once
/// `Dom<T>`, using OnceCell
/// Essentially a `OnceCell<Dom<T>>`.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[must_root]
pub struct DomOnceCell<T: DomObject> {
    ptr: OnceCell<Dom<T>>,
}

impl<T> DomOnceCell<T>
where
    T: DomObject
{
    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    #[allow(unrooted_must_root)]
    pub fn init_once<F>(&self, cb: F) -> &T
        where F: FnOnce() -> DomRoot<T>
    {
        debug_assert!(thread_state::get().is_script());
        &self.ptr.init_once(|| Dom::from_ref(&cb()))
    }
}

impl<T: DomObject> Default for DomOnceCell<T> {
    #[allow(unrooted_must_root)]
    fn default() -> DomOnceCell<T> {
        debug_assert!(thread_state::get().is_script());
        DomOnceCell {
            ptr: OnceCell::new(),
        }
    }
}

impl<T: DomObject> HeapSizeOf for DomOnceCell<T> {
    fn heap_size_of_children(&self) -> usize {
        // See comment on HeapSizeOf for Dom<T>.
        0
    }
}

#[allow(unrooted_must_root)]
unsafe impl<T: DomObject> JSTraceable for DomOnceCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if let Some(ptr) = self.ptr.as_ref() {
            ptr.trace(trc);
        }
    }
}

impl<T: DomObject> LayoutDom<T> {
    /// Returns an unsafe pointer to the interior of this JS object. This is
    /// the only method that be safely accessed from layout. (The fact that
    /// this is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *const T {
        debug_assert!(thread_state::get().is_layout());
        self.ptr.get()
    }

    /// Returns a reference to the interior of this JS object. This method is
    /// safe to call because it originates from the layout thread, and it cannot
    /// mutate DOM nodes.
    pub fn get_for_script(&self) -> &T {
        debug_assert!(thread_state::get().is_script());
        unsafe { &*self.ptr.get() }
    }
}

/// Helper trait for safer manipulations of Option<Heap<T>> values.
pub trait OptionalHeapSetter {
    type Value;
    /// Update this optional heap value with a new value.
    fn set(&mut self, v: Option<Self::Value>);
}

impl<T: GCMethods + Copy> OptionalHeapSetter for Option<Heap<T>> where Heap<T>: Default {
    type Value = T;
    fn set(&mut self, v: Option<T>) {
        let v = match v {
            None => {
                *self = None;
                return;
            }
            Some(v) => v,
        };

        if self.is_none() {
            *self = Some(Heap::default());
        }

        self.as_ref().unwrap().set(v);
    }
}
