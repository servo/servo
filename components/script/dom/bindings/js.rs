/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Smart pointers for the JS-managed DOM objects.
//!
//! The DOM is made up of Rust types whose lifetime is entirely controlled by the whims of
//! the SpiderMonkey garbage collector. The types in this module are designed to ensure
//! that any interactions with said Rust types only occur on values that will remain alive
//! the entire time.
//!
//! Here is a brief overview of the important types:
//!
//! - `JSRef<T>`: a freely-copyable reference to a rooted value.
//! - `Root<T>`: a stack-based reference to a rooted value.
//! - `JS<T>`: a pointer to JS-owned memory that can automatically be traced by the GC when
//!          encountered as a field of a Rust structure.
//! - `Temporary<T>`: a value that will remain rooted for the duration of its lifetime.
//!
//! The rule of thumb is as follows:
//!
//! - All methods return `Temporary<T>`, to ensure the value remains alive until it is stored
//!   somewhere that is reachable by the GC.
//! - All functions take `JSRef<T>` arguments, to ensure that they will remain uncollected for
//!   the duration of their usage.
//! - All types contain `JS<T>` fields and derive the `Encodable` trait, to ensure that they are
//!   transitively marked as reachable by the GC if the enclosing value is reachable.
//! - All methods for type `T` are implemented for `JSRef<T>`, to ensure that the self value
//!   will not be collected for the duration of the method call.
//!
//! Both `Temporary<T>` and `JS<T>` do not allow access to their inner value without explicitly
//! creating a stack-based root via the `root` method. This returns a `Root<T>`, which causes
//! the JS-owned value to be uncollectable for the duration of the `Root` object's lifetime.
//! A `JSRef<T>` can be obtained from a `Root<T>` either by dereferencing the `Root<T>` (`*rooted`)
//! or explicitly calling the `root_ref` method. These `JSRef<T>` values are not allowed to
//! outlive their originating `Root<T>`, to ensure that all interactions with the enclosed value
//! only occur when said value is uncollectable, and will cause static lifetime errors if
//! misused.
//!
//! Other miscellaneous helper traits:
//!
//! - `OptionalRootable` and `OptionalRootedRootable`: make rooting `Option` values easy via a `root` method
//! - `ResultRootable`: make rooting successful `Result` values easy
//! - `TemporaryPushable`: allows mutating vectors of `JS<T>` with new elements of `JSRef`/`Temporary`
//! - `OptionalSettable`: allows assigning `Option` values of `JSRef`/`Temporary` to fields of `Option<JS<T>>`
//! - `RootedReference`: makes obtaining an `Option<JSRef<T>>` from an `Option<Root<T>>` easy

use dom::bindings::utils::{Reflector, Reflectable};
use dom::node::Node;
use dom::xmlhttprequest::{XMLHttpRequest, TrustedXHRAddress};
use dom::worker::{Worker, TrustedWorkerAddress};
use js::jsapi::JSObject;
use layout_interface::TrustedNodeAddress;
use script_task::StackRoots;

use std::cell::{Cell, RefCell};
use std::kinds::marker::ContravariantLifetime;
use std::mem;

/// A type that represents a JS-owned value that is rooted for the lifetime of this value.
/// Importantly, it requires explicit rooting in order to interact with the inner value.
/// Can be assigned into JS-owned member fields (i.e. `JS<T>` types) safely via the
/// `JS<T>::assign` method or `OptionalSettable::assign` (for `Option<JS<T>>` fields).
#[allow(unrooted_must_root)]
pub struct Temporary<T> {
    inner: JS<T>,
    /// On-stack JS pointer to assuage conservative stack scanner
    _js_ptr: *mut JSObject,
}

