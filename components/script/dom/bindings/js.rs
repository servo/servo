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

use dom::bindings::trace::JSTraceable;
use dom::bindings::trace::trace_reflector;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::node::Node;
use js::jsapi::{JSObject, Heap, JSTracer};
use js::jsval::JSVal;
use layout_interface::TrustedNodeAddress;
use script_task::STACK_ROOTS;
use util::mem::HeapSizeOf;

use core::nonzero::NonZero;
use std::cell::{Cell, UnsafeCell};
use std::default::Default;
use std::ops::Deref;

/// A traced reference to a DOM object. Must only be used as a field in other
/// DOM objects.
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
    pub unsafe fn to_layout(self) -> LayoutJS<T> {
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
    pub fn from_rooted(root: &Root<T>) -> JS<T> {
        JS {
            ptr: unsafe { NonZero::new(&**root) }
        }
    }
    /// Create a JS<T> from a &T
    pub fn from_ref(obj: &T) -> JS<T> {
        JS {
            ptr: unsafe { NonZero::new(&*obj) }
        }
    }
    /// Store an rooted value in this field. This is safe under the
    /// assumption that JS<T> values are only used as fields in DOM types that
    /// are reachable in the GC graph, so this unrooted value becomes
    /// transitively rooted for the lifetime of its new owner.
    pub fn assign(&mut self, val: Root<T>) {
        self.ptr = val.ptr.clone();
    }
}

/// An unrooted reference to a DOM object for use in layout. `Layout*Helpers`
/// traits must be implemented on this.
#[allow_unrooted_interior]
pub struct LayoutJS<T> {
    ptr: NonZero<*const T>
}

impl<T: Reflectable> LayoutJS<T> {
    /// Get the reflector.
    pub unsafe fn get_jsobject(&self) -> *mut JSObject {
        (**self.ptr).reflector().get_jsobject().get()
    }
}

impl<T> Copy for JS<T> {}

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

    /// Set the value in this `MutHeapJSVal`, calling read barriers as appropriate.
    pub fn get(&self) -> JSVal {
        unsafe { (*self.val.get()).get() }
    }
}


/// A holder that provides interior mutability for GC-managed values such as
/// `JS<T>`.
#[must_root]
#[derive(JSTraceable)]
pub struct MutHeap<T: HeapGCValue + Copy> {
    val: Cell<T>,
}

impl<T: HeapGCValue + Copy> MutHeap<T> {
    /// Create a new `MutHeap`.
    pub fn new(initial: T) -> MutHeap<T> {
        MutHeap {
            val: Cell::new(initial),
        }
    }

    /// Set this `MutHeap` to the given value.
    pub fn set(&self, val: T) {
        self.val.set(val)
    }

    /// Set the value in this `MutHeap`.
    pub fn get(&self) -> T {
        self.val.get()
    }
}

/// A mutable holder for GC-managed values such as `JSval` and `JS<T>`, with
/// nullability represented by an enclosing Option wrapper. Must be used in
/// place of traditional internal mutability to ensure that the proper GC
/// barriers are enforced.
#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct MutNullableHeap<T: HeapGCValue + Copy> {
    ptr: Cell<Option<T>>
}

impl<T: HeapGCValue + Copy> MutNullableHeap<T> {
    /// Create a new `MutNullableHeap`.
    pub fn new(initial: Option<T>) -> MutNullableHeap<T> {
        MutNullableHeap {
            ptr: Cell::new(initial)
        }
    }

    /// Set this `MutNullableHeap` to the given value.
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
    pub fn or_init<F>(&self, cb: F) -> Root<T>
        where F: FnOnce() -> Root<T>
    {
        match self.get() {
            Some(inner) => Root::from_rooted(inner),
            None => {
                let inner = cb();
                self.set(Some(JS::from_rooted(&inner)));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `JS<T>` as `LayoutJS<T>`.
    /// For use by layout, which can't use safe types like Temporary.
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutJS<T>> {
        self.ptr.get().map(|js| js.to_layout())
    }

    /// Get a rooted value out of this object
    // FIXME(#6684)
    pub fn get_rooted(&self) -> Option<Root<T>> {
        self.get().map(|o| o.root())
    }
}

impl<T: HeapGCValue + Copy> Default for MutNullableHeap<T> {
    fn default() -> MutNullableHeap<T> {
        MutNullableHeap {
            ptr: Cell::new(None)
        }
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
    fn r<'a>(&'a self) -> Option<&'a T>;
}

impl<T: Reflectable> RootedReference<T> for Option<Root<T>> {
    fn r<'a>(&'a self) -> Option<&'a T> {
        self.as_ref().map(|root| root.r())
    }
}

/// Get an `Option<Option<&T>>` out of an `Option<Option<Root<T>>>`
pub trait OptionalRootedReference<T> {
    /// Obtain a safe optional optional reference to the wrapped JS owned-value
    /// that cannot outlive the lifetime of this root.
    fn r<'a>(&'a self) -> Option<Option<&'a T>>;
}

impl<T: Reflectable> OptionalRootedReference<T> for Option<Option<Root<T>>> {
    fn r<'a>(&'a self) -> Option<Option<&'a T>> {
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
    pub fn r<'a>(&'a self) -> &'a T {
        &**self
    }

    /// Generate a new root from a JS<T> reference
    #[allow(unrooted_must_root)]
    pub fn from_rooted(js: JS<T>) -> Root<T> {
        js.root()
    }
}

impl<T: Reflectable> Deref for Root<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
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
