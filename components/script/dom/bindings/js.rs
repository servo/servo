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
//! - `JSRef<T>`: a freely-copyable reference to a rooted DOM object.
//! - `Root<T>`: a stack-based reference to a rooted DOM object.
//! - `JS<T>`: a reference to a DOM object that can automatically be traced by
//!   the GC when encountered as a field of a Rust structure.
//! - `Temporary<T>`: a reference to a DOM object that will remain rooted for
//!   the duration of its lifetime.
//!
//! The rule of thumb is as follows:
//!
//! - All methods return `Temporary<T>`, to ensure the value remains alive
//!   until it is stored somewhere that is reachable by the GC.
//! - All functions take `JSRef<T>` arguments, to ensure that they will remain
//!   uncollected for the duration of their usage.
//! - All DOM structs contain `JS<T>` fields and derive the `JSTraceable`
//!   trait, to ensure that they are transitively marked as reachable by the GC
//!   if the enclosing value is reachable.
//! - All methods for type `T` are implemented for `JSRef<T>`, to ensure that
//!   the self value will not be collected for the duration of the method call.
//!
//! Both `Temporary<T>` and `JS<T>` do not allow access to their inner value
//! without explicitly creating a stack-based root via the `root` method
//! through the `Rootable<T>` trait. This returns a `Root<T>`, which causes the
//! JS-owned value to be uncollectable for the duration of the `Root` object's
//! lifetime. A `JSRef<T>` can be obtained from a `Root<T>` by calling the `r`
//! method. These `JSRef<T>` values are not allowed to outlive their
//! originating `Root<T>`, to ensure that all interactions with the enclosed
//! value only occur when said value is uncollectable, and will cause static
//! lifetime errors if misused.
//!
//! Other miscellaneous helper traits:
//!
//! - `OptionalRootable` and `OptionalOptionalRootable`: make rooting `Option`
//!   values easy via a `root` method
//! - `ResultRootable`: make rooting successful `Result` values easy
//! - `TemporaryPushable`: allows mutating vectors of `JS<T>` with new elements
//!   of `JSRef`/`Temporary`
//! - `RootedReference`: makes obtaining an `Option<JSRef<T>>` from an
//!   `Option<Root<T>>` easy

use dom::bindings::trace::JSTraceable;
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::node::Node;
use js::jsapi::JSObject;
use js::jsval::JSVal;
use layout_interface::TrustedNodeAddress;
use script_task::STACK_ROOTS;

use core::nonzero::NonZero;
use libc;
use std::cell::{Cell, UnsafeCell};
use std::default::Default;
use std::intrinsics::return_address;
use std::marker::PhantomData;
use std::ops::Deref;

/// An unrooted, JS-owned value. Must not be held across a GC.
///
/// This is used in particular to wrap pointers extracted from a reflector.
#[must_root]
pub struct Unrooted<T> {
    ptr: NonZero<*const T>
}

impl<T: Reflectable> Unrooted<T> {
    /// Create a new JS-owned value wrapped from a raw Rust pointer.
    pub unsafe fn from_raw(raw: *const T) -> Unrooted<T> {
        assert!(!raw.is_null());
        Unrooted {
            ptr: NonZero::new(raw)
        }
    }

    /// Create a new unrooted value from a `JS<T>`.
    #[allow(unrooted_must_root)]
    pub fn from_js(ptr: JS<T>) -> Unrooted<T> {
        Unrooted {
            ptr: ptr.ptr
        }
    }

    /// Create a new unrooted value from a `Temporary<T>`.
    #[allow(unrooted_must_root)]
    pub fn from_temporary(ptr: Temporary<T>) -> Unrooted<T> {
        Unrooted::from_js(ptr.inner)
    }

    /// Get the `Reflector` for this pointer.
    pub fn reflector<'a>(&'a self) -> &'a Reflector {
        unsafe {
            (**self.ptr).reflector()
        }
    }

    /// Returns an unsafe pointer to the interior of this object.
    pub unsafe fn unsafe_get(&self) -> *const T {
        *self.ptr
    }
}

impl<T: Reflectable> Rootable<T> for Unrooted<T> {
    /// Create a stack-bounded root for this value.
    fn root(&self) -> Root<T> {
        STACK_ROOTS.with(|ref collection| {
            let RootCollectionPtr(collection) = collection.get().unwrap();
            unsafe {
                Root::new(&*collection, self.ptr)
            }
        })
    }
}

