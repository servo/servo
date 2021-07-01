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

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomObject, MutDomObject, Reflector, Untransplantable};
use crate::dom::bindings::trace::trace_reflector;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::node::Node;
use js::jsapi::{Heap, JSObject, JSTracer};
use js::rust::GCMethods;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use mitochondria::OnceCell;
use script_layout_interface::TrustedNodeAddress;
use std::cell::{Cell, UnsafeCell};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr;
use style::thread_state;

/// A rooted value.
#[allow(unrooted_must_root)]
#[unrooted_must_root_lint::allow_unrooted_interior]
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
    pub unsafe fn new(value: T) -> Self {
        unsafe fn add_to_root_list(object: *const dyn JSTraceable) -> *const RootCollection {
            assert_in_script();
            STACK_ROOTS.with(|ref root_list| {
                let root_list = &*root_list.get().unwrap();
                root_list.root(object);
                root_list
            })
        }

        let root_list = add_to_root_list(value.stable_trace_object());
        Root { value, root_list }
    }
}

/// Represents values that can be rooted through a stable address that will
/// not change for their whole lifetime.
pub unsafe trait StableTraceObject {
    /// Returns a stable trace object which address won't change for the whole
    /// lifetime of the value.
    fn stable_trace_object(&self) -> *const dyn JSTraceable;
}

unsafe impl<T> StableTraceObject for Dom<T>
where
    T: DomObject,
{
    fn stable_trace_object<'a>(&'a self) -> *const dyn JSTraceable {
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
        unsafe { &*(self.reflector() as *const Reflector as *const ReflectorStackRoot) }
    }
}

