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
//use std::ops::{Deref, DerefMut};

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
    pub fn new(obj: ~T,
               window:  &JSRef<Window>,
               wrap_fn: extern "Rust" fn(*JSContext, &JSRef<Window>, ~T) -> JS<T>) -> JS<T> {
        wrap_fn(window.get().get_cx(), window, obj)
    }

    pub unsafe fn from_raw(raw: *mut T) -> JS<T> {
        JS {
            ptr: RefCell::new(raw)
        }
    }


    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> JS<T> {
        let TrustedNodeAddress(addr) = inner;
        JS {
            ptr: RefCell::new(addr as *mut T)
        }
    }

    pub fn root<'a>(&self, collection: &'a RootCollection) -> Root<'a, T> {
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

impl<T> JS<T> {
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

impl<'a, T: Reflectable> RootedReference<T> for Option<Root<'a, T>> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>> {
        self.as_ref().map(|root| root.root_ref())
    }
}

#[deriving(Eq, Clone)]
struct RootReference(*JSObject);

impl RootReference {
    fn new<'a, T: Reflectable>(unrooted: &Root<'a, T>) -> RootReference {
        RootReference(unrooted.rooted())
    }

    fn null() -> RootReference {
        RootReference(ptr::null())
    }
}

static MAX_STACK_ROOTS: uint = 10;

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

    fn new_root<'a, T: Reflectable>(&'a self, unrooted: &JS<T>) -> Root<'a, T> {
        Root::new(self, unrooted)
    }

    fn root_impl(&self, unrooted: RootReference) {
        let current = self.current.get();
        assert!(current < MAX_STACK_ROOTS);
        self.roots[current].set(unrooted);
        self.current.set(current + 1);
    }

    fn root<'a, T: Reflectable>(&self, unrooted: &Root<'a, T>) {
        self.root_impl(RootReference::new(unrooted));
    }

    pub fn root_raw(&self, unrooted: *JSObject) {
        self.root_impl(RootReference(unrooted));
    }

    fn unroot_impl(&self, rooted: RootReference) {
        let mut current = self.current.get();
        assert!(current != 0);
        current -= 1;
        assert!(self.roots[current].get() == rooted);
        self.roots[current].set(RootReference::null());
        self.current.set(current);
    }

    fn unroot<'a, T: Reflectable>(&self, rooted: &Root<'a, T>) {
        self.unroot_impl(RootReference::new(rooted));
    }

    pub fn unroot_raw(&self, rooted: *JSObject) {
        self.unroot_impl(RootReference(rooted));
    }
}

pub struct Root<'a, T> {
    root_list: &'a RootCollection,
    ptr: RefCell<*mut T>,
}

impl<'a, T: Reflectable> Root<'a, T> {
    fn new(roots: &'a RootCollection, unrooted: &JS<T>) -> Root<'a, T> {
        let root = Root {
            root_list: roots,
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

    pub fn root_ref<'b>(&'b self) -> JSRef<'b,T> {
        unsafe {
            JSRef {
                ptr: self.ptr.clone(),
                chain: ::std::cast::transmute_region(&()),
            }
        }
    }
}

#[unsafe_destructor]
impl<'a, T: Reflectable> Drop for Root<'a, T> {
    fn drop(&mut self) {
        self.root_list.unroot(self);
    }
}

impl<'a, T: Reflectable> Reflectable for Root<'a, T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.get().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.get_mut().mut_reflector()
    }
}

/*impl<'a, T> Deref for Root<'a, T> {
    fn deref<'a>(&'a self) -> &'a T {
        self.get()
    }
}

impl<'a, T> DerefMut for Root<'a, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.get_mut()
    }
}*/

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