impl<T> Copy for Unrooted<T> {}
impl<T> Clone for Unrooted<T> {
    fn clone(&self) -> Unrooted<T> { *self }
}

/// A type that represents a JS-owned value that is rooted for the lifetime of
/// this value. Importantly, it requires explicit rooting in order to interact
/// with the inner value. Can be assigned into JS-owned member fields (i.e.
/// `JS<T>` types) safely via the `JS<T>::assign` method or
/// `OptionalSettable::assign` (for `Option<JS<T>>` fields).
#[allow(unrooted_must_root)]
pub struct Temporary<T> {
    inner: JS<T>,
    /// On-stack JS pointer to assuage conservative stack scanner
    _js_ptr: *mut JSObject,
}

impl<T> Clone for Temporary<T> {
    fn clone(&self) -> Temporary<T> {
        Temporary {
            inner: self.inner,
            _js_ptr: self._js_ptr,
        }
    }
}

impl<T> PartialEq for Temporary<T> {
    fn eq(&self, other: &Temporary<T>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Reflectable> Temporary<T> {
    /// Create a new `Temporary` value from an unrooted value.
    #[allow(unrooted_must_root)]
    pub fn from_unrooted(unrooted: Unrooted<T>) -> Temporary<T> {
        Temporary {
            inner: JS { ptr: unrooted.ptr },
            _js_ptr: unrooted.reflector().get_jsobject(),
        }
    }

    /// Create a new `Temporary` value from a rooted value.
    #[allow(unrooted_must_root)]
    pub fn from_rooted<U: Assignable<T>>(root: U) -> Temporary<T> {
        let inner = JS::from_rooted(root);
        Temporary {
            inner: inner,
            _js_ptr: inner.reflector().get_jsobject(),
        }
    }
}

impl<T: Reflectable> Rootable<T> for Temporary<T> {
    /// Create a stack-bounded root for this value.
    fn root(&self) -> Root<T> {
        self.inner.root()
    }
}

/// A traced reference to a DOM object. Must only be used as a field in other
/// DOM objects.
#[must_root]
pub struct JS<T> {
    ptr: NonZero<*const T>
}

impl<T> JS<T> {
    /// Returns `LayoutJS<T>` containing the same pointer.
    pub unsafe fn to_layout(self) -> LayoutJS<T> {
        LayoutJS {
            ptr: self.ptr.clone()
        }
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
pub struct LayoutJS<T> {
    ptr: NonZero<*const T>
}

impl<T: Reflectable> LayoutJS<T> {
    /// Get the reflector.
    pub unsafe fn get_jsobject(&self) -> *mut JSObject {
        (**self.ptr).reflector().get_jsobject()
    }
}

impl<T> Copy for JS<T> {}

impl<T> Copy for LayoutJS<T> {}

impl<T> PartialEq for JS<T> {
    #[allow(unrooted_must_root)]
    fn eq(&self, other: &JS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> PartialEq for LayoutJS<T> {
    #[allow(unrooted_must_root)]
    fn eq(&self, other: &LayoutJS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl <T> Clone for JS<T> {
    #[inline]
    fn clone(&self) -> JS<T> {
        JS {
            ptr: self.ptr.clone()
        }
    }
}

impl <T> Clone for LayoutJS<T> {
    #[inline]
    fn clone(&self) -> LayoutJS<T> {
        LayoutJS {
            ptr: self.ptr.clone()
        }
    }
}

impl LayoutJS<Node> {
    /// Create a new JS-owned value wrapped from an address known to be a
    /// `Node` pointer.
    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress)
                                            -> LayoutJS<Node> {
        let TrustedNodeAddress(addr) = inner;
        LayoutJS {
            ptr: NonZero::new(addr as *const Node)
        }
    }
}

impl<T: Reflectable> Rootable<T> for JS<T> {
    /// Root this JS-owned value to prevent its collection as garbage.
    fn root(&self) -> Root<T> {
        STACK_ROOTS.with(|ref collection| {
            let RootCollectionPtr(collection) = collection.get().unwrap();
            unsafe {
                Root::new(&*collection, self.ptr)
            }
        })
    }
}

impl<U: Reflectable> JS<U> {
    /// Create a `JS<T>` from any JS-managed pointer.
    pub fn from_rooted<T: Assignable<U>>(root: T) -> JS<U> {
        unsafe {
            root.get_js()
        }
    }
}

//XXXjdm This is disappointing. This only gets called from trace hooks, in theory,
//       so it's safe to assume that self is rooted and thereby safe to access.
impl<T: Reflectable> Reflectable for JS<T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        unsafe {
            (**self.ptr).reflector()
        }
    }
}

/// A trait to be implemented for JS-managed types that can be stored in
/// mutable member fields.
///
/// Do not implement this trait yourself.
pub trait HeapGCValue: JSTraceable {
}

impl HeapGCValue for JSVal {
}

impl<T: Reflectable> HeapGCValue for JS<T> {
}

/// A holder that provides interior mutability for GC-managed values such as
/// `JSVal` and `JS<T>`.
///
/// Must be used in place of traditional interior mutability to ensure proper
/// GC barriers are enforced.
#[must_root]
#[jstraceable]
pub struct MutHeap<T: HeapGCValue+Copy> {
    val: Cell<T>,
}

impl<T: HeapGCValue+Copy> MutHeap<T> {
    /// Create a new `MutHeap`.
    pub fn new(initial: T) -> MutHeap<T> {
        MutHeap {
            val: Cell::new(initial),
        }
    }

    /// Set this `MutHeap` to the given value, calling write barriers as
    /// appropriate.
    pub fn set(&self, val: T) {
        self.val.set(val)
    }

    /// Set the value in this `MutHeap`, calling read barriers as appropriate.
    pub fn get(&self) -> T {
        self.val.get()
    }
}

/// A mutable holder for GC-managed values such as `JSval` and `JS<T>`, with
/// nullability represented by an enclosing Option wrapper. Must be used in
/// place of traditional internal mutability to ensure that the proper GC
/// barriers are enforced.
#[must_root]
#[jstraceable]
pub struct MutNullableHeap<T: HeapGCValue+Copy> {
    ptr: Cell<Option<T>>
}

impl<T: HeapGCValue+Copy> MutNullableHeap<T> {
    /// Create a new `MutNullableHeap`.
    pub fn new(initial: Option<T>) -> MutNullableHeap<T> {
        MutNullableHeap {
            ptr: Cell::new(initial)
        }
    }

    /// Set this `MutNullableHeap` to the given value, calling write barriers
    /// as appropriate.
    pub fn set(&self, val: Option<T>) {
        self.ptr.set(val);
    }

    /// Retrieve a copy of the current optional inner value.
    pub fn get(&self) -> Option<T> {
        self.ptr.get()
    }
}

impl<T: Reflectable> MutNullableHeap<JS<T>> {
    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> Temporary<T>
        where F: FnOnce() -> Temporary<T>
    {
        match self.get() {
            Some(inner) => Temporary::from_rooted(inner),
            None => {
                let inner = cb();
                self.set(Some(JS::from_rooted(inner.clone())));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `JS<T>` as `LayoutJS<T>`.
    /// For use by layout, which can't use safe types like Temporary.
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutJS<T>> {
        self.ptr.get().map(|js| js.to_layout())
    }
}

impl<T: HeapGCValue+Copy> Default for MutNullableHeap<T> {
    fn default() -> MutNullableHeap<T> {
        MutNullableHeap {
            ptr: Cell::new(None)
        }
    }
}

impl<T: Reflectable> JS<T> {
    /// Store an unrooted value in this field. This is safe under the
    /// assumption that JS<T> values are only used as fields in DOM types that
    /// are reachable in the GC graph, so this unrooted value becomes
    /// transitively rooted for the lifetime of its new owner.
    pub fn assign(&mut self, val: Temporary<T>) {
        *self = val.inner.clone();
    }
}

impl<T: Reflectable> LayoutJS<T> {
    /// Returns an unsafe pointer to the interior of this JS object. This is
    /// the only method that be safely accessed from layout. (The fact that
    /// this is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *const T {
        *self.ptr
    }
}

/// Get an `Option<JSRef<T>>` out of an `Option<Root<T>>`
pub trait RootedReference<T> {
    /// Obtain a safe optional reference to the wrapped JS owned-value that
    /// cannot outlive the lifetime of this root.
    fn r<'a>(&'a self) -> Option<JSRef<'a, T>>;
}

impl<T: Reflectable> RootedReference<T> for Option<Root<T>> {
    fn r<'a>(&'a self) -> Option<JSRef<'a, T>> {
        self.as_ref().map(|root| root.r())
    }
}

/// Get an `Option<Option<JSRef<T>>>` out of an `Option<Option<Root<T>>>`
pub trait OptionalRootedReference<T> {
    /// Obtain a safe optional optional reference to the wrapped JS owned-value
    /// that cannot outlive the lifetime of this root.
    fn r<'a>(&'a self) -> Option<Option<JSRef<'a, T>>>;
}

impl<T: Reflectable> OptionalRootedReference<T> for Option<Option<Root<T>>> {
    fn r<'a>(&'a self) -> Option<Option<JSRef<'a, T>>> {
        self.as_ref().map(|inner| inner.r())
    }
}

/// Trait that allows extracting a `JS<T>` value from a variety of
/// rooting-related containers, which in general is an unsafe operation since
/// they can outlive the rooted lifetime of the original value.
pub trait Assignable<T> {
    /// Extract an unrooted `JS<T>`.
    unsafe fn get_js(&self) -> JS<T>;
}

impl<T> Assignable<T> for JS<T> {
    unsafe fn get_js(&self) -> JS<T> {
        self.clone()
    }
}

impl<'a, T: Reflectable> Assignable<T> for JSRef<'a, T> {
    unsafe fn get_js(&self) -> JS<T> {
        JS {
            ptr: self.ptr
        }
    }
}

impl<T: Reflectable> Assignable<T> for Temporary<T> {
    unsafe fn get_js(&self) -> JS<T> {
        self.inner.clone()
    }
}


/// Root a rootable `Option` type (used for `Option<Temporary<T>>`)
pub trait OptionalRootable<T> {
    /// Root the inner value, if it exists.
    fn root(&self) -> Option<Root<T>>;
}

impl<T: Reflectable, U: Rootable<T>> OptionalRootable<T> for Option<U> {
    fn root(&self) -> Option<Root<T>> {
        self.as_ref().map(|inner| inner.root())
    }
}

/// Root a rootable `Option<Option>` type (used for `Option<Option<JS<T>>>`)
pub trait OptionalOptionalRootable<T> {
    /// Root the inner value, if it exists.
    fn root(&self) -> Option<Option<Root<T>>>;
}

impl<T: Reflectable, U: OptionalRootable<T>> OptionalOptionalRootable<T> for Option<U> {
    fn root(&self) -> Option<Option<Root<T>>> {
        self.as_ref().map(|inner| inner.root())
    }
}

/// Root a rootable `Result` type (any of `Temporary<T>` or `JS<T>`)
pub trait ResultRootable<T,U> {
    /// Root the inner value, if it exists.
    fn root(self) -> Result<Root<T>, U>;
}

impl<T: Reflectable, U, V: Rootable<T>> ResultRootable<T, U> for Result<V, U> {
    fn root(self) -> Result<Root<T>, U> {
        self.map(|inner| inner.root())
    }
}

/// Root a rootable type.
pub trait Rootable<T> {
    /// Root the value.
    fn root(&self) -> Root<T>;
}


/// Provides a facility to push unrooted values onto lists of rooted values.
/// This is safe under the assumption that said lists are reachable via the GC
/// graph, and therefore the new values are transitively rooted for the
/// lifetime of their new owner.
pub trait TemporaryPushable<T> {
    /// Push a new value onto this container.
    fn push_unrooted(&mut self, val: &T);
    /// Insert a new value into this container.
    fn insert_unrooted(&mut self, index: usize, val: &T);
}

impl<T: Assignable<U>, U: Reflectable> TemporaryPushable<T> for Vec<JS<U>> {
    fn push_unrooted(&mut self, val: &T) {
        self.push(unsafe { val.get_js() });
    }

