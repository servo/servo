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
//! - All functions take `&JSRef<T>` arguments, to ensure that they will remain uncollected for
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
use js::{ContextFriendFields, THING_ROOT_OBJECT, THING_ROOT_ID, THING_ROOT_VALUE};
use js::THING_ROOT_STRING;
use js::glue::{insertObjectLinkedListElement, getPersistentRootedObjectList};
use js::glue::{objectIsPoisoned, objectRelocate, objectNeedsPostBarrier, objectPostBarrier};
use js::jsapi::{JSContext, JSObject, JS_IsInRequest, JS_GetRuntime, Handle, MutableHandle, jsid};
use js::jsapi::{JSFunction, JSString, JSPropertyDescriptor};
use js::jsval::JSVal;
use layout_interface::TrustedNodeAddress;
use script_task::StackRoots;

use libc;
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::intrinsics::TypeId;
use std::kinds::marker::ContravariantLifetime;
use std::mem;
use std::ptr;

/// A type that represents a JS-owned value that is rooted for the lifetime of this value.
/// Importantly, it requires explicit rooting in order to interact with the inner value.
/// Can be assigned into JS-owned member fields (i.e. `JS<T>` types) safely via the
/// `JS<T>::assign` method or `OptionalSettable::assign` (for `Option<JS<T>>` fields).
#[allow(unrooted_must_root)]
pub struct Temporary<T> {
    inner: JS<T>,
    _js_ptr: Box<PersistentRootedObjectElement>,
}

struct PersistentRootedObjectElement {
    next: *mut PersistentRootedObjectElement,
    prev: *mut PersistentRootedObjectElement,
    _isSentinel: bool,
    _ptr: *mut JSObject,
}

impl PersistentRootedObjectElement {
    fn new(ptr: *mut JSObject) -> PersistentRootedObjectElement {
        PersistentRootedObjectElement {
            next: ptr::mut_null(),
            prev: ptr::mut_null(),
            _isSentinel: false,
            _ptr: ptr,
        }
    }

    fn init(&mut self) {
        assert!(self.next.is_null());
        assert!(self.prev.is_null());
        let roots = StackRoots.get().unwrap();
        let rt = unsafe { JS_GetRuntime((**roots).cx) };
        self.next = self as *mut _;
        self.prev = self as *mut _;
        unsafe {
            let list = getPersistentRootedObjectList(rt);
            insertObjectLinkedListElement(list, self as *mut _ as *mut _);
        }
    }
}

impl Drop for PersistentRootedObjectElement {
    fn drop(&mut self) {
        assert!(!self.next.is_null());
        assert!(!self.prev.is_null());
        if self.next != self as *mut _ {
            unsafe {
                (*self.prev).next = self.next;
                (*self.next).prev = self.prev;
                self.next = self as *mut _;
                self.prev = self as *mut _;
            }
        }
    }
}

impl<T> PartialEq for Temporary<T> {
    fn eq(&self, other: &Temporary<T>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Reflectable> Temporary<T> {
    /// Create a new `Temporary` value from a JS-owned value.
    pub fn new(inner: JS<T>) -> Temporary<T> {
        let mut js_ptr =
            box PersistentRootedObjectElement::new(inner.reflector().get_jsobject());
        js_ptr.init();
        Temporary {
            inner: inner,
            _js_ptr: js_ptr,
        }
    }

    /// Create a new `Temporary` value from a rooted value.
    pub fn from_rooted<'a>(root: &JSRef<'a, T>) -> Temporary<T> {
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
    pub fn from_rooted(root: &T) -> JS<U> {
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
            (*self.ptr).reflector()
        }
    }
}

impl<T: Reflectable> JS<T> {
    /// Returns an unsafe pointer to the interior of this JS object without touching the borrow
    /// flags. This is the only method that be safely accessed from layout. (The fact that this
    /// is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *const T {
        self.ptr
    }
}

impl<From, To> JS<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute_copy(&self) -> JS<To> {
        mem::transmute_copy(self)
    }
}

/// A mutable JS&lt;T&gt; value, with nullability represented by an enclosing
/// Option wrapper. Must be used in place of traditional internal mutability
/// to ensure that the proper GC barriers are enforced.
pub struct MutNullableJS<T> {
    ptr: Cell<Option<JS<T>>>
}

impl<T: Assignable<U>, U: Reflectable> MutNullableJS<U> {
    pub fn new(initial: Option<T>) -> MutNullableJS<U> {
        MutNullableJS {
            ptr: Cell::new(initial.map(|initial| unsafe { initial.get_js() }))
        }
    }
}

