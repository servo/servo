use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ptr::{read, null};
use std::sync::Arc;

/// Helper trait for conversions between FFI Strong/Borrowed types and Arcs
pub unsafe trait RefCounted where Self: Sized {
    /// Gecko's name for the type
    /// This is equivalent to ArcBox<Self>
    type FFIType: Sized;
    fn with<F, Output>(raw: Borrowed<Self::FFIType>, cb: F) -> Output
               where F: FnOnce(&Arc<Self>) -> Output {
        debug_assert!(!raw.is_null());

        let owned = unsafe { Self::borrowed_into(raw) };
        let result = cb(&owned);
        forget(owned);
        result
    }

    fn maybe_with<F, Output>(maybe_raw: Borrowed<Self::FFIType>, cb: F) -> Output
                         where F: FnOnce(Option<&Arc<Self>>) -> Output {
        let owned = if maybe_raw.is_null() {
            None
        } else {
            Some(unsafe { Self::borrowed_into(maybe_raw) })
        };

        let result = cb(owned.as_ref());
        forget(owned);

        result
    }

    fn into(ptr: Strong<Self::FFIType>) -> Arc<Self> {
        unsafe { transmute(ptr) }
    }

    unsafe fn borrowed_into(ptr: Borrowed<Self::FFIType>) -> Arc<Self> {
        transmute(ptr)
    }

    fn from_arc(owned: Arc<Self>) -> Strong<Self::FFIType> {
        unsafe { transmute(owned) }
    }

    unsafe fn addref(ptr: Borrowed<Self::FFIType>) {
        Self::with(ptr, |arc| forget(arc.clone()));
    }

    unsafe fn release(ptr: Borrowed<Self::FFIType>) {
        let _ = Self::borrowed_into(ptr);
    }

    unsafe fn to_borrowed<'a>(arc: &'a Arc<Self>)
        -> Borrowed<'a, Self::FFIType> {
        let borrowedptr = arc as *const Arc<Self> as *const Borrowed<'a, Self::FFIType>;
        read(borrowedptr)
    }

    fn null_strong() -> Strong<Self::FFIType> {
        unsafe { transmute(null::<Self::FFIType>()) }
    }
}

#[repr(C)]
/// Gecko-FFI-safe borrowed Arc (&T where T is an ArcBox)
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
        self.ptr == null()
    }
}

#[repr(C)]
/// Gecko-FFI-safe Arc (Box<T> where T is an ArcBox; i.e. Arc<U>)
pub struct Strong<T> {
    ptr: *const T,
    _marker: PhantomData<T>,
}
