use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::{read, null};
use std::sync::Arc;

pub unsafe trait RefCounted<'a> where Self: Sized {
    type Strong;
    type Borrowed;
    fn into_strong(x: Arc<Self>) -> Self::Strong;
    fn null_strong() -> Self::Strong;
    /// This should panic if x is null
    fn from_borrowed(x: Self::Borrowed) -> &'a Arc<Self>;
    fn into_borrowed(x: &'a Arc<Self>) -> Self::Borrowed;
    fn from_borrowed_opt(x: Self::Borrowed) -> Option<&'a Arc<Self>> {
        let arc = Self::from_borrowed(x);
        let ptr = &**arc as *const Self;
        if ptr == null() {
            None
        } else {
            Some(arc)
        }
    }
    unsafe fn decref_borrowed(x: Self::Borrowed) where Self: 'a {
        let _: Arc<Self> = ptr::read(Self::from_borrowed(x) as *const Arc<_>);
    }
    fn make_strong(x: Self::Borrowed) -> Self::Strong where Self: 'a {
        let arc = Self::from_borrowed(x).clone();
        Self::into_strong(arc)
    }
}

#[derive(Copy, Clone)]
pub struct ServoComputedValuesBorrowed<'a> {
    ptr: usize, // actual type defined in style crate
    _marker: PhantomData<&'a ()>,
}

impl<'a> ServoComputedValuesBorrowed<'a> {
    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }
}

#[derive(Copy, Clone)]
pub struct RawServoStyleSheetBorrowed<'a> {
    ptr: usize, // actual type defined in style crate
    _marker: PhantomData<&'a ()>,
}

impl<'a> RawServoStyleSheetBorrowed<'a> {
    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }
}