impl<T> Default for MutNullableJS<T> {
    fn default() -> MutNullableJS<T> {
        MutNullableJS {
            ptr: Cell::new(None::<JS<T>>)
        }
    }
}

impl<T: Reflectable> MutNullableJS<T> {
    /// Store an unrooted value in this field. This is safe under the assumption that JS<T>
    /// values are only used as fields in DOM types that are reachable in the GC graph,
    /// so this unrooted value becomes transitively rooted for the lifetime of its new owner.
    pub fn assign<U: Assignable<T>>(&self, val: Option<U>) {
        let reflector = val.as_ref()
                           .map(|val| unsafe { val.get_js() }.reflector().get_jsobject())
                           .unwrap_or(ptr::mut_null());
        unsafe {
            assert!(!objectIsPoisoned(reflector));
        }
        if unsafe { objectNeedsPostBarrier(reflector) } {
            self.ptr.set(val.as_ref().map(|val| unsafe { val.get_js() }));
            let raw_ptr = self.ptr.get()
                                  .map(|val| val.reflector().rootable())
                                  .unwrap_or(ptr::mut_null());
            unsafe {
                objectPostBarrier(raw_ptr);
            }
        } else if unsafe { objectNeedsPostBarrier(self.ptr
                                                      .get()
                                                      .map(|val| val.reflector().get_jsobject())
                                                      .unwrap_or(ptr::mut_null())) } {
            let raw_ptr = self.ptr.get()
                                  .map(|val| val.reflector().rootable())
                                  .unwrap_or(ptr::mut_null());
            unsafe {
                objectRelocate(raw_ptr);
                self.ptr.set(val.map(|val| val.get_js()));
            }
        } else {
            self.ptr.set(val.map(|val| unsafe { val.get_js() }));
        }
    }

    /// Set the inner value to null, without making API users jump through useless
    /// type-ascription hoops.
    pub fn clear(&self) {
        self.assign(None::<JS<T>>);
    }

    /// Retrieve a copy of the current optional inner value.
    pub fn get(&self) -> Option<Temporary<T>> {
        self.ptr.get().map(|inner| Temporary::new(inner))
    }

    /// Retrieve a copy of the inner optional JS&lt;T&gt;. For use by layout, which
    /// can't use safe types like Temporary.
    pub unsafe fn get_inner(&self) -> Option<JS<T>> {
        self.ptr.get()
    }
}

/// Get an `Option<JSRef<T>>` out of an `Option<Root<T>>`
pub trait RootedReference<T> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>>;
    fn init(&self);
}

impl<'a, 'b, T: Reflectable> RootedReference<T> for Option<Root<'a, 'b, T>> {
    fn root_ref<'a>(&'a self) -> Option<JSRef<'a, T>> {
        self.as_ref().map(|root| root.root_ref())
    }

    fn init(&self) {
        self.as_ref().map(|root| root.init());
    }
}

/// Get an `Option<Option<JSRef<T>>>` out of an `Option<Option<Root<T>>>`
pub trait OptionalRootedReference<T> {
    fn root_ref<'a>(&'a self) -> Option<Option<JSRef<'a, T>>>;
    fn init(&self);
}

impl<'a, 'b, T: Reflectable> OptionalRootedReference<T> for Option<Option<Root<'a, 'b, T>>> {
    fn root_ref<'a>(&'a self) -> Option<Option<JSRef<'a, T>>> {
        self.as_ref().map(|inner| inner.root_ref())
    }

    fn init(&self) {
        self.as_ref().map(|inner| inner.init());
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
        self.as_ref().map(|inner| JS::from_rooted(inner))
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
    roots: RefCell<Vec<*mut libc::c_void>>,
    cx: *mut JSContext,
}

impl RootCollection {
    /// Create an empty collection of roots
    pub fn new(cx: *mut JSContext) -> RootCollection {
        RootCollection {
            roots: RefCell::new(vec!()),
            cx: cx,
        }
    }

    /// Create a new stack-bounded root that will not outlive this collection
    #[allow(unrooted_must_root)]
    fn new_root<'a, 'b, T: Reflectable>(&'a self, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        Root::new(self, unrooted)
    }

    /// Create a new stack-bounded root that will not outlive this collection
    fn new_raw_root<'a, 'b, S: RootableSMPointerType, T: RootablePointer<S>>(&'a self, unrooted: T) -> Root<'a, 'b, libc::c_void, *mut S> {
        Root::from_raw_ptr(self, unrooted.pointer())
    }

