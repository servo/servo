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
//! - `Root<T>`: a stack-based reference to a rooted DOM object.
//! - `JS<T>`: a reference to a DOM object that can automatically be traced by
//!   the GC when encountered as a field of a Rust structure.
//!
//! `JS<T>` does not allow access to their inner value without explicitly
//! creating a stack-based root via the `root` method. This returns a `Root<T>`,
//! which causes the JS-owned value to be uncollectable for the duration of the
//! `Root` object's lifetime. A reference to the object can then be obtained
//! from the `Root` object. These references are not allowed to outlive their
//! originating `Root<T>`.
//!

use core::nonzero::NonZero;
use dom::bindings::conversions::{Castable, DerivedFrom};
use dom::bindings::trace::JSTraceable;
use dom::bindings::trace::trace_reflector;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::node::Node;
use js::jsapi::{Heap, JSObject, JSTracer};
use js::jsval::JSVal;
use layout_interface::TrustedNodeAddress;
use script_task::STACK_ROOTS;
use std::cell::UnsafeCell;
use std::default::Default;
use std::mem;
use std::ops::Deref;
use std::ptr;
use util::mem::HeapSizeOf;

/// A traced reference to a DOM object
///
/// This type is critical to making garbage collection work with the DOM,
/// but it is very dangerous; if garbage collection happens with a `JS<T>`
/// on the stack, the `JS<T>` can point to freed memory.
///
/// This should only be used as a field in other DOM objects.
#[must_root]
pub struct JS<T> {
    ptr: NonZero<*const T>
}

// JS<T> is similar to Rc<T>, in that it's not always clear how to avoid double-counting.
// For now, we choose not to follow any such pointers.
impl<T> HeapSizeOf for JS<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T> JS<T> {
    /// Returns `LayoutJS<T>` containing the same pointer.
    pub unsafe fn to_layout(&self) -> LayoutJS<T> {
        LayoutJS {
            ptr: self.ptr.clone()
        }
    }
}

impl<T: Reflectable> JS<T> {
    /// Root this JS-owned value to prevent its collection as garbage.
    pub fn root(&self) -> Root<T> {
        Root::new(self.ptr)
    }
    /// Create a JS<T> from a Root<T>
    /// XXX Not a great API. Should be a call on Root<T> instead
    #[allow(unrooted_must_root)]
    pub fn from_rooted(root: &Root<T>) -> JS<T> {
        JS {
            ptr: unsafe { NonZero::new(&**root) }
        }
    }
    /// Create a JS<T> from a &T
    #[allow(unrooted_must_root)]
    pub fn from_ref(obj: &T) -> JS<T> {
        JS {
            ptr: unsafe { NonZero::new(&*obj) }
        }
    }
}

impl<T: Reflectable> Deref for JS<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // We can only have &JS<T> from a rooted thing, so it's safe to deref
        // it to &T.
        unsafe { &**self.ptr }
    }
}

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", unsafe { (**self.ptr).reflector() });
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
#[allow_unrooted_interior]
pub struct LayoutJS<T> {
    ptr: NonZero<*const T>
}

impl<T: Castable> LayoutJS<T> {
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(&self) -> LayoutJS<U> where U: Castable, T: DerivedFrom<U> {
        unsafe { mem::transmute_copy(self) }
    }

    /// Cast a DOM object downwards to one of the interfaces it might implement.
    pub fn downcast<U>(&self) -> Option<LayoutJS<U>> where U: DerivedFrom<T> {
        unsafe {
            if (*self.unsafe_get()).is::<U>() {
                Some(mem::transmute_copy(self))
            } else {
                None
            }
        }
    }
}

impl<T: Reflectable> LayoutJS<T> {
    /// Get the reflector.
    pub unsafe fn get_jsobject(&self) -> *mut JSObject {
        (**self.ptr).reflector().get_jsobject().get()
    }
}

impl<T> Copy for LayoutJS<T> {}