impl<T> PartialEq for Temporary<T> {
    fn eq(&self, other: &Temporary<T>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Reflectable> Temporary<T> {
    /// Create a new `Temporary` value from a JS-owned value.
    pub fn new(inner: JS<T>) -> Temporary<T> {
        Temporary {
            inner: inner,
            _js_ptr: inner.reflector().get_jsobject(),
        }
    }

    /// Create a new `Temporary` value from a rooted value.
    pub fn from_rooted<'a>(root: JSRef<'a, T>) -> Temporary<T> {
        Temporary::new(JS::from_rooted(root))
    }

    /// Create a stack-bounded root for this value.
    pub fn root<'a, 'b>(self) -> Root<'a, 'b, T> {
        let collection = StackRoots.get().unwrap();
        unsafe {
            (**collection).new_root(&self.inner)
        }
    }

    unsafe fn inner(&self) -> JS<T> {
        self.inner.clone()
    }

    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute<To>(self) -> Temporary<To> {
        mem::transmute(self)
    }
}

/// A rooted, JS-owned value. Must only be used as a field in other JS-owned types.
#[must_root]
pub struct JS<T> {
    ptr: *const T
}

impl<T> PartialEq for JS<T> {
    #[allow(unrooted_must_root)]
    fn eq(&self, other: &JS<T>) -> bool {
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

impl JS<Node> {
    /// Create a new JS-owned value wrapped from an address known to be a `Node` pointer.
    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> JS<Node> {
        let TrustedNodeAddress(addr) = inner;
        JS {
            ptr: addr as *const Node
        }
    }
}

impl JS<XMLHttpRequest> {
    pub unsafe fn from_trusted_xhr_address(inner: TrustedXHRAddress) -> JS<XMLHttpRequest> {
        let TrustedXHRAddress(addr) = inner;
        JS {
            ptr: addr as *const XMLHttpRequest
        }
    }
}

impl JS<Worker> {
    pub unsafe fn from_trusted_worker_address(inner: TrustedWorkerAddress) -> JS<Worker> {
        let TrustedWorkerAddress(addr) = inner;
        JS {
            ptr: addr as *const Worker
        }
    }
}

impl<T: Reflectable> JS<T> {
    /// Create a new JS-owned value wrapped from a raw Rust pointer.
    pub unsafe fn from_raw(raw: *const T) -> JS<T> {
        JS {
            ptr: raw
        }
    }


    /// Root this JS-owned value to prevent its collection as garbage.
    pub fn root<'a, 'b>(&self) -> Root<'a, 'b, T> {
        let collection = StackRoots.get().unwrap();
        unsafe {
            (**collection).new_root(self)
        }
    }
}

impl<T: Assignable<U>, U: Reflectable> JS<U> {
    pub fn from_rooted(root: T) -> JS<U> {
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
            (*self.unsafe_get()).reflector()
        }
    }
}

impl<T: Reflectable> JS<T> {
    /// Returns an unsafe pointer to the interior of this JS object without touching the borrow
    /// flags. This is the only method that be safely accessed from layout. (The fact that this
    /// is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *mut T {
        mem::transmute_copy(&self.ptr)
    }

    /// Store an unrooted value in this field. This is safe under the assumption that JS<T>
    /// values are only used as fields in DOM types that are reachable in the GC graph,
    /// so this unrooted value becomes transitively rooted for the lifetime of its new owner.
    pub fn assign(&mut self, val: Temporary<T>) {
        *self = unsafe { val.inner() };
    }
}

impl<From, To> JS<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute(self) -> JS<To> {
        mem::transmute(self)
    }

    pub unsafe fn transmute_copy(&self) -> JS<To> {
        mem::transmute_copy(self)
    }
}


/// Get an `Option<JSRef<T>>` out of an `Option<Root<T>>`
pub trait RootedReference<T> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>>;
}

impl<'a, 'b, T: Reflectable> RootedReference<T> for Option<Root<'a, 'b, T>> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>> {
        self.as_ref().map(|root| root.root_ref())
    }
}

/// Get an `Option<Option<JSRef<T>>>` out of an `Option<Option<Root<T>>>`
pub trait OptionalRootedReference<T> {
    fn root_ref<'a>(&'a self) -> Option<Option<JSRef<'a, T>>>;
}

impl<'a, 'b, T: Reflectable> OptionalRootedReference<T> for Option<Option<Root<'a, 'b, T>>> {
    fn root_ref<'a>(&'a self) -> Option<Option<JSRef<'a, T>>> {
        self.as_ref().map(|inner| inner.root_ref())
    }
}

/// Trait that allows extracting a `JS<T>` value from a variety of rooting-related containers,
/// which in general is an unsafe operation since they can outlive the rooted lifetime of the
/// original value.
/*definitely not public*/ trait Assignable<T> {
    unsafe fn get_js(&self) -> JS<T>;
}

impl<T> Assignable<T> for JS<T> {
    unsafe fn get_js(&self) -> JS<T> {
        self.clone()
    }
}