    /// Create a new stack-bounded root that will not outlive this collection
    fn new_raw_value_root<'a, 'b, S: RootableSMValueType+'static>(&'a self, unrooted: S) -> Root<'a, 'b, libc::c_void, S> {
        Root::from_raw_value(self, unrooted)
    }

    /// Track a stack-based root to ensure LIFO root ordering
    fn root<'a, 'b, T, S>(&self, untracked: &Root<'a, 'b, T, S>) {
        let mut roots = self.roots.borrow_mut();
        let ptr = &untracked.js_ptr as *const S as *mut libc::c_void;
        roots.push(ptr);
        debug!("  rooting {:p}", ptr);
    }

    /// Stop tracking a stack-based root, asserting if LIFO root ordering has been violated
    fn unroot<'a, 'b, T, S>(&self, rooted: &Root<'a, 'b, T, S>) {
        let mut roots = self.roots.borrow_mut();
        let expected = &rooted.js_ptr as *const S as *mut libc::c_void;
        let actual = *roots.last().unwrap();
        debug!("unrooting {:p} (expecting {:p})", actual, expected);
        assert!(actual == expected);
        roots.pop().unwrap();
    }
}

/// A rooted JS value. The JS value is pinned for the duration of this object's lifetime;
/// roots are additive, so this object's destruction will not invalidate other roots
/// for the same JS value. `Root`s cannot outlive the associated `RootCollection` object.
/// Attempts to transfer ownership of a `Root` via moving will trigger dynamic unrooting
/// failures due to incorrect ordering.
pub struct Root<'a, 'b, T, S=*mut JSObject> {
    stack: Cell<*mut *const *const libc::c_void>,
    prev: Cell<*const *const libc::c_void>,
    js_ptr: S,
    _mCheckNotUsedAsTemporary_statementDone: bool,
    /// List that ensures correct dynamic root ordering
    root_list: &'a RootCollection,
    /// Reference to rooted value that must not outlive this container
    jsref: JSRef<'b, T>,
}

impl<'a, 'b, T, S:'static> Root<'a, 'b, T, S> {
    pub fn init(&self) {
        assert!(self.stack.get().is_null());
        let this_id = TypeId::of::<S>();
        let kind = if this_id == TypeId::of::<*mut JSObject>() {
            THING_ROOT_OBJECT
        } else if this_id == TypeId::of::<*mut JSString>() {
            THING_ROOT_STRING
        } else if this_id == TypeId::of::<JSVal>() {
            THING_ROOT_VALUE
        } else if this_id == TypeId::of::<jsid>() {
            THING_ROOT_ID
        } else {
            fail!("unknown type being rooted")
        };
        unsafe {
            assert!(JS_IsInRequest(JS_GetRuntime(self.root_list.cx)));
            self.root_list.root(self);
            let cxfields: *mut ContextFriendFields = mem::transmute(self.root_list.cx);
            self.stack.set(&mut (*cxfields).thingGCRooters[kind as uint]);
            self.prev.set(*self.stack.get());
            *self.stack.get() = self as *const Root<T, S> as *const *const libc::c_void;
        }
    }
}

trait RootableSMType {}
impl RootableSMType for JSVal {}
impl RootableSMType for JSObject {}
impl RootableSMType for jsid {}
impl RootableSMType for JSFunction {}
impl RootableSMType for JSString {}
impl RootableSMType for JSPropertyDescriptor {}

trait RootableSMPointerType: RootableSMType {}
trait RootableSMValueType: RootableSMType {}

impl RootableSMPointerType for JSObject {}
impl RootableSMPointerType for JSFunction {}
impl RootableSMPointerType for JSString {}

impl RootableSMValueType for JSVal {}
impl RootableSMValueType for jsid {}
impl RootableSMValueType for JSPropertyDescriptor {}

pub trait RootablePointer<S: RootableSMPointerType> {
    fn root_ptr<'a, 'b>(self) -> Root<'a, 'b, libc::c_void, *mut S>;
    fn pointer(&self) -> *mut S;
}

pub trait RootableValue<S: RootableSMValueType> {
    fn root_value<'a, 'b>(self) -> Root<'a, 'b, libc::c_void, S>;
}

fn raw_root_ptr_impl<'a, 'b, S: RootableSMPointerType, T: RootablePointer<S>>(ptr: T) -> Root<'a, 'b, libc::c_void, *mut S> {
    let collection = StackRoots.get().unwrap();
    unsafe {
        (**collection).new_raw_root(ptr.pointer())
    }
}

fn raw_root_value_impl<'a, 'b, S: RootableSMValueType+'static>(val: S) -> Root<'a, 'b, libc::c_void, S> {
    let collection = StackRoots.get().unwrap();
    unsafe {
        (**collection).new_raw_value_root(val)
    }
}

