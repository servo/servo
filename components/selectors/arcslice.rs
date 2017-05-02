/* Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 *
 * See the COPYRIGHT file at the top-level directory of this distribution */
//! A thread-safe reference-counted slice type.
//!
//! Forked from https://github.com/huonw/shared_slice , which doesn't work on
//! rust stable.

use std::{cmp, fmt, ops};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};


/// A reference-counted slice type.
pub struct ArcSlice<T> {
    data: *const [T],
    counts: Arc<Box<[T]>>,
}

unsafe impl<T: Send + Sync> Send for ArcSlice<T> {}
unsafe impl<T: Send + Sync> Sync for ArcSlice<T> {}

/// A non-owning reference-counted slice type.
///
/// This is to `ArcSlice` as `std::sync::Weak` is to `std::sync::Arc`, and
/// allows one to have cyclic references without stopping memory from
/// being deallocated.
pub struct WeakSlice<T> {
    data: *const [T],
    counts: Weak<Box<[T]>>,
}
unsafe impl<T: Send + Sync> Send for WeakSlice<T> {}
unsafe impl<T: Send + Sync> Sync for WeakSlice<T> {}

impl<T> ArcSlice<T> {
    /// Construct a new `ArcSlice` containing the elements of `slice`.
    ///
    /// This reuses the allocation of `slice`.
    pub fn new(slice: Box<[T]>) -> ArcSlice<T> {
        ArcSlice {
            data: &*slice,
            counts: Arc::new(slice),
        }
    }

    /// Downgrade self into a weak slice.
    pub fn downgrade(&self) -> WeakSlice<T> {
        WeakSlice {
            data: self.data,
            counts: Arc::downgrade(&self.counts)
        }
    }

    /// Construct a new `ArcSlice` that only points to elements at
    /// indices `lo` (inclusive) through `hi` (exclusive).
    ///
    /// This consumes `self` to avoid unnecessary reference-count
    /// modifications. Use `.clone()` if it is necessary to refer to
    /// `self` after calling this.
    ///
    /// # Panics
    ///
    /// Panics if `lo > hi` or if either are strictly greater than
    /// `self.len()`.
    pub fn slice(mut self, lo: usize, hi: usize) -> ArcSlice<T> {
        self.data = &self[lo..hi];
        self
    }
    /// Construct a new `ArcSlice` that only points to elements at
    /// indices up to `hi` (exclusive).
    ///
    /// This consumes `self` to avoid unnecessary reference-count
    /// modifications. Use `.clone()` if it is necessary to refer to
    /// `self` after calling this.
    ///
    /// # Panics
    ///
    /// Panics if `hi > self.len()`.
    pub fn slice_to(self, hi: usize) -> ArcSlice<T> {
        self.slice(0, hi)
    }
    /// Construct a new `ArcSlice` that only points to elements at
    /// indices starting at  `lo` (inclusive).
    ///
    /// This consumes `self` to avoid unnecessary reference-count
    /// modifications. Use `.clone()` if it is necessary to refer to
    /// `self` after calling this.
    ///
    /// # Panics
    ///
    /// Panics if `lo > self.len()`.
    pub fn slice_from(self, lo: usize) -> ArcSlice<T> {
        let hi = self.len();
        self.slice(lo, hi)
    }
}

impl<T> Clone for ArcSlice<T> {
    fn clone(&self) -> ArcSlice<T> {
        ArcSlice {
            data: self.data,
            counts: self.counts.clone()
        }
    }
}

impl<T> ops::Deref for ArcSlice<T> {
    type Target = [T];
    fn deref<'a>(&'a self) -> &'a [T] {
        unsafe { &*self.data }
    }
}

impl<T> AsRef<[T]> for ArcSlice<T> {
    fn as_ref(&self) -> &[T] { &**self }
}

impl<T: PartialEq> PartialEq for ArcSlice<T> {
    fn eq(&self, other: &ArcSlice<T>) -> bool { **self == **other }
    fn ne(&self, other: &ArcSlice<T>) -> bool { **self != **other }
}
impl<T: Eq> Eq for ArcSlice<T> {}