    fn insert_unrooted(&mut self, index: usize, val: &T) {
        self.insert(index, unsafe { val.get_js() });
    }
}

/// An opaque, LIFO rooting mechanism. This tracks roots and ensures that they
/// are destructed in a LIFO order.
///
/// See also [*Exact Stack Rooting - Storing a GCPointer on the CStack*]
/// (https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting).
#[no_move]
pub struct RootCollection {
    roots: UnsafeCell<RootedVec<*mut JSObject>>,
}

/// A pointer to a RootCollection, for use in global variables.
pub struct RootCollectionPtr(pub *const RootCollection);

impl Copy for RootCollectionPtr {}
impl Clone for RootCollectionPtr {
    fn clone(&self) -> RootCollectionPtr { *self }
}

impl RootCollection {
    /// Create an empty collection of roots
    pub fn new() -> RootCollection {
        let addr = unsafe {
            return_address() as *const libc::c_void
        };

        RootCollection {
            roots: UnsafeCell::new(RootedVec::new_with_destination_address(addr)),
        }
    }

    /// Track a stack-based root as a pointer to ensure LIFO root ordering.
    fn root<'b>(&self, untracked_js_ptr: *mut JSObject) {
        unsafe {
            let roots = self.roots.get();
            (*roots).push(untracked_js_ptr);
            debug!("  rooting {:?}", untracked_js_ptr);
        }
    }

