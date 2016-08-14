use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ptr;
use std::sync::Arc;

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
///
/// Should be implemented by types which are passed over FFI as Arcs
/// via Strong and Borrowed
pub unsafe trait HasArcFFI where Self: Sized {
    /// Gecko's name for the type
    /// This is equivalent to ArcInner<Self>
    type FFIType: Sized;

    /// Given a non-null borrowed FFI reference, this produces a temporary
    /// Arc which is borrowed by the given closure and used.
    /// Panics on null.
    fn with<F, Output>(raw: Borrowed<Self::FFIType>, cb: F) -> Output
               where F: FnOnce(&Arc<Self>) -> Output {
        assert!(!raw.is_null());

        let owned = unsafe { Self::borrowed_into(raw) };
        let result = cb(&owned);
        // this isn't unwind safe, do not use without panic=abort
        forget(owned);
        result
    }

    /// Given a maybe-null borrowed FFI reference, this produces a temporary
    /// Option<Arc> (None if null) which is borrowed by the given closure and used
    fn maybe_with<F, Output>(maybe_raw: Borrowed<Self::FFIType>, cb: F) -> Output
                         where F: FnOnce(Option<&Arc<Self>>) -> Output {
        let owned = if maybe_raw.is_null() {
            None
        } else {
            Some(unsafe { Self::borrowed_into(maybe_raw) })
        };

        let result = cb(owned.as_ref());
        // this isn't unwind safe, do not use without panic=abort
        forget(owned);

        result
    }

    /// Given a non-null strong FFI reference, converts it into an Arc.
    /// Panics on null.
    fn into(ptr: Strong<Self::FFIType>) -> Arc<Self> {
        assert!(!ptr.is_null());
        unsafe { transmute(ptr) }
    }

    /// Given a borrowed FFI reference, converts it to an Arc.
    /// Unsafe to run on null values.
    /// Can cause use-after-free if the Arc is allowed to have its destructor
    /// run, please forget() arcs created by this
    unsafe fn borrowed_into(ptr: Borrowed<Self::FFIType>) -> Arc<Self> {
        transmute(ptr)
    }

    /// Converts an Arc into a strong FFI reference.
    fn from_arc(owned: Arc<Self>) -> Strong<Self::FFIType> {
        unsafe { transmute(owned) }
    }

    /// Artificially increments the refcount of a borrowed Arc over FFI.
    unsafe fn addref(ptr: Borrowed<Self::FFIType>) {
        Self::with(ptr, |arc| forget(arc.clone()));
    }

    /// Given a non-null borrowed FFI reference, decrements the refcount.
    /// Unsafe since it doesn't consume the backing Arc. Run it only when you
    /// know that a strong reference to the backing Arc is disappearing
    /// (usually on the C++ side) without running the Arc destructor.
    unsafe fn release(ptr: Borrowed<Self::FFIType>) {
        assert!(!ptr.is_null());
        let _ = Self::borrowed_into(ptr);
    }

    /// Produces a borrowed FFI reference by borrowing an Arc.
    fn to_borrowed<'a>(arc: &'a Arc<Self>)
        -> Borrowed<'a, Self::FFIType> {
        let borrowedptr = arc as *const Arc<Self> as *const Borrowed<'a, Self::FFIType>;
        unsafe { ptr::read(borrowedptr) }
    }

    /// Produces a null strong FFI reference
    fn null_strong() -> Strong<Self::FFIType> {
        unsafe { transmute(ptr::null::<Self::FFIType>()) }
    }
}

#[repr(C)]
/// Gecko-FFI-safe borrowed Arc (&T where T is an ArcInner).
/// This can be null.
pub struct Borrowed<'a, T: 'a> {
    ptr: *const T,
    marker: PhantomData<&'a T>,
}

// manual impls because derive doesn't realize that `T: Clone` isn't necessary
impl<'a, T> Copy for Borrowed<'a, T> {}

impl<'a, T> Clone for Borrowed<'a, T> {
    fn clone(&self) -> Self {*self}
}

impl<'a, T> Borrowed<'a, T> {
    pub fn is_null(&self) -> bool {
        self.ptr == ptr::null()
    }
}

#[repr(C)]
/// Gecko-FFI-safe Arc (T is an ArcInner).
/// This can be null.
pub struct Strong<T> {
    ptr: *const T,
    _marker: PhantomData<T>,
}

impl<T> Strong<T> {
    pub fn is_null(&self) -> bool {
        self.ptr == ptr::null()
    }
}
