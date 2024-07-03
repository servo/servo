/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

use std::cell::{Cell, OnceCell, UnsafeCell};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::{mem, ptr};

use js::jsapi::{JSObject, JSTracer};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use script_layout_interface::TrustedNodeAddress;
use style::thread_state;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomObject, MutDomObject, Reflector};
use crate::dom::bindings::trace::{trace_reflector, JSTraceable};
use crate::dom::node::Node;

/// A rooted value.
#[allow(crown::unrooted_must_root)]
#[crown::unrooted_must_root_lint::allow_unrooted_interior]
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
    #[allow(crown::unrooted_must_root)]
    pub unsafe fn new(value: T) -> Self {
        unsafe fn add_to_root_list(object: *const dyn JSTraceable) -> *const RootCollection {
            assert_in_script();
            STACK_ROOTS.with(|root_list| {
                let root_list = &*root_list.get().unwrap();
                root_list.root(object);
                root_list
            })
        }

        let root_list = add_to_root_list(value.stable_trace_object());
        Root { value, root_list }
    }
}

/// `StableTraceObject` represents values that can be rooted through a stable address that will
/// not change for their whole lifetime.
/// It is an unsafe trait that requires implementors to ensure certain safety guarantees.
///
/// # Safety
///
/// Implementors of this trait must ensure that the `trace` method correctly accounts for all
/// owned and referenced objects, so that the garbage collector can accurately determine which
/// objects are still in use. Failing to adhere to this contract may result in undefined behavior,
/// such as use-after-free errors.
pub unsafe trait StableTraceObject {
    /// Returns a stable trace object which address won't change for the whole
    /// lifetime of the value.
    fn stable_trace_object(&self) -> *const dyn JSTraceable;
}

unsafe impl<T> StableTraceObject for Dom<T>
where
    T: DomObject,
{
    fn stable_trace_object(&self) -> *const dyn JSTraceable {
        // The JSTraceable impl for Reflector doesn't actually do anything,
        // so we need this shenanigan to actually trace the reflector of the
        // T pointer in Dom<T>.
        #[allow(crown::unrooted_must_root)]
        struct ReflectorStackRoot(Reflector);
        unsafe impl JSTraceable for ReflectorStackRoot {
            unsafe fn trace(&self, tracer: *mut JSTracer) {
                trace_reflector(tracer, "on stack", &self.0);
            }
        }
        unsafe { &*(self.reflector() as *const Reflector as *const ReflectorStackRoot) }
    }
}

unsafe impl<T> StableTraceObject for MaybeUnreflectedDom<T>
where
    T: DomObject,
{
    fn stable_trace_object(&self) -> *const dyn JSTraceable {
        // The JSTraceable impl for Reflector doesn't actually do anything,
        // so we need this shenanigan to actually trace the reflector of the
        // T pointer in Dom<T>.
        #[allow(crown::unrooted_must_root)]
        struct MaybeUnreflectedStackRoot<T>(T);
        unsafe impl<T> JSTraceable for MaybeUnreflectedStackRoot<T>
        where
            T: DomObject,
        {
            unsafe fn trace(&self, tracer: *mut JSTracer) {
                if self.0.reflector().get_jsobject().is_null() {
                    self.0.trace(tracer);
                } else {
                    trace_reflector(tracer, "on stack", self.0.reflector());
                }
            }
        }
        unsafe { &*(self.ptr.as_ptr() as *const T as *const MaybeUnreflectedStackRoot<T>) }
    }
}

impl<T> Deref for Root<T>
where
    T: Deref + StableTraceObject,
{
    type Target = <T as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        assert_in_script();
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
    where
        U: Castable,
        T: DerivedFrom<U>,
    {
        unsafe { mem::transmute(root) }
    }

    /// Cast a DOM object root downwards to one of the interfaces it might implement.
    pub fn downcast<U>(root: DomRoot<T>) -> Option<DomRoot<U>>
    where
        U: DerivedFrom<T>,
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

impl<T> MallocSizeOf for DomRoot<T>
where
    T: DomObject + MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        (**self).size_of(ops)
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
        DomRoot::from_ref(self)
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
/// See also [*Exact Stack Rooting - Storing a GCPointer on the CStack*][cstack].
///
/// [cstack]: https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting
pub struct RootCollection {
    roots: UnsafeCell<Vec<*const dyn JSTraceable>>,
}

thread_local!(static STACK_ROOTS: Cell<Option<*const RootCollection>> = const { Cell::new(None) });

pub struct ThreadLocalStackRoots<'a>(PhantomData<&'a u32>);

