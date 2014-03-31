/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflector, Reflectable};
use dom::window::Window;
use js::jsapi::{JSObject, JSContext};
use layout_interface::TrustedNodeAddress;

use std::cast;
use std::cell::{Cell, RefCell};
use std::ptr;

/// A type that represents a JS-owned value that may or may not be rooted.
/// Importantly, it requires rooting in order to interact with the value in any way.
/// Can be assigned into JS-owned member fields (ie. JS<T> types) safely via the
/// `JS<T>::assign` method or `OptionalAssignable::assign` (for Option<JS<T>> fields).
pub struct Unrooted<T> {
    inner: JS<T>
}

impl<T> Eq for Unrooted<T> {
    fn eq(&self, other: &Unrooted<T>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Reflectable> Unrooted<T> {
    /// Create a new Unrooted value from a JS-owned value.
    pub fn new(inner: JS<T>) -> Unrooted<T> {
        Unrooted {
            inner: inner
        }
    }

    /// Create a new Unrooted value from a rooted value.
    pub fn new_rooted<'a>(root: &JSRef<'a, T>) -> Unrooted<T> {
        Unrooted {
            inner: root.unrooted()
        }
    }

    /// Root this unrooted value.
    pub fn root<'a, 'b>(self, collection: &'a RootCollection) -> Root<'a, 'b, T> {
        collection.new_root(&self.inner)
    }

    unsafe fn inner(&self) -> JS<T> {
        self.inner.clone()
    }

    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute<To>(self) -> Unrooted<To> {
        cast::transmute(self)
    }
}

/// A rooted, JS-owned value. Must only be used as a field in other JS-owned types.
pub struct JS<T> {
    ptr: RefCell<*mut T>
}

impl<T> Eq for JS<T> {
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

impl<T: Reflectable> JS<T> {
    /// Create a new JS-reflected DOM object; returns an Unrooted type because the new value
    /// is not safe to use until it is rooted.
    pub fn new(obj: ~T,
               window:  &JSRef<Window>,
               wrap_fn: extern "Rust" fn(*JSContext, &JSRef<Window>, ~T) -> JS<T>) -> Unrooted<T> {
        Unrooted::new(wrap_fn(window.get().get_cx(), window, obj))
    }

    /// Create a new JS-owned value wrapped from a raw Rust pointer.
    pub unsafe fn from_raw(raw: *mut T) -> JS<T> {
        JS {
            ptr: RefCell::new(raw)
        }
    }


    /// Create a new JS-owned value wrapped from an address known to be a Node pointer.
    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> JS<T> {
        let TrustedNodeAddress(addr) = inner;
        JS {
            ptr: RefCell::new(addr as *mut T)
        }
    }

    /// Root this JS-owned value to prevent its collection as garbage.
    pub fn root<'a, 'b>(&self, collection: &'a RootCollection) -> Root<'a, 'b, T> {
        collection.new_root(self)
    }
}

impl<T: Reflectable> Reflectable for JS<T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.get().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.get_mut().mut_reflector()
    }
}

impl<T: Reflectable> JS<T> {
    pub fn get<'a>(&'a self) -> &'a T {
        let borrowed = self.ptr.borrow();
        unsafe {
            &**borrowed
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        let mut borrowed = self.ptr.borrow_mut();
        unsafe {
            &mut **borrowed
        }
    }

    /// Returns an unsafe pointer to the interior of this JS object without touching the borrow
    /// flags. This is the only method that be safely accessed from layout. (The fact that this
    /// is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *mut T {
        cast::transmute_copy(&self.ptr)
    }

    /// Store an unrooted value in this field. This is safe under the assumption that JS<T>
    /// values are only used as fields in DOM types that are reachable in the GC graph,
    /// so this unrooted value becomes transitively rooted for the lifetime of its new owner.
    pub fn assign(&mut self, val: Unrooted<T>) {
        *self = unsafe { val.inner() };
    }
}

impl<From, To> JS<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute(self) -> JS<To> {
        cast::transmute(self)
    }

    pub unsafe fn transmute_copy(&self) -> JS<To> {
        cast::transmute_copy(self)
    }
}

pub trait RootedReference<T> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>>;
}

impl<'a, 'b, T: Reflectable> RootedReference<T> for Option<Root<'a, 'b, T>> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>> {
        self.as_ref().map(|root| root.root_ref())
    }
}