impl<T> PartialEq for JS<T> {
    fn eq(&self, other: &JS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> PartialEq for LayoutJS<T> {
    fn eq(&self, other: &LayoutJS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl <T> Clone for JS<T> {
    #[inline]
    #[allow(unrooted_must_root)]
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


/// A trait to be implemented for JS-managed types that can be stored in
/// mutable member fields.
///
/// Do not implement this trait yourself.
pub trait HeapGCValue: JSTraceable {
}

impl HeapGCValue for Heap<JSVal> {
}

impl<T: Reflectable> HeapGCValue for JS<T> {
}

/// A holder that provides interior mutability for GC-managed JSVals.
///
/// Must be used in place of traditional interior mutability to ensure proper
/// GC barriers are enforced.
#[must_root]
#[derive(JSTraceable)]
pub struct MutHeapJSVal {
    val: UnsafeCell<Heap<JSVal>>,
}

impl MutHeapJSVal {
    /// Create a new `MutHeapJSVal`.
    pub fn new() -> MutHeapJSVal {
        MutHeapJSVal {
            val: UnsafeCell::new(Heap::default()),
        }
    }

    /// Set this `MutHeapJSVal` to the given value, calling write barriers as
    /// appropriate.
    pub fn set(&self, val: JSVal) {
        unsafe {
            let cell = self.val.get();
            (*cell).set(val);
        }
    }

    /// Get the value in this `MutHeapJSVal`, calling read barriers as appropriate.
    pub fn get(&self) -> JSVal {
        unsafe { (*self.val.get()).get() }
    }
}


/// A holder that provides interior mutability for GC-managed values such as
/// `JS<T>`.  Essentially a `Cell<JS<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `JS<T>`.
#[must_root]
#[derive(JSTraceable)]
pub struct MutHeap<T: HeapGCValue> {
    val: UnsafeCell<T>,
}

impl<T: Reflectable> MutHeap<JS<T>> {
    /// Create a new `MutHeap`.
    pub fn new(initial: &T) -> MutHeap<JS<T>> {
        MutHeap {
            val: UnsafeCell::new(JS::from_ref(initial)),
        }
    }

    /// Set this `MutHeap` to the given value.
    pub fn set(&self, val: &T) {
        unsafe {
            *self.val.get() = JS::from_ref(val);
        }
    }

    /// Get the value in this `MutHeap`.
    pub fn get(&self) -> Root<T> {
        unsafe {
            ptr::read(self.val.get()).root()
        }
    }
}

impl<T: HeapGCValue> HeapSizeOf for MutHeap<T> {
    fn heap_size_of_children(&self) -> usize {
        // See comment on HeapSizeOf for JS<T>.
        0
    }
}

/// A holder that provides interior mutability for GC-managed values such as
/// `JS<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<JS<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects; see warning
/// on `JS<T>`.
#[must_root]
#[derive(JSTraceable)]
pub struct MutNullableHeap<T: HeapGCValue> {
    ptr: UnsafeCell<Option<T>>
}

impl<T: Reflectable> MutNullableHeap<JS<T>> {
    /// Create a new `MutNullableHeap`.
    pub fn new(initial: Option<&T>) -> MutNullableHeap<JS<T>> {
        MutNullableHeap {
            ptr: UnsafeCell::new(initial.map(JS::from_ref))
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> Root<T>
        where F: FnOnce() -> Root<T>
    {
        match self.get() {
            Some(inner) => inner,
            None => {
                let inner = cb();
                self.set(Some(&inner));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `JS<T>` as `LayoutJS<T>`.
    /// For use by layout, which can't use safe types like Temporary.
    #[allow(unrooted_must_root)]
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutJS<T>> {
        ptr::read(self.ptr.get()).map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    #[allow(unrooted_must_root)]
    pub fn get(&self) -> Option<Root<T>> {
        unsafe {
            ptr::read(self.ptr.get()).map(|o| o.root())
        }
    }

    /// Get a rooted value out of this object
    pub fn get_rooted(&self) -> Option<Root<T>> {
        self.get()
    }

    /// Set this `MutNullableHeap` to the given value.
    pub fn set(&self, val: Option<&T>) {
        unsafe {
            *self.ptr.get() = val.map(|p| JS::from_ref(p));
        }
    }
}

impl<T: HeapGCValue> Default for MutNullableHeap<T> {
    #[allow(unrooted_must_root)]
    fn default() -> MutNullableHeap<T> {
        MutNullableHeap {
            ptr: UnsafeCell::new(None)
        }
    }
}

impl<T: HeapGCValue> HeapSizeOf for MutNullableHeap<T> {
    fn heap_size_of_children(&self) -> usize {
        // See comment on HeapSizeOf for JS<T>.
        0
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
    fn r(&self) -> Option<&T>;
}

impl<T: Reflectable> RootedReference<T> for Option<Root<T>> {
    fn r(&self) -> Option<&T> {
        self.as_ref().map(|root| root.r())
    }
}

/// Get an `Option<Option<&T>>` out of an `Option<Option<Root<T>>>`
pub trait OptionalRootedReference<T> {
    /// Obtain a safe optional optional reference to the wrapped JS owned-value
    /// that cannot outlive the lifetime of this root.
    fn r(&self) -> Option<Option<&T>>;
}

impl<T: Reflectable> OptionalRootedReference<T> for Option<Option<Root<T>>> {
    fn r(&self) -> Option<Option<&T>> {
        self.as_ref().map(|inner| inner.r())
    }
}

/// A rooting mechanism for reflectors on the stack.
/// LIFO is not required.
///
/// See also [*Exact Stack Rooting - Storing a GCPointer on the CStack*]
/// (https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting).
#[no_move]
pub struct RootCollection {
    roots: UnsafeCell<Vec<*const Reflector>>,
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
        RootCollection {
            roots: UnsafeCell::new(vec!()),
        }
    }

    /// Start tracking a stack-based root
    fn root<'b>(&self, untracked_reflector: *const Reflector) {
        unsafe {
            let mut roots = &mut *self.roots.get();
            roots.push(untracked_reflector);
            assert!(!(*untracked_reflector).get_jsobject().is_null())
        }
    }

    /// Stop tracking a stack-based root, asserting if the reflector isn't found
    fn unroot<'b, T: Reflectable>(&self, rooted: &Root<T>) {
        unsafe {
            let mut roots = &mut *self.roots.get();
            let old_reflector = &*rooted.r().reflector();
            match roots.iter().rposition(|r| *r == old_reflector) {
                Some(idx) => {
                    roots.remove(idx);
                },
                None => panic!("Can't remove a root that was never rooted!")
            }
        }
    }
}

/// SM Callback that traces the rooted reflectors
pub unsafe fn trace_roots(tracer: *mut JSTracer) {
    STACK_ROOTS.with(|ref collection| {
        let RootCollectionPtr(collection) = collection.get().unwrap();
        let collection = &*(*collection).roots.get();
        for root in collection {
            trace_reflector(tracer, "reflector", &**root);
        }
    });
}

/// A rooted reference to a DOM object.
///
/// The JS value is pinned for the duration of this object's lifetime; roots
/// are additive, so this object's destruction will not invalidate other roots
/// for the same JS value. `Root`s cannot outlive the associated
/// `RootCollection` object.
#[allow_unrooted_interior]
pub struct Root<T: Reflectable> {
    /// Reference to rooted value that must not outlive this container
    ptr: NonZero<*const T>,
    /// List that ensures correct dynamic root ordering
    root_list: *const RootCollection,
}

impl<T: Castable> Root<T> {
    /// Cast a DOM object root upwards to one of the interfaces it derives from.
    pub fn upcast<U>(root: Root<T>) -> Root<U> where U: Castable, T: DerivedFrom<U> {
        unsafe { mem::transmute(root) }
    }

    /// Cast a DOM object root downwards to one of the interfaces it might implement.
    pub fn downcast<U>(root: Root<T>) -> Option<Root<U>> where U: DerivedFrom<T> {
        if root.is::<U>() {
            Some(unsafe { mem::transmute(root) })
        } else {
            None
        }
    }
}

impl<T: Reflectable> Root<T> {
    /// Create a new stack-bounded root for the provided JS-owned value.
    /// It cannot not outlive its associated `RootCollection`, and it gives
    /// out references which cannot outlive this new `Root`.
    pub fn new(unrooted: NonZero<*const T>)
               -> Root<T> {
        STACK_ROOTS.with(|ref collection| {
            let RootCollectionPtr(collection) = collection.get().unwrap();
            unsafe { (*collection).root(&*(**unrooted).reflector()) }
            Root {
                ptr: unrooted,
                root_list: collection,
            }
        })
    }

    /// Generate a new root from a reference
    pub fn from_ref(unrooted: &T) -> Root<T> {
        Root::new(unsafe { NonZero::new(&*unrooted) })
    }

    /// Obtain a safe reference to the wrapped JS owned-value that cannot
    /// outlive the lifetime of this root.
    pub fn r(&self) -> &T {
        &**self
    }
}

impl<T: Reflectable> Deref for Root<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &**self.ptr.deref() }
    }
}

impl<T: Reflectable> PartialEq for Root<T> {
    fn eq(&self, other: &Root<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<T: Reflectable> Drop for Root<T> {
    fn drop(&mut self) {
        unsafe { (*self.root_list).unroot(self); }
    }
}