impl<'a> ThreadLocalStackRoots<'a> {
    pub fn new(roots: &'a RootCollection) -> Self {
        STACK_ROOTS.with(|r| r.set(Some(roots)));
        ThreadLocalStackRoots(PhantomData)
    }
}

impl<'a> Drop for ThreadLocalStackRoots<'a> {
    fn drop(&mut self) {
        STACK_ROOTS.with(|r| r.set(None));
    }
}

impl RootCollection {
    /// Create an empty collection of roots
    pub fn new() -> RootCollection {
        assert_in_script();
        RootCollection {
            roots: UnsafeCell::new(vec![]),
        }
    }

    /// Starts tracking a trace object.
    unsafe fn root(&self, object: *const dyn JSTraceable) {
        assert_in_script();
        (*self.roots.get()).push(object);
    }

    /// Stops tracking a trace object, asserting if it isn't found.
    unsafe fn unroot(&self, object: *const dyn JSTraceable) {
        assert_in_script();
        let roots = &mut *self.roots.get();
        match roots
            .iter()
            .rposition(|r| std::ptr::addr_eq(*r as *const (), object as *const ()))
        {
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
    STACK_ROOTS.with(|collection| {
        let collection = &*(*collection.get().unwrap()).roots.get();
        for root in collection {
            (**root).trace(tracer);
        }
    });
}

/// Get a slice of references to DOM objects.
pub trait DomSlice<T>
where
    T: JSTraceable + DomObject,
{
    /// Returns the slice of `T` references.
    fn r(&self) -> &[&T];
}

impl<T> DomSlice<T> for [Dom<T>]
where
    T: JSTraceable + DomObject,
{
    #[inline]
    fn r(&self) -> &[&T] {
        let _ = mem::transmute::<Dom<T>, &T>;
        unsafe { &*(self as *const [Dom<T>] as *const [&T]) }
    }
}

/// A traced reference to a DOM object
///
/// This type is critical to making garbage collection work with the DOM,
/// but it is very dangerous; if garbage collection happens with a `Dom<T>`
/// on the stack, the `Dom<T>` can point to freed memory.
///
/// This should only be used as a field in other DOM objects.
#[crown::unrooted_must_root_lint::must_root]
pub struct Dom<T> {
    ptr: ptr::NonNull<T>,
}

// Dom<T> is similar to Rc<T>, in that it's not always clear how to avoid double-counting.
// For now, we choose not to follow any such pointers.
impl<T> MallocSizeOf for Dom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T> Dom<T> {
    /// Returns `LayoutDom<T>` containing the same pointer.
    ///
    /// # Safety
    ///
    /// The `self` parameter to this method must meet all the requirements of [`ptr::NonNull::as_ref`].
    pub unsafe fn to_layout(&self) -> LayoutDom<T> {
        assert_in_layout();
        LayoutDom {
            value: self.ptr.as_ref(),
        }
    }
}

impl<T: DomObject> Dom<T> {
    /// Create a `Dom<T>` from a `&T`
    #[allow(crown::unrooted_must_root)]
    pub fn from_ref(obj: &T) -> Dom<T> {
        assert_in_script();
        Dom {
            ptr: ptr::NonNull::from(obj),
        }
    }
}

impl<T: DomObject> Deref for Dom<T> {
    type Target = T;

    fn deref(&self) -> &T {
        assert_in_script();
        // We can only have &Dom<T> from a rooted thing, so it's safe to deref
        // it to &T.
        unsafe { &*self.ptr.as_ptr() }
    }
}

unsafe impl<T: DomObject> JSTraceable for Dom<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        let trace_string;
        let trace_info = if cfg!(debug_assertions) {
            trace_string = format!("for {} on heap", ::std::any::type_name::<T>());
            &trace_string[..]
        } else {
            "for DOM object on heap"
        };
        trace_reflector(trc, trace_info, (*self.ptr.as_ptr()).reflector());
    }
}