    /// Stop tracking a stack-based root, asserting if LIFO root ordering has
    /// been violated
    fn unroot<'b, T: Reflectable>(&self, rooted: &Root<T>) {
        unsafe {
            let roots = self.roots.get();
            let unrooted = (*roots).pop().unwrap();
            debug!("unrooted {:?} (expecting {:?}", unrooted, rooted.js_ptr);
            assert!(unrooted == rooted.js_ptr);
        }
    }
}

/// A rooted reference to a DOM object.
///
/// The JS value is pinned for the duration of this object's lifetime; roots
/// are additive, so this object's destruction will not invalidate other roots
/// for the same JS value. `Root`s cannot outlive the associated
/// `RootCollection` object. Attempts to transfer ownership of a `Root` via
/// moving will trigger dynamic unrooting failures due to incorrect ordering.
#[no_move]
pub struct Root<T: Reflectable> {
    /// List that ensures correct dynamic root ordering
    root_list: &'static RootCollection,
    /// Reference to rooted value that must not outlive this container
    ptr: NonZero<*const T>,
    /// On-stack JS pointer to assuage conservative stack scanner
    js_ptr: *mut JSObject,
}

impl<T: Reflectable> Root<T> {
    /// Create a new stack-bounded root for the provided JS-owned value.
    /// It cannot not outlive its associated `RootCollection`, and it contains
    /// a `JSRef` which cannot outlive this new `Root`.
    #[inline]
    fn new(roots: &'static RootCollection, unrooted: NonZero<*const T>)
           -> Root<T> {
        let js_ptr = unsafe {
            (**unrooted).reflector().get_jsobject()
        };
        roots.root(js_ptr);
        Root {
            root_list: roots,
            ptr: unrooted,
            js_ptr: js_ptr,
        }
    }

    /// Obtain a safe reference to the wrapped JS owned-value that cannot
    /// outlive the lifetime of this root.
    pub fn r<'b>(&'b self) -> JSRef<'b, T> {
        JSRef {
            ptr: self.ptr,
            chain: PhantomData,
        }
    }

    /// Obtain an unsafe reference to the wrapped JS owned-value that can
    /// outlive the lifetime of this root.
    ///
    /// DO NOT CALL.
    pub fn get_unsound_ref_forever<'b>(&self) -> JSRef<'b, T> {
        JSRef {
            ptr: self.ptr,
            chain: PhantomData,
        }
    }
}