// This trait should never be public; it allows access to unrooted values, and is thus
// easy to misuse.
/*definitely not public*/ trait Assignable<T> {
    fn get_js(&self) -> JS<T>;
}

impl<T> Assignable<T> for JS<T> {
    fn get_js(&self) -> JS<T> {
        self.clone()
    }
}

impl<'a, T> Assignable<T> for JSRef<'a, T> {
    fn get_js(&self) -> JS<T> {
        self.unrooted()
    }
}

// Assignable should not be exposed publically, since it's used to
// extract unrooted values in a safe way WHEN USED CORRECTLY.
impl<T: Reflectable> Assignable<T> for Unrooted<T> {
    fn get_js(&self) -> JS<T> {
        unsafe { self.inner() }
    }
}

pub trait OptionalAssignable<T> {
    fn assign(&mut self, val: Option<T>);
}

impl<T: Assignable<U>, U: Reflectable> OptionalAssignable<T> for Option<JS<U>> {
    fn assign(&mut self, val: Option<T>) {
        *self = val.map(|val| val.get_js());
    }
}

pub trait OptionalRootable<T> {
    fn root<'a, 'b>(self, roots: &'a RootCollection) -> Option<Root<'a, 'b, T>>;
}

impl<T: Reflectable> OptionalRootable<T> for Option<Unrooted<T>> {
    fn root<'a, 'b>(self, roots: &'a RootCollection) -> Option<Root<'a, 'b, T>> {
        self.map(|inner| inner.root(roots))
    }
}

pub trait ResultRootable<T,U> {
    fn root<'a, 'b>(self, roots: &'a RootCollection) -> Result<Root<'a, 'b, T>, U>;
}

impl<T: Reflectable, U> ResultRootable<T, U> for Result<Unrooted<T>, U> {
    fn root<'a, 'b>(self, roots: &'a RootCollection) -> Result<Root<'a, 'b, T>, U> {
        self.map(|inner| inner.root(roots))
    }
}

/// Provides a facility to push unrooted values onto lists of rooted values. This is safe
/// under the assumption that said lists are reachable via the GC graph, and therefore the
/// new values are transitively rooted for the lifetime of their new owner.
pub trait UnrootedPushable<T> {
    fn push_unrooted(&mut self, val: Unrooted<T>);
}

impl<T: Reflectable> UnrootedPushable<T> for Vec<JS<T>> {
    fn push_unrooted(&mut self, val: Unrooted<T>) {
        unsafe { self.push(val.inner()) };
    }
}

#[deriving(Eq, Clone)]
struct RootReference(*JSObject);

impl RootReference {
    fn new<'a, 'b, T: Reflectable>(unrooted: &Root<'a, 'b, T>) -> RootReference {
        RootReference(unrooted.rooted())
    }

    fn null() -> RootReference {
        RootReference(ptr::null())
    }
}

static MAX_STACK_ROOTS: uint = 10;

/// An opaque, LIFO rooting mechanism.
pub struct RootCollection {
    roots: [Cell<RootReference>, ..MAX_STACK_ROOTS],
    current: Cell<uint>,
}

impl RootCollection {
    pub fn new() -> RootCollection {
        RootCollection {
            roots: [Cell::new(RootReference::null()), ..MAX_STACK_ROOTS],
            current: Cell::new(0),
        }
    }

    fn new_root<'a, 'b, T: Reflectable>(&'a self, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        Root::new(self, unrooted)
    }

    fn root_impl(&self, unrooted: RootReference) {
        let current = self.current.get();
        assert!(current < MAX_STACK_ROOTS);
        self.roots[current].set(unrooted);
        debug!("  rooting {:?}", unrooted);
        self.current.set(current + 1);
    }

    fn root<'a, 'b, T: Reflectable>(&self, unrooted: &Root<'a, 'b, T>) {
        self.root_impl(RootReference::new(unrooted));
    }

    /// Root a raw JS pointer.
    pub fn root_raw(&self, unrooted: *JSObject) {
        self.root_impl(RootReference(unrooted));
    }

    fn unroot_impl(&self, rooted: RootReference) {
        let mut current = self.current.get();
        assert!(current != 0);
        current -= 1;
        debug!("unrooting {:?} (expecting {:?}", self.roots[current].get(), rooted);
        assert!(self.roots[current].get() == rooted);
        self.roots[current].set(RootReference::null());
        self.current.set(current);
    }

    fn unroot<'a, 'b, T: Reflectable>(&self, rooted: &Root<'a, 'b, T>) {
        self.unroot_impl(RootReference::new(rooted));
    }