/// A traced reference to a DOM object that may not be reflected yet.
#[crown::unrooted_must_root_lint::must_root]
pub struct MaybeUnreflectedDom<T> {
    ptr: ptr::NonNull<T>,
}

impl<T> MaybeUnreflectedDom<T>
where
    T: DomObject,
{
    #[allow(crown::unrooted_must_root)]
    pub unsafe fn from_box(value: Box<T>) -> Self {
        Self {
            ptr: Box::leak(value).into(),
        }
    }
}

impl<T> Root<MaybeUnreflectedDom<T>>
where
    T: DomObject,
{
    pub fn as_ptr(&self) -> *const T {
        self.value.ptr.as_ptr()
    }
}

impl<T> Root<MaybeUnreflectedDom<T>>
where
    T: MutDomObject,
{
    pub unsafe fn reflect_with(self, obj: *mut JSObject) -> DomRoot<T> {
        let ptr = self.as_ptr();
        drop(self);
        let root = DomRoot::from_ref(&*ptr);
        root.init_reflector(obj);
        root
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
#[crown::unrooted_must_root_lint::allow_unrooted_interior]
pub struct LayoutDom<'dom, T> {
    value: &'dom T,
}

impl<'dom, T> LayoutDom<'dom, T>
where
    T: Castable,
{
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(&self) -> LayoutDom<'dom, U>
    where
        U: Castable,
        T: DerivedFrom<U>,
    {
        assert_in_layout();
        LayoutDom {
            value: self.value.upcast::<U>(),
        }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    pub fn downcast<U>(&self) -> Option<LayoutDom<'dom, U>>
    where
        U: DerivedFrom<T>,
    {
        assert_in_layout();
        self.value.downcast::<U>().map(|value| LayoutDom { value })
    }

    /// Returns whether this inner object is a U.
    pub fn is<U>(&self) -> bool
    where
        U: DerivedFrom<T>,
    {
        assert_in_layout();
        self.value.is::<U>()
    }
}

impl<T> LayoutDom<'_, T>
where
    T: DomObject,
{
    /// Get the reflector.
    pub unsafe fn get_jsobject(&self) -> *mut JSObject {
        assert_in_layout();
        self.value.reflector().get_jsobject().get()
    }
}

impl<T> Copy for LayoutDom<'_, T> {}

impl<T> PartialEq for Dom<T> {
    fn eq(&self, other: &Dom<T>) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<'a, T: DomObject> PartialEq<&'a T> for Dom<T> {
    fn eq(&self, other: &&'a T) -> bool {
        *self == Dom::from_ref(*other)
    }
}

impl<T> Eq for Dom<T> {}

impl<T> PartialEq for LayoutDom<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.value, other.value)
    }
}

impl<T> Eq for LayoutDom<'_, T> {}

impl<T> Hash for Dom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state)
    }
}

impl<T> Hash for LayoutDom<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.value as *const T).hash(state)
    }
}

impl<T> Clone for Dom<T> {
    #[inline]
    #[allow(crown::unrooted_must_root)]
    fn clone(&self) -> Self {
        assert_in_script();
        Dom { ptr: self.ptr }
    }
}

impl<T> Clone for LayoutDom<'_, T> {
    #[inline]
    #[allow(clippy::non_canonical_clone_impl)]
    fn clone(&self) -> Self {
        assert_in_layout();
        *self
    }
}

impl LayoutDom<'_, Node> {
    /// Create a new JS-owned value wrapped from an address known to be a
    /// `Node` pointer.
    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> Self {
        assert_in_layout();
        let TrustedNodeAddress(addr) = inner;
        LayoutDom {
            value: &*(addr as *const Node),
        }
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`.  Essentially a `Cell<Dom<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[crown::unrooted_must_root_lint::must_root]
#[derive(JSTraceable)]
pub struct MutDom<T: DomObject> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject> MutDom<T> {
    /// Create a new `MutDom`.
    pub fn new(initial: &T) -> MutDom<T> {
        assert_in_script();
        MutDom {
            val: UnsafeCell::new(Dom::from_ref(initial)),
        }
    }

    /// Set this `MutDom` to the given value.
    pub fn set(&self, val: &T) {
        assert_in_script();
        unsafe {
            *self.val.get() = Dom::from_ref(val);
        }
    }

    /// Get the value in this `MutDom`.
    pub fn get(&self) -> DomRoot<T> {
        assert_in_script();
        unsafe { DomRoot::from_ref(&*ptr::read(self.val.get())) }
    }
}

impl<T: DomObject> MallocSizeOf for MutDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

impl<T: DomObject> PartialEq for MutDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.val.get() == *other.val.get() }
    }
}