impl<T: Reflectable> Drop for Root<T> {
    fn drop(&mut self) {
        self.root_list.unroot(self);
    }
}

impl<'a, T: Reflectable> Deref for JSRef<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        unsafe {
            &**self.ptr
        }
    }
}

/// A reference to a DOM object that is guaranteed to be alive. This is freely
/// copyable.
pub struct JSRef<'a, T> {
    ptr: NonZero<*const T>,
    chain: PhantomData<&'a ()>,
}

impl<'a, T> Copy for JSRef<'a, T> {}

impl<'a, T> Clone for JSRef<'a, T> {
    fn clone(&self) -> JSRef<'a, T> {
        JSRef {
            ptr: self.ptr.clone(),
            chain: self.chain,
        }
    }
}

impl<'a, 'b, T> PartialEq<JSRef<'b, T>> for JSRef<'a, T> {
    fn eq(&self, other: &JSRef<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a, T: Reflectable> JSRef<'a, T> {
    /// Returns the inner pointer directly.
    pub fn extended_deref(self) -> &'a T {
        unsafe {
            &**self.ptr
        }
    }
}

impl<'a, T: Reflectable> Reflectable for JSRef<'a, T> {
    fn reflector<'b>(&'b self) -> &'b Reflector {
        (**self).reflector()
    }
}