unsafe impl<T> StableTraceObject for MaybeUnreflectedDom<T>
where
    T: DomObject,
{
    fn stable_trace_object<'a>(&'a self) -> *const dyn JSTraceable {
        // The JSTraceable impl for Reflector doesn't actually do anything,
        // so we need this shenanigan to actually trace the reflector of the
        // T pointer in Dom<T>.
        #[allow(unrooted_must_root)]
        struct MaybeUnreflectedStackRoot<T>(T);
        unsafe impl<T> JSTraceable for MaybeUnreflectedStackRoot<T>
        where
            T: DomObject,
        {
            unsafe fn trace(&self, tracer: *mut JSTracer) {
                if self.0.reflector().get_jsobject().is_null() {
                    self.0.trace(tracer);
                } else {
                    trace_reflector(tracer, "on stack", &self.0.reflector());
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
    roots: UnsafeCell<Vec<*const dyn JSTraceable>>,
}

thread_local!(static STACK_ROOTS: Cell<Option<*const RootCollection>> = Cell::new(None));

pub struct ThreadLocalStackRoots<'a>(PhantomData<&'a u32>);

impl<'a> ThreadLocalStackRoots<'a> {
    pub fn new(roots: &'a RootCollection) -> Self {
        STACK_ROOTS.with(|ref r| r.set(Some(roots)));
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
            .rposition(|r| *r as *const () == object as *const ())
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
    STACK_ROOTS.with(|ref collection| {
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
#[unrooted_must_root_lint::must_root]
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
    pub unsafe fn to_layout(&self) -> LayoutDom<T> {
        assert_in_layout();
        LayoutDom {
            value: self.ptr.as_ref(),
        }
    }
}

impl<T: DomObject> Dom<T> {
    /// Create a Dom<T> from a &T
    #[allow(unrooted_must_root)]
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

unsafe impl<T: DomObject + Untransplantable> JSTraceable for Dom<T> {
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
#[unrooted_must_root_lint::must_root]
pub struct MaybeUnreflectedDom<T> {
    ptr: ptr::NonNull<T>,
}

impl<T> MaybeUnreflectedDom<T>
where
    T: DomObject,
{
    #[allow(unrooted_must_root)]
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
#[unrooted_must_root_lint::allow_unrooted_interior]
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
        self.value as *const T == other.value as *const T
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
    #[allow(unrooted_must_root)]
    fn clone(&self) -> Self {
        assert_in_script();
        Dom {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T> Clone for LayoutDom<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        assert_in_layout();
        LayoutDom { value: self.value }
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
#[unrooted_must_root_lint::must_root]
#[derive(JSTraceable)]
pub struct MutDom<T: DomObject + Untransplantable> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject + Untransplantable> MutDom<T> {
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

impl<T: DomObject + Untransplantable> MallocSizeOf for MutDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

impl<T: DomObject + Untransplantable> PartialEq for MutDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.val.get() == *other.val.get() }
    }
}

impl<T: DomObject + Untransplantable + PartialEq> PartialEq<T> for MutDom<T> {
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
#[unrooted_must_root_lint::must_root]
#[derive(JSTraceable)]
pub struct MutNullableDom<T: DomObject + Untransplantable> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject + Untransplantable> MutNullableDom<T> {
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
    #[allow(unrooted_must_root)]
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutDom<T>> {
        assert_in_layout();
        (*self.ptr.get()).as_ref().map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    #[allow(unrooted_must_root)]
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

impl<T: DomObject + Untransplantable> PartialEq for MutNullableDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.ptr.get() == *other.ptr.get() }
    }
}

impl<'a, T: DomObject + Untransplantable> PartialEq<Option<&'a T>> for MutNullableDom<T> {
    fn eq(&self, other: &Option<&T>) -> bool {
        unsafe { *self.ptr.get() == other.map(Dom::from_ref) }
    }
}

impl<T: DomObject + Untransplantable> Default for MutNullableDom<T> {
    #[allow(unrooted_must_root)]
    fn default() -> MutNullableDom<T> {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl<T: DomObject + Untransplantable> MallocSizeOf for MutNullableDom<T> {
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
#[unrooted_must_root_lint::must_root]
pub struct DomOnceCell<T: DomObject> {
    ptr: OnceCell<Dom<T>>,
}

impl<T> DomOnceCell<T>
where
    T: DomObject,
{
    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    #[allow(unrooted_must_root)]
    pub fn init_once<F>(&self, cb: F) -> &T
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
        &self.ptr.init_once(|| Dom::from_ref(&cb()))
    }
}

impl<T: DomObject> Default for DomOnceCell<T> {
    #[allow(unrooted_must_root)]
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

#[allow(unrooted_must_root)]
unsafe impl<T: DomObject + Untransplantable> JSTraceable for DomOnceCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if let Some(ptr) = self.ptr.as_ref() {
            ptr.trace(trc);
        }
    }
}

/// Essentially a [`MutNullableDom`], but directly references the reflector
/// so that, with a proper care, a cross-realm reference is prevented from
/// being formed by transplantation.
///
/// This type can hold a reference to an un-[`Untransplantable`] DOM object.
/// In turn, such objects also need this sort of type to hold references to
/// other DOM objects whether they are transplantable or not (so the name is
/// inaccurate, actually).
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[unrooted_must_root_lint::must_root]
pub struct MutNullableTransplantableDom<T: DomObject> {
    /// A reference to the DOM object.
    ptr: UnsafeCell<Option<ptr::NonNull<T>>>,
    /// A tracable reference to the reflector.
    reflector: Heap<*mut JSObject>,
}

impl<T: DomObject> MutNullableTransplantableDom<T> {
    /// Create a new `MutNullableTransplantableDom`.
    ///
    /// # Safety
    ///
    /// The constructed `MutNullableTransplantableDom` must be pinned before
    /// use.
    ///
    /// FIXME: `std::pin::Pin` might be able to express this better
    pub unsafe fn new() -> MutNullableTransplantableDom<T> {
        assert_in_script();
        MutNullableTransplantableDom {
            ptr: UnsafeCell::new(None),
            reflector: Heap::default(),
        }
    }

    /// Get a rooted DOM object out of this object.
    #[allow(unrooted_must_root)]
    pub fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { ptr::read(self.ptr.get()).map(|o| DomRoot::from_ref(o.as_ref())) }
    }

    /// Set this `MutNullableTransplantableDom` to the given value. The
    /// reflector will be wrapped for `global`'s realm.
    pub fn set(&self, val: Option<&T>, global: &crate::dom::globalscope::GlobalScope) {
        assert_in_script();
        unsafe {
            if let Some(dom) = val {
                let cx = global.get_cx();
                let _ac = crate::realms::enter_realm(global);
                rooted!(in(*cx) let mut reflector = *dom.reflector().get_jsobject());
                js::jsapi::JS_WrapObject(*cx, reflector.handle_mut().into());
                self.reflector.set(reflector.get());
            } else {
                self.reflector.set(std::ptr::null_mut());
            }

            *self.ptr.get() = val.map(Into::into);
        }
    }
}

unsafe impl<T: DomObject> JSTraceable for MutNullableTransplantableDom<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.reflector.trace(trc);
    }
}

impl<T: DomObject> MallocSizeOf for MutNullableTransplantableDom<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

/// Essentially a [`DomOnceCell`], but directly references the reflector
/// so that, with a proper care, a cross-realm reference is prevented from
/// being formed by transplantation.
///
/// This type can hold a reference to an un-[`Untransplantable`] DOM object.
/// In turn, such objects also need this sort of type to hold references to
/// other DOM objects whether they are transplantable or not (so the name is
/// inaccurate, actually).
///
/// This should only be used as a field in other DOM objects; see warning
/// on `Dom<T>`.
#[unrooted_must_root_lint::must_root]
pub struct TransplantableDomOnceCell<T: DomObject> {
    /// A reference to the DOM object.
    ptr: OnceCell<ptr::NonNull<T>>,
    /// A tracable reference to the reflector.
    ///
    /// Invariant: `reflector` points to `ptr.reflector()` or its CCW.
    reflector: Heap<*mut JSObject>,
}

impl<T: DomObject> TransplantableDomOnceCell<T> {
    /// Create a new `TransplantableDomOnceCell`.
    ///
    /// # Safety
    ///
    /// The constructed `TransplantableDomOnceCell` must be pinned before
    /// use.
    ///
    /// FIXME: `std::pin::Pin` might be able to express this better
    pub unsafe fn new() -> TransplantableDomOnceCell<T> {
        assert_in_script();
        TransplantableDomOnceCell {
            ptr: OnceCell::new(),
            reflector: Heap::default(),
        }
    }

    // FIXME: The compartment invariants will be violated if an incorrect global
    //        scope is supplied. Should this method be `unsafe fn` because for
    //        this reason, or shouldn't it be because there are currently
    //        gazillions of other non-`unsafe` ways (`find_document` for one) to
    //        obtain other realms' DOM objects?
    /// Set this `TransplantableDomOnceCell` to the given value. The
    /// reflector will be wrapped for `global`'s realm. Does nothing if it's
    /// already set.
    ///
    /// # Errors
    ///
    /// This method returns `Ok(())` if the cell was empty and `Err(())` if
    /// it was full.
    pub fn set<'a>(
        &self,
        val: Option<&T>,
        global: &crate::dom::globalscope::GlobalScope,
    ) -> Result<(), ()> {
        assert_in_script();

        if self.ptr.as_ref().is_some() {
            return Err(());
        }

        if let Some(dom) = val {
            self.ptr.init_once(|| {
                // We've already checked the emptiness of `self.ptr`
                debug_assert!(self.ptr.as_ref().is_none());

                // Initialize `self.reflector`.
                let cx = global.get_cx();
                let _ac = crate::realms::enter_realm(global);
                rooted!(in(*cx) let mut reflector = *dom.reflector().get_jsobject());
                unsafe { js::jsapi::JS_WrapObject(*cx, reflector.handle_mut().into()) };

                // The above code isn't supposed to initialize `self` reentrantly
                assert!(self.reflector.get().is_null());
                self.reflector.set(reflector.get());

                dom.into()
            });
        }
        Ok(())
    }

    /// Get a reference to the DOM object.
    pub fn as_ref(&self) -> Option<&T> {
        self.ptr.as_ref().map(|ptr| unsafe { &*ptr.as_ptr() })
    }

    /// Rewrap the reflector with a new realm.
    pub fn rewrap(&self, global: &crate::dom::globalscope::GlobalScope) {
        if self.reflector.get().is_null() {
            return;
        }
        let cx = global.get_cx();
        let _ac = crate::realms::enter_realm(global);
        rooted!(in(*cx) let mut reflector = self.reflector.get());
        unsafe { js::jsapi::JS_WrapObject(*cx, reflector.handle_mut().into()) };
        self.reflector.set(reflector.get());
    }
}

unsafe impl<T: DomObject> JSTraceable for TransplantableDomOnceCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.reflector.trace(trc);
    }
}

impl<T: DomObject> MallocSizeOf for TransplantableDomOnceCell<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // See comment on MallocSizeOf for Dom<T>.
        0
    }
}

impl<'dom, T> LayoutDom<'dom, T>
where
    T: 'dom + DomObject,
{
    /// Returns a reference to the interior of this JS object. The fact
    /// that this is unsafe is what necessitates the layout wrappers.
    pub unsafe fn unsafe_get(self) -> &'dom T {
        assert_in_layout();
        self.value
    }

    /// Transforms a slice of Dom<T> into a slice of LayoutDom<T>.
    // FIXME(nox): This should probably be done through a ToLayout trait.
    pub unsafe fn to_layout_slice(slice: &'dom [Dom<T>]) -> &'dom [LayoutDom<'dom, T>] {
        // This doesn't compile if Dom and LayoutDom don't have the same
        // representation.
        let _ = mem::transmute::<Dom<T>, LayoutDom<T>>;
        &*(slice as *const [Dom<T>] as *const [LayoutDom<T>])
    }
}

/// Helper trait for safer manipulations of `Option<Heap<T>>` values.
pub trait OptionalHeapSetter {
    type Value;
    /// Update this optional heap value with a new value.
    fn set(&mut self, v: Option<Self::Value>);
}

impl<T: GCMethods + Copy> OptionalHeapSetter for Option<Heap<T>>
where
    Heap<T>: Default,
{
    type Value = T;
    fn set(&mut self, v: Option<T>) {
        let v = match v {
            None => {
                *self = None;
                return;
            },
            Some(v) => v,
        };

        if self.is_none() {
            *self = Some(Heap::default());
        }

        self.as_ref().unwrap().set(v);
    }
}