    /// Unroot a raw JS pointer. Must occur in reverse order to its rooting.
    pub fn unroot_raw(&self, rooted: *JSObject) {
        self.unroot_impl(RootReference(rooted));
    }
}

/// A rooted JS value. The JS value is pinned for the duration of this object's lifetime;
/// roots are additive, so this object's destruction will not invalidate other roots
/// for the same JS value. Roots cannot outlive the associated RootCollection object.
/// Attempts to transfer ownership of a Root via moving will trigger dynamic unrooting
/// failures due to incorrect ordering.
pub struct Root<'a, 'b, T> {
    root_list: &'a RootCollection,
    jsref: JSRef<'b, T>,
    ptr: RefCell<*mut T>,
}

impl<'a, 'b, T: Reflectable> Root<'a, 'b, T> {
    fn new(roots: &'a RootCollection, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        let root = Root {
            root_list: roots,
            jsref: JSRef {
                ptr: unrooted.ptr.clone(),
                chain: unsafe { ::std::cast::transmute_region(&()) },
            },
            ptr: unrooted.ptr.clone()
        };
        roots.root(&root);
        root
    }

    pub fn get<'a>(&'a self) -> &'a T {
        unsafe {
            let borrow = self.ptr.borrow();
            &**borrow
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
       unsafe {
            let mut borrow = self.ptr.borrow_mut();
            &mut **borrow
        }
    }

    fn rooted(&self) -> *JSObject {
        self.reflector().get_jsobject()
    }

    fn internal_root_ref<'a>(&'a self) -> &'a JSRef<'b, T> {
        &'a self.jsref
    }

    fn mut_internal_root_ref<'a>(&'a mut self) -> &'a mut JSRef<'b, T> {
        &'a mut self.jsref
    }

    pub fn root_ref<'b>(&'b self) -> JSRef<'b,T> {
        self.internal_root_ref().clone()
    }
}

#[unsafe_destructor]
impl<'a, 'b, T: Reflectable> Drop for Root<'a, 'b, T> {
    fn drop(&mut self) {
        self.root_list.unroot(self);
    }
}

impl<'a, 'b, T: Reflectable> Reflectable for Root<'a, 'b, T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.get().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.get_mut().mut_reflector()
    }
}

impl<'a, 'b, T: Reflectable> Deref<JSRef<'b, T>> for Root<'a, 'b, T> {
    fn deref<'c>(&'c self) -> &'c JSRef<'b, T> {
        self.internal_root_ref()
    }
}

impl<'a, 'b, T: Reflectable> DerefMut<JSRef<'b, T>> for Root<'a, 'b, T> {
    fn deref_mut<'c>(&'c mut self) -> &'c mut JSRef<'b, T> {
        self.mut_internal_root_ref()
    }
}

impl<'a, T: Reflectable> Deref<T> for JSRef<'a, T> {
    fn deref<'b>(&'b self) -> &'b T {
        self.get()
    }
}

impl<'a, T: Reflectable> DerefMut<T> for JSRef<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.get_mut()
    }
}

/// Encapsulates a reference to something that is guaranteed to be alive. This is freely copyable.
pub struct JSRef<'a, T> {
    ptr: RefCell<*mut T>,
    chain: &'a (),
}

impl<'a, T> Clone for JSRef<'a, T> {
    fn clone(&self) -> JSRef<'a, T> {
        JSRef {
            ptr: self.ptr.clone(),
            chain: self.chain
        }
    }
}

impl<'a, T> Eq for JSRef<'a, T> {
    fn eq(&self, other: &JSRef<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a,T> JSRef<'a,T> {
    pub fn get<'a>(&'a self) -> &'a T {
        unsafe {
            let borrow = self.ptr.borrow();
            &**borrow
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        let mut borrowed = self.ptr.borrow_mut();
        unsafe {
            &mut **borrowed
        }
    }

    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute<'b, To>(&'b self) -> &'b JSRef<'a, To> {
        cast::transmute(self)
    }

    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute_mut<'b, To>(&'b mut self) -> &'b mut JSRef<'a, To> {
        cast::transmute(self)
    }

    pub fn unrooted(&self) -> JS<T> {
        JS {
            ptr: self.ptr.clone()
        }
    }
}

impl<'a, T: Reflectable> Reflectable for JSRef<'a, T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.get().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.get_mut().mut_reflector()
    }
}
