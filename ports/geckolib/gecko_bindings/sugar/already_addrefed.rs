use structs::already_AddRefed;
use std::mem;
use std::ops;

impl<T> ops::Deref for already_AddRefed<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        assert!(!self.mRawPtr.is_null());
        unsafe { mem::transmute(self.mRawPtr) }
    }
}

impl<T> ops::DerefMut for already_AddRefed<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        assert!(!self.mRawPtr.is_null());
        unsafe { mem::transmute(self.mRawPtr) }
    }
}

impl<T> already_AddRefed<T> {
    pub fn checked<'a>(&'a self) -> Option<&'a T> {
        // TODO: This can probably just be written as:
        //
        // mem::transmute(self.mRawPtr);
        //
        // since rust optimises the Option<&T> case with None as the null
        // pointer.
        //
        // Nontheless this is probably optimised away, so we just don't rely on
        // it.
        if self.mRawPtr.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.mRawPtr) })
        }
    }

    pub fn checked_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        if self.mRawPtr.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.mRawPtr) })
        }
    }
}