impl<S: RootableSMPointerType> RootablePointer<S> for *mut S {
    fn root_ptr<'a, 'b>(self) -> Root<'a, 'b, libc::c_void, *mut S> {
        raw_root_ptr_impl(self)
    }

    fn pointer(&self) -> *mut S {
        *self
    }
}

impl<S: RootableSMValueType+'static> RootableValue<S> for S {
    fn root_value<'a, 'b>(self) -> Root<'a, 'b, libc::c_void, S> {
        raw_root_value_impl(self)
    }
}

impl<'a, 'b, S: RootableSMPointerType> Root<'a, 'b, libc::c_void, *mut S> {
    fn from_raw_ptr(roots: &'a RootCollection, unrooted: *mut S) -> Root<'a, 'b, libc::c_void, *mut S> {
        Root {
            stack: Cell::new(ptr::mut_null()),
            prev: Cell::new(ptr::null()),
            root_list: roots,
            _mCheckNotUsedAsTemporary_statementDone: true,
            jsref: JSRef {
                ptr: ptr::null(),
                chain: ContravariantLifetime,
            },
            js_ptr: unrooted,
        }
    }
}

impl<'a, 'b, S: RootableSMPointerType+'static> Root<'a, 'b, libc::c_void, *mut S> {
    pub fn handle<'c>(&'c self) -> Handle<'c, *mut S> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
        Handle {
            unnamed_field1: &self.js_ptr,
        }
    }

    pub fn mut_handle<'c>(&'c mut self) -> MutableHandle<'c, *mut S> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
        MutableHandle {
            unnamed_field1: &mut self.js_ptr
        }
    }

    pub fn raw<'c>(&'c self) -> &'c *mut S {
        &self.js_ptr
    }
}

impl<'a, 'b, S: RootableSMValueType+'static> Root<'a, 'b, libc::c_void, S> {
    fn from_raw_value(roots: &'a RootCollection, unrooted: S) -> Root<'a, 'b, libc::c_void, S> {
        Root {
            stack: Cell::new(ptr::mut_null()),
            prev: Cell::new(ptr::null()),
            root_list: roots,
            _mCheckNotUsedAsTemporary_statementDone: true,
            jsref: JSRef {
                ptr: ptr::null(),
                chain: ContravariantLifetime,
            },
            js_ptr: unrooted,
        }
    }

    pub fn handle_<'c>(&'c self) -> Handle<'c, S> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
        Handle {
            unnamed_field1: &self.js_ptr,
        }
    }

    pub fn mut_handle_<'c>(&'c mut self) -> MutableHandle<'c, S> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
        MutableHandle {
            unnamed_field1: &mut self.js_ptr,
        }
    }

    pub fn raw_<'c>(&'c self) -> &'c S {
        &self.js_ptr
    }
}

impl<'a, 'b, T: Reflectable> Root<'a, 'b, T> {
    /// Create a new stack-bounded root for the provided JS-owned value.
    /// It cannot not outlive its associated `RootCollection`, and it contains a `JSRef`
    /// which cannot outlive this new `Root`.
    fn new(roots: &'a RootCollection, unrooted: &JS<T>) -> Root<'a, 'b, T> {
        Root {
            stack: Cell::new(ptr::mut_null()),
            prev: Cell::new(ptr::null()),
            root_list: roots,
            _mCheckNotUsedAsTemporary_statementDone: true,
            jsref: JSRef {
                ptr: unrooted.ptr,
                chain: ContravariantLifetime,
            },
            js_ptr: unrooted.reflector().get_jsobject(),
        }
    }

    /// Obtain a safe reference to the wrapped JS owned-value that cannot outlive
    /// the lifetime of this root.
    pub fn root_ref<'b>(&'b self) -> JSRef<'b,T> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
        self.jsref.clone()
    }
}

#[unsafe_destructor]
impl<'a, 'b, T, S> Drop for Root<'a, 'b, T, S> {
    fn drop(&mut self) {
        unsafe {
            assert!(self.stack.get() != ptr::mut_null(), "Uninitialized root encountered");
            assert!(*self.stack.get() == self as *mut Root<T, S> as *const *const libc::c_void);
            *self.stack.get() = self.prev.get();
        }
        self.root_list.unroot(self);
    }
}

impl<'a, 'b, T: Reflectable> Deref<JSRef<'b, T>> for Root<'a, 'b, T> {
    fn deref<'c>(&'c self) -> &'c JSRef<'b, T> {
        if self.stack.get() == ptr::mut_null() {
            self.init();
        }
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
    pub unsafe fn transmute<'b, To>(&'b self) -> &'b JSRef<'a, To> {
        mem::transmute(self)
    }

    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute_mut<'b, To>(&'b mut self) -> &'b mut JSRef<'a, To> {
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