impl<T: DomObject + PartialEq> PartialEq<T> for MutDom<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe { **self.val.get() == *other }
    }
}

pub(crate) fn assert_in_script() {
    debug_assert!(thread_state::get().is_script());
}

pub(crate) fn assert_in_layout() {
    debug_assert!(thread_state::get().is_layout());
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<Dom<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[crown::unrooted_must_root_lint::must_root]
#[derive(JSTraceable)]
pub struct MutNullableDom<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> MutNullableDom<T> {
    /// Create a new `MutNullableDom`.
    pub fn new(initial: Option<&T>) -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(initial.map(Dom::from_ref)),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> DomRoot<T>
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
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
    #[allow(crown::unrooted_must_root)]
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutDom<T>> {
        assert_in_layout();
        (*self.ptr.get()).as_ref().map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    #[allow(crown::unrooted_must_root)]
    pub fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { ptr::read(self.ptr.get()).map(|o| DomRoot::from_ref(&*o)) }
    }

    /// Set this `MutNullableDom` to the given value.
    pub fn set(&self, val: Option<&T>) {
        assert_in_script();
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
        unsafe { *self.ptr.get() == *other.ptr.get() }
    }
}

impl<'a, T: DomObject> PartialEq<Option<&'a T>> for MutNullableDom<T> {
    fn eq(&self, other: &Option<&T>) -> bool {
        unsafe { *self.ptr.get() == other.map(Dom::from_ref) }
    }
}

impl<T: DomObject> Default for MutNullableDom<T> {
    #[allow(crown::unrooted_must_root)]
    fn default() -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl<T: DomObject> MallocSizeOf for MutNullableDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

/// A holder that allows to lazily initialize the value only once
/// `Dom<T>`, using OnceCell
/// Essentially a `OnceCell<Dom<T>>`.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[crown::unrooted_must_root_lint::must_root]
pub struct DomOnceCell<T: DomObject> {
    ptr: OnceCell<Dom<T>>,
}

impl<T> DomOnceCell<T>
where
    T: DomObject,
{
    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    #[allow(crown::unrooted_must_root)]
    pub fn init_once<F>(&self, cb: F) -> &T
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
        self.ptr.get_or_init(|| Dom::from_ref(&cb()))
    }
}

impl<T: DomObject> Default for DomOnceCell<T> {
    #[allow(crown::unrooted_must_root)]
    fn default() -> DomOnceCell<T> {
        assert_in_script();
        DomOnceCell {
            ptr: OnceCell::new(),
        }
    }
}

impl<T: DomObject> MallocSizeOf for DomOnceCell<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

#[allow(crown::unrooted_must_root)]
unsafe impl<T: DomObject> JSTraceable for DomOnceCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if let Some(ptr) = self.ptr.get() {
            ptr.trace(trc);
        }
    }
}

impl<'dom, T> LayoutDom<'dom, T>
where
    T: 'dom + DomObject,
{
    /// Returns a reference to the interior of this JS object. The fact
    /// that this is unsafe is what necessitates the layout wrappers.
    pub fn unsafe_get(self) -> &'dom T {
        assert_in_layout();
        self.value
    }

    /// Transforms a slice of `Dom<T>` into a slice of `LayoutDom<T>`.
    // FIXME(nox): This should probably be done through a ToLayout trait.
    pub unsafe fn to_layout_slice(slice: &'dom [Dom<T>]) -> &'dom [LayoutDom<'dom, T>] {
        // This doesn't compile if Dom and LayoutDom don't have the same
        // representation.
        let _ = mem::transmute::<Dom<T>, LayoutDom<T>>;
        &*(slice as *const [Dom<T>] as *const [LayoutDom<T>])
    }
}