impl<'a, T> Assignable<T> for JSRef<'a, T> {
    unsafe fn get_js(&self) -> JS<T> {
        self.unrooted()
    }
}

impl<T: Reflectable> Assignable<T> for Temporary<T> {
    unsafe fn get_js(&self) -> JS<T> {
        self.inner()
    }
}

/// Assign an optional rootable value (either of `JS<T>` or `Temporary<T>`) to an optional
/// field of a DOM type (ie. `Option<JS<T>>`)
pub trait OptionalSettable<T> {
    fn assign(&self, val: Option<T>);
}

impl<T: Assignable<U>, U: Reflectable> OptionalSettable<T> for Cell<Option<JS<U>>> {
    fn assign(&self, val: Option<T>) {
        self.set(val.map(|val| unsafe { val.get_js() }));
    }
}


/// Root a rootable `Option` type (used for `Option<Temporary<T>>`)
pub trait OptionalRootable<T> {
    fn root<'a, 'b>(self) -> Option<Root<'a, 'b, T>>;
}

impl<T: Reflectable> OptionalRootable<T> for Option<Temporary<T>> {
    fn root<'a, 'b>(self) -> Option<Root<'a, 'b, T>> {
        self.map(|inner| inner.root())
    }
}

/// Return an unrooted type for storing in optional DOM fields
pub trait OptionalUnrootable<T> {
    fn unrooted(&self) -> Option<JS<T>>;
}

impl<'a, T: Reflectable> OptionalUnrootable<T> for Option<JSRef<'a, T>> {
    fn unrooted(&self) -> Option<JS<T>> {
        self.as_ref().map(|inner| JS::from_rooted(*inner))
    }
}

/// Root a rootable `Option` type (used for `Option<JS<T>>`)
pub trait OptionalRootedRootable<T> {
    fn root<'a, 'b>(&self) -> Option<Root<'a, 'b, T>>;
}

impl<T: Reflectable> OptionalRootedRootable<T> for Option<JS<T>> {
    fn root<'a, 'b>(&self) -> Option<Root<'a, 'b, T>> {
        self.as_ref().map(|inner| inner.root())
    }
}

/// Root a rootable `Option<Option>` type (used for `Option<Option<JS<T>>>`)
pub trait OptionalOptionalRootedRootable<T> {
    fn root<'a, 'b>(&self) -> Option<Option<Root<'a, 'b, T>>>;
}

impl<T: Reflectable> OptionalOptionalRootedRootable<T> for Option<Option<JS<T>>> {
    fn root<'a, 'b>(&self) -> Option<Option<Root<'a, 'b, T>>> {
        self.as_ref().map(|inner| inner.root())
    }
}


/// Root a rootable `Result` type (any of `Temporary<T>` or `JS<T>`)
pub trait ResultRootable<T,U> {
    fn root<'a, 'b>(self) -> Result<Root<'a, 'b, T>, U>;
}

impl<T: Reflectable, U> ResultRootable<T, U> for Result<Temporary<T>, U> {
    fn root<'a, 'b>(self) -> Result<Root<'a, 'b, T>, U> {
        self.map(|inner| inner.root())
    }
}

impl<T: Reflectable, U> ResultRootable<T, U> for Result<JS<T>, U> {
    fn root<'a, 'b>(self) -> Result<Root<'a, 'b, T>, U> {
        self.map(|inner| inner.root())
    }
}

/// Provides a facility to push unrooted values onto lists of rooted values. This is safe
/// under the assumption that said lists are reachable via the GC graph, and therefore the
/// new values are transitively rooted for the lifetime of their new owner.
pub trait TemporaryPushable<T> {
    fn push_unrooted(&mut self, val: &T);
    fn insert_unrooted(&mut self, index: uint, val: &T);
}

impl<T: Assignable<U>, U: Reflectable> TemporaryPushable<T> for Vec<JS<U>> {
    fn push_unrooted(&mut self, val: &T) {
        self.push(unsafe { val.get_js() });
    }

    fn insert_unrooted(&mut self, index: uint, val: &T) {
        self.insert(index, unsafe { val.get_js() });
    }
}

/// An opaque, LIFO rooting mechanism.
pub struct RootCollection {
    roots: RefCell<Vec<*mut JSObject>>,
}

impl RootCollection {
    /// Create an empty collection of roots
    pub fn new() -> RootCollection {
        RootCollection {
            roots: RefCell::new(vec!()),
        }
    }