impl<T: PartialOrd> PartialOrd for ArcSlice<T> {
    fn partial_cmp(&self, other: &ArcSlice<T>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
    fn lt(&self, other: &ArcSlice<T>) -> bool { **self < **other }
    fn le(&self, other: &ArcSlice<T>) -> bool { **self <= **other }
    fn gt(&self, other: &ArcSlice<T>) -> bool { **self > **other }
    fn ge(&self, other: &ArcSlice<T>) -> bool { **self >= **other }
}
impl<T: Ord> Ord for ArcSlice<T> {
    fn cmp(&self, other: &ArcSlice<T>) -> cmp::Ordering { (**self).cmp(&**other) }
}

impl<T: Hash> Hash for ArcSlice<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T: fmt::Debug> fmt::Debug for ArcSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> WeakSlice<T> {
    /// Attempt to upgrade `self` to a strongly-counted `ArcSlice`.
    ///
    /// Returns `None` if this is not possible (the data has already
    /// been freed).
    pub fn upgrade(&self) -> Option<ArcSlice<T>> {
        self.counts.upgrade().map(|counts| {
            ArcSlice {
                data: self.data,
                counts: counts
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::cmp::Ordering;
    use std::sync::{Arc, Mutex};
    use super::{ArcSlice, WeakSlice};
    #[test]
    fn clone() {
        let x = ArcSlice::new(Box::new([Cell::new(false)]));
        let y = x.clone();

        assert_eq!(x[0].get(), false);
        assert_eq!(y[0].get(), false);

        x[0].set(true);
        assert_eq!(x[0].get(), true);
        assert_eq!(y[0].get(), true);
    }

    #[test]
    fn test_upgrade_downgrade() {
        let x = ArcSlice::new(Box::new([1]));
        let y: WeakSlice<_> = x.downgrade();

        assert_eq!(y.upgrade(), Some(x.clone()));

        drop(x);

        assert!(y.upgrade().is_none())
    }

    #[test]
    fn test_total_cmp() {
        let x = ArcSlice::new(Box::new([1, 2, 3]));
        let y = ArcSlice::new(Box::new([1, 2, 3]));
        let z = ArcSlice::new(Box::new([1, 2, 4]));

        assert_eq!(x, x);
        assert_eq!(x, y);
        assert!(x != z);
        assert!(y != z);

        assert!(x < z);
        assert!(x <= z);
        assert!(!(x > z));
        assert!(!(x >= z));

        assert!(!(z < x));
        assert!(!(z <= x));
        assert!(z > x);
        assert!(z >= x);

        assert_eq!(x.partial_cmp(&x), Some(Ordering::Equal));
        assert_eq!(x.partial_cmp(&y), Some(Ordering::Equal));
        assert_eq!(x.partial_cmp(&z), Some(Ordering::Less));
        assert_eq!(z.partial_cmp(&y), Some(Ordering::Greater));

        assert_eq!(x.cmp(&x), Ordering::Equal);
        assert_eq!(x.cmp(&y), Ordering::Equal);
        assert_eq!(x.cmp(&z), Ordering::Less);
        assert_eq!(z.cmp(&y), Ordering::Greater);
    }

    #[test]
    fn test_partial_cmp() {
        use std::f64;
        let x = ArcSlice::new(Box::new([1.0, f64::NAN]));
        let y = ArcSlice::new(Box::new([1.0, f64::NAN]));
        let z = ArcSlice::new(Box::new([2.0, f64::NAN]));
        let w = ArcSlice::new(Box::new([f64::NAN, 1.0]));
        assert!(!(x == y));
        assert!(x != y);

        assert!(!(x < y));
        assert!(!(x <= y));
        assert!(!(x > y));
        assert!(!(x >= y));

        assert!(x < z);
        assert!(x <= z);
        assert!(!(x > z));
        assert!(!(x >= z));

        assert!(!(z < w));
        assert!(!(z <= w));
        assert!(!(z > w));
        assert!(!(z >= w));

        assert_eq!(x.partial_cmp(&x), None);
        assert_eq!(x.partial_cmp(&y), None);
        assert_eq!(x.partial_cmp(&z), Some(Ordering::Less));
        assert_eq!(z.partial_cmp(&x), Some(Ordering::Greater));

        assert_eq!(x.partial_cmp(&w), None);
        assert_eq!(y.partial_cmp(&w), None);
        assert_eq!(z.partial_cmp(&w), None);
        assert_eq!(w.partial_cmp(&w), None);
    }

    #[test]
    fn test_show() {
        let x = ArcSlice::new(Box::new([1, 2]));
        assert_eq!(format!("{:?}", x), "[1, 2]");

        let y: ArcSlice<i32> = ArcSlice::new(Box::new([]));
        assert_eq!(format!("{:?}", y), "[]");
    }

    #[test]
    fn test_slice() {
        let x = ArcSlice::new(Box::new([1, 2, 3]));
        let real = [1, 2, 3];
        for i in 0..3 + 1 {
            for j in i..3 + 1 {
                let slice: ArcSlice<_> = x.clone().slice(i, j);
                assert_eq!(&*slice, &real[i..j]);
            }
            assert_eq!(&*x.clone().slice_to(i), &real[..i]);
            assert_eq!(&*x.clone().slice_from(i), &real[i..]);
        }
    }


    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Send>() {}

        assert_send::<ArcSlice<u8>>();
        assert_sync::<ArcSlice<u8>>();
        assert_send::<WeakSlice<u8>>();
        assert_sync::<WeakSlice<u8>>();
    }

    #[test]
    fn test_drop() {
        let drop_flag = Arc::new(Mutex::new(0));
        struct Foo(Arc<Mutex<i32>>);

        impl Drop for Foo {
            fn drop(&mut self) {
                let mut n = self.0.lock().unwrap();
                *n += 1;
            }
        }

        let whole = ArcSlice::new(Box::new([Foo(drop_flag.clone()), Foo(drop_flag.clone())]));

        drop(whole);
        assert_eq!(*drop_flag.lock().unwrap(), 2);

        *drop_flag.lock().unwrap() = 0;

        let whole = ArcSlice::new(Box::new([Foo(drop_flag.clone()), Foo(drop_flag.clone())]));
        let part = whole.slice(1, 2);
        drop(part);
        assert_eq!(*drop_flag.lock().unwrap(), 2);
    }
}
