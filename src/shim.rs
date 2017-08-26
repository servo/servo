use std::marker::PhantomData;

pub struct NonZeroPtr<T: 'static>(&'static T);

impl<T: 'static> NonZeroPtr<T> {
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        NonZeroPtr(&*ptr)
    }
    pub fn as_ptr(&self) -> *mut T {
        self.0 as *const T as *mut T
    }
}

pub struct Unique<T: 'static> {
    ptr: NonZeroPtr<T>,
    _marker: PhantomData<T>,
}

impl<T: 'static> Unique<T> {
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Unique {
            ptr: NonZeroPtr::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

unsafe impl<T: Send + 'static> Send for Unique<T> { }

unsafe impl<T: Sync + 'static> Sync for Unique<T> { }

pub struct Shared<T: 'static>  {
    ptr: NonZeroPtr<T>,
    _marker: PhantomData<T>,
    // force it to be !Send/!Sync
    _marker2: PhantomData<*const u8>,
}

impl<T: 'static> Shared<T> {
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Shared {
            ptr: NonZeroPtr::new_unchecked(ptr),
            _marker: PhantomData,
            _marker2: PhantomData,
        }
    }

    pub unsafe fn as_mut(&self) -> &mut T {
        &mut *self.ptr.as_ptr()
    }
}

impl<'a, T> From<&'a mut T> for Shared<T> {
    fn from(reference: &'a mut T) -> Self {
        unsafe { Shared::new_unchecked(reference) }
    }
}