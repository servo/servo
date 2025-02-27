/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, UnsafeCell};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::{fmt, mem, ptr};

use js::gc::Traceable as JSTraceable;
use js::jsapi::{JSObject, JSTracer};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use style::thread_state;

use crate::conversions::DerivedFrom;
use crate::inheritance::Castable;
use crate::reflector::{DomObject, MutDomObject, Reflector};
use crate::trace::trace_reflector;

/// A rooted value.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
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
    /// It gives out references which cannot outlive this new `Root`.
    ///
    /// # Safety
    /// It must not outlive its associated `RootCollection`.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
        #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
        #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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

impl<T: fmt::Debug + StableTraceObject> fmt::Debug for Root<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: fmt::Debug + DomObject> fmt::Debug for Dom<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// A traced reference to a DOM object
///
/// This type is critical to making garbage collection work with the DOM,
/// but it is very dangerous; if garbage collection happens with a `Dom<T>`
/// on the stack, the `Dom<T>` can point to freed memory.
///
/// This should only be used as a field in other DOM objects.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[repr(transparent)]
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

impl<T> Hash for Dom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state)
    }
}

impl<T> Clone for Dom<T> {
    #[inline]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn clone(&self) -> Self {
        assert_in_script();
        Dom { ptr: self.ptr }
    }
}

impl<T: DomObject> Dom<T> {
    /// Create a `Dom<T>` from a `&T`
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub fn from_ref(obj: &T) -> Dom<T> {
        assert_in_script();
        Dom {
            ptr: ptr::NonNull::from(obj),
        }
    }

    /// Return a rooted version of this DOM object ([`DomRoot<T>`]) suitable for use on the stack.
    pub fn as_rooted(&self) -> DomRoot<T> {
        DomRoot::from_ref(self)
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct MaybeUnreflectedDom<T> {
    ptr: ptr::NonNull<T>,
}

impl<T> MaybeUnreflectedDom<T>
where
    T: DomObject,
{
    /// Create a new MaybeUnreflectedDom value from the given boxed DOM object.
    ///
    /// # Safety
    /// TODO: unclear why this is marked unsafe.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
    /// Treat the given JS object as the reflector of this unreflected object.
    ///
    /// # Safety
    /// obj must point to a valid, non-null JS object.
    pub unsafe fn reflect_with(self, obj: *mut JSObject) -> DomRoot<T> {
        let ptr = self.as_ptr();
        drop(self);
        let root = DomRoot::from_ref(&*ptr);
        root.init_reflector(obj);
        root
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
        unsafe { mem::transmute::<DomRoot<T>, DomRoot<U>>(root) }
    }

    /// Cast a DOM object root downwards to one of the interfaces it might implement.
    pub fn downcast<U>(root: DomRoot<T>) -> Option<DomRoot<U>>
    where
        U: DerivedFrom<T>,
    {
        if root.is::<U>() {
            Some(unsafe { mem::transmute::<DomRoot<T>, DomRoot<U>>(root) })
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

    /// Create a traced version of this rooted object.
    ///
    /// # Safety
    ///
    /// This should never be used to create on-stack values. Instead these values should always
    /// end up as members of other DOM objects.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub fn as_traced(&self) -> Dom<T> {
        Dom::from_ref(self)
    }
}

impl<T> MallocSizeOf for DomRoot<T>
where
    T: DomObject + MallocSizeOf,
{
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
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

impl<T: DomObject> Eq for DomRoot<T> {}

impl<T: DomObject> Hash for DomRoot<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
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

impl RootCollection {
    /// Create an empty collection of roots
    #[allow(clippy::new_without_default)]
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

thread_local!(pub static STACK_ROOTS: Cell<Option<*const RootCollection>> = const { Cell::new(None) });

/// SM Callback that traces the rooted reflectors
///
/// # Safety
/// tracer must point to a valid, non-null JS tracer object.
pub unsafe fn trace_roots(tracer: *mut JSTracer) {
    trace!("tracing stack roots");
    STACK_ROOTS.with(|collection| {
        let collection = &*(*collection.get().unwrap()).roots.get();
        for root in collection {
            (**root).trace(tracer);
        }
    });
}

pub fn assert_in_script() {
    debug_assert!(thread_state::get().is_script());
}