    /// Create a new stack-bounded root that will not outlive this collection
    #[allow(unrooted_must_root)]
    fn new_root<'a, 'b, T: Reflectable>(&'a self, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        Root::new(self, unrooted)
    }

    /// Track a stack-based root to ensure LIFO root ordering
    fn root<'a, 'b, T: Reflectable>(&self, untracked: &Root<'a, 'b, T>) {
        let mut roots = self.roots.borrow_mut();
        roots.push(untracked.js_ptr);
        debug!("  rooting {:?}", untracked.js_ptr);
    }

    /// Stop tracking a stack-based root, asserting if LIFO root ordering has been violated
    fn unroot<'a, 'b, T: Reflectable>(&self, rooted: &Root<'a, 'b, T>) {
        let mut roots = self.roots.borrow_mut();
        debug!("unrooting {:?} (expecting {:?}", roots.last().unwrap(), rooted.js_ptr);
        assert!(*roots.last().unwrap() == rooted.js_ptr);
        roots.pop().unwrap();
    }
}

/// A rooted JS value. The JS value is pinned for the duration of this object's lifetime;
/// roots are additive, so this object's destruction will not invalidate other roots
/// for the same JS value. `Root`s cannot outlive the associated `RootCollection` object.
/// Attempts to transfer ownership of a `Root` via moving will trigger dynamic unrooting
/// failures due to incorrect ordering.
pub struct Root<'a, 'b, T> {
    /// List that ensures correct dynamic root ordering
    root_list: &'a RootCollection,
    /// Reference to rooted value that must not outlive this container
    jsref: JSRef<'b, T>,
    /// On-stack JS pointer to assuage conservative stack scanner
    js_ptr: *mut JSObject,
}

impl<'a, 'b, T: Reflectable> Root<'a, 'b, T> {
    /// Create a new stack-bounded root for the provided JS-owned value.
    /// It cannot not outlive its associated `RootCollection`, and it contains a `JSRef`
    /// which cannot outlive this new `Root`.
    fn new(roots: &'a RootCollection, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        let root = Root {
            root_list: roots,
            jsref: JSRef {
                ptr: unrooted.ptr.clone(),
                chain: ContravariantLifetime,
            },
            js_ptr: unrooted.reflector().get_jsobject(),
        };
        roots.root(&root);
        root
    }

    /// Obtain a safe reference to the wrapped JS owned-value that cannot outlive
    /// the lifetime of this root.
    pub fn root_ref<'b>(&'b self) -> JSRef<'b,T> {
        self.jsref.clone()
    }
}

#[unsafe_destructor]
impl<'a, 'b, T: Reflectable> Drop for Root<'a, 'b, T> {
    fn drop(&mut self) {
        self.root_list.unroot(self);
    }
}

impl<'a, 'b, T: Reflectable> Deref<JSRef<'b, T>> for Root<'a, 'b, T> {
    fn deref<'c>(&'c self) -> &'c JSRef<'b, T> {
        &self.jsref
    }
}

impl<'a, T: Reflectable> Deref<T> for JSRef<'a, T> {
    fn deref<'b>(&'b self) -> &'b T {
        unsafe {
            &*self.ptr
        }
    }
}

/// Encapsulates a reference to something that is guaranteed to be alive. This is freely copyable.
pub struct JSRef<'a, T> {
    ptr: *const T,
    chain: ContravariantLifetime<'a>,
}

impl<'a, T> Clone for JSRef<'a, T> {
    fn clone(&self) -> JSRef<'a, T> {
        JSRef {
            ptr: self.ptr.clone(),
            chain: self.chain,
        }
    }
}

impl<'a, T> PartialEq for JSRef<'a, T> {
    fn eq(&self, other: &JSRef<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a,T> JSRef<'a,T> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute<To>(self) -> JSRef<'a, To> {
        mem::transmute(self)
    }

    // FIXME(zwarich): It would be nice to get rid of this entirely.
    pub unsafe fn transmute_borrowed<'b, To>(&'b self) -> &'b JSRef<'a, To> {
        mem::transmute(self)
    }

    pub fn unrooted(&self) -> JS<T> {
        JS {
            ptr: self.ptr
        }
    }
}

impl<'a, T: Reflectable> Reflectable for JSRef<'a, T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.deref().reflector()
    }
}
