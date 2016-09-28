// Copyright 2012-2014 The Rust Project Developers.
// See http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A fork of std::cell::RefCell that makes `as_unsafe_cell` usable on stable Rust.
//!
//! FIXME(https://github.com/rust-lang/rust/issues/27708): Remove this
//! (revert commit f7f81e0ed0b62402db291e28a9bb16f7194ebf78 / PR #11835)
//! when `std::cell::RefCell::as_unsafe_cell` is in Rust’s stable channel.

#![allow(unsafe_code)]

use std::cell::{UnsafeCell, Cell};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A fork of std::cell::RefCell that makes `as_unsafe_cell` usable on stable Rust.
///
/// FIXME(https://github.com/rust-lang/rust/issues/27708): Remove this
/// (revert commit f7f81e0ed0b62402db291e28a9bb16f7194ebf78 / PR #11835)
/// when `std::cell::RefCell::as_unsafe_cell` is in Rust’s stable channel.
pub struct RefCell<T: ?Sized> {
    borrow: Cell<BorrowFlag>,
    value: UnsafeCell<T>,
}

/// An enumeration of values returned from the `state` method on a `RefCell<T>`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BorrowState {
    /// The cell is currently being read, there is at least one active `borrow`.
    Reading,
    /// The cell is currently being written to, there is an active `borrow_mut`.
    Writing,
    /// There are no outstanding borrows on this cell.
    Unused,
}

/// An error returned by [`RefCell::try_borrow`](struct.RefCell.html#method.try_borrow).
pub struct BorrowError<'a, T: 'a + ?Sized> {
    marker: PhantomData<&'a RefCell<T>>,
}

impl<'a, T: ?Sized> Debug for BorrowError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BorrowError").finish()
    }
}

impl<'a, T: ?Sized> Display for BorrowError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("already mutably borrowed", f)
    }
}

/// An error returned by [`RefCell::try_borrow_mut`](struct.RefCell.html#method.try_borrow_mut).
pub struct BorrowMutError<'a, T: 'a + ?Sized> {
    marker: PhantomData<&'a RefCell<T>>,
}

impl<'a, T: ?Sized> Debug for BorrowMutError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BorrowMutError").finish()
    }
}

impl<'a, T: ?Sized> Display for BorrowMutError<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("already borrowed", f)
    }
}

// Values [1, MAX-1] represent the number of `Ref` active
// (will not outgrow its range since `usize` is the size of the address space)
type BorrowFlag = usize;
const UNUSED: BorrowFlag = 0;
const WRITING: BorrowFlag = !0;

impl<T> RefCell<T> {
    /// Creates a new `RefCell` containing `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    /// ```
    #[inline]
    pub fn new(value: T) -> RefCell<T> {
        RefCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(UNUSED),
        }
    }

    /// Consumes the `RefCell`, returning the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// let five = c.into_inner();
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        // Since this function takes `self` (the `RefCell`) by value, the
        // compiler statically verifies that it is not currently borrowed.
        // Therefore the following assertion is just a `debug_assert!`.
        debug_assert!(self.borrow.get() == UNUSED);
        unsafe { self.value.into_inner() }
    }
}

impl<T: ?Sized> RefCell<T> {
    /// Query the current state of this `RefCell`
    ///
    /// The returned value can be dispatched on to determine if a call to
    /// `borrow` or `borrow_mut` would succeed.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(borrow_state)]
    ///
    /// use std::cell::{BorrowState, RefCell};
    ///
    /// let c = RefCell::new(5);
    ///
    /// match c.borrow_state() {
    ///     BorrowState::Writing => println!("Cannot be borrowed"),
    ///     BorrowState::Reading => println!("Cannot be borrowed mutably"),
    ///     BorrowState::Unused => println!("Can be borrowed (mutably as well)"),
    /// }
    /// ```
    #[inline]
    pub fn borrow_state(&self) -> BorrowState {
        match self.borrow.get() {
            WRITING => BorrowState::Writing,
            UNUSED => BorrowState::Unused,
            _ => BorrowState::Reading,
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// let borrowed_five = c.borrow();
    /// let borrowed_five2 = c.borrow();
    /// ```
    ///
    /// An example of panic:
    ///
    /// ```
    /// use std::cell::RefCell;
    /// use std::thread;
    ///
    /// let result = thread::spawn(move || {
    ///    let c = RefCell::new(5);
    ///    let m = c.borrow_mut();
    ///
    ///    let b = c.borrow(); // this causes a panic
    /// }).join();
    ///
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn borrow(&self) -> Ref<T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_borrow)]
    ///
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(c.try_borrow().is_err());
    /// }
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow().is_ok());
    /// }
    /// ```
    #[inline]
    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError<T>> {
        match BorrowRef::new(&self.borrow) {
            Some(b) => Ok(Ref {
                value: unsafe { &*self.value.get() },
                borrow: b,
            }),
            None => Err(BorrowError { marker: PhantomData }),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// *c.borrow_mut() = 7;
    ///
    /// assert_eq!(*c.borrow(), 7);
    /// ```
    ///
    /// An example of panic:
    ///
    /// ```
    /// use std::cell::RefCell;
    /// use std::thread;
    ///
    /// let result = thread::spawn(move || {
    ///    let c = RefCell::new(5);
    ///    let m = c.borrow();
    ///
    ///    let b = c.borrow_mut(); // this causes a panic
    /// }).join();
    ///
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn borrow_mut(&self) -> RefMut<T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value cannot be borrowed
    /// while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_borrow)]
    ///
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow_mut().is_err());
    /// }
    ///
    /// assert!(c.try_borrow_mut().is_ok());
    /// ```
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError<T>> {
        match BorrowRefMut::new(&self.borrow) {
            Some(b) => Ok(RefMut {
                value: unsafe { &mut *self.value.get() },
                borrow: b,
            }),
            None => Err(BorrowMutError { marker: PhantomData }),
        }
    }

    /// Returns a reference to the underlying `UnsafeCell`.
    ///
    /// This can be used to circumvent `RefCell`'s safety checks.
    ///
    /// This function is `unsafe` because `UnsafeCell`'s field is public.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(as_unsafe_cell)]
    ///
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    /// let c = unsafe { c.as_unsafe_cell() };
    /// ```
    #[inline]
    pub unsafe fn as_unsafe_cell(&self) -> &UnsafeCell<T> {
        &self.value
    }

    /// Returns a raw pointer to the underlying data in this cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let c = RefCell::new(5);
    ///
    /// let ptr = c.as_ptr();
    /// ```
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows `RefCell` mutably (at compile-time) so there is no
    /// need for dynamic checks.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// let mut c = RefCell::new(5);
    /// *c.get_mut() += 1;
    ///
    /// assert_eq!(c, RefCell::new(6));
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value.get()
        }
    }
}

unsafe impl<T: ?Sized> Send for RefCell<T> where T: Send {}

impl<T: Clone> Clone for RefCell<T> {
    #[inline]
    fn clone(&self) -> RefCell<T> {
        RefCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for RefCell<T> {
    #[inline]
    fn default() -> RefCell<T> {
        RefCell::new(Default::default())
    }
}

impl<T: ?Sized + PartialEq> PartialEq for RefCell<T> {
    #[inline]
    fn eq(&self, other: &RefCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: ?Sized + Eq> Eq for RefCell<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for RefCell<T> {
    #[inline]
    fn partial_cmp(&self, other: &RefCell<T>) -> Option<Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }

    #[inline]
    fn lt(&self, other: &RefCell<T>) -> bool {
        *self.borrow() < *other.borrow()
    }

    #[inline]
    fn le(&self, other: &RefCell<T>) -> bool {
        *self.borrow() <= *other.borrow()
    }

    #[inline]
    fn gt(&self, other: &RefCell<T>) -> bool {
        *self.borrow() > *other.borrow()
    }

    #[inline]
    fn ge(&self, other: &RefCell<T>) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: ?Sized + Ord> Ord for RefCell<T> {
    #[inline]
    fn cmp(&self, other: &RefCell<T>) -> Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        match borrow.get() {
            WRITING => None,
            b => {
                borrow.set(b + 1);
                Some(BorrowRef { borrow: borrow })
            },
        }
    }
}

impl<'b> Drop for BorrowRef<'b> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(borrow != WRITING && borrow != UNUSED);
        self.borrow.set(borrow - 1);
    }
}

impl<'b> Clone for BorrowRef<'b> {
    #[inline]
    fn clone(&self) -> BorrowRef<'b> {
        // Since this Ref exists, we know the borrow flag
        // is not set to WRITING.
        let borrow = self.borrow.get();
        debug_assert!(borrow != UNUSED);
        // Prevent the borrow counter from overflowing.
        assert!(borrow != WRITING);
        self.borrow.set(borrow + 1);
        BorrowRef { borrow: self.borrow }
    }
}

/// Wraps a borrowed reference to a value in a `RefCell` box.
/// A wrapper type for an immutably borrowed value from a `RefCell<T>`.
///
/// See the [module-level documentation](index.html) for more.
pub struct Ref<'b, T: ?Sized + 'b> {
    value: &'b T,
    borrow: BorrowRef<'b>,
}

impl<'b, T: ?Sized> Deref for Ref<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> Ref<'b, T> {
    /// Copies a `Ref`.
    ///
    /// The `RefCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::clone(...)`.  A `Clone` implementation or a method would interfere
    /// with the widespread use of `r.borrow().clone()` to clone the contents of
    /// a `RefCell`.
    #[inline]
    pub fn clone(orig: &Ref<'b, T>) -> Ref<'b, T> {
        Ref {
            value: orig.value,
            borrow: orig.borrow.clone(),
        }
    }

    /// Make a new `Ref` for a component of the borrowed data.
    ///
    /// The `RefCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `Ref::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `RefCell` used through `Deref`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::cell::{RefCell, Ref};
    ///
    /// let c = RefCell::new((5, 'b'));
    /// let b1: Ref<(u32, char)> = c.borrow();
    /// let b2: Ref<u32> = Ref::map(b1, |t| &t.0);
    /// assert_eq!(*b2, 5)
    /// ```
    #[inline]
    pub fn map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U>
        where F: FnOnce(&T) -> &U
    {
        Ref {
            value: f(orig.value),
            borrow: orig.borrow,
        }
    }
}

impl<'b, T: ?Sized> RefMut<'b, T> {
    /// Make a new `RefMut` for a component of the borrowed data, e.g. an enum
    /// variant.
    ///
    /// The `RefCell` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RefMut::map(...)`.  A method would interfere with methods of the same
    /// name on the contents of a `RefCell` used through `Deref`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::cell::{RefCell, RefMut};
    ///
    /// let c = RefCell::new((5, 'b'));
    /// {
    ///     let b1: RefMut<(u32, char)> = c.borrow_mut();
    ///     let mut b2: RefMut<u32> = RefMut::map(b1, |t| &mut t.0);
    ///     assert_eq!(*b2, 5);
    ///     *b2 = 42;
    /// }
    /// assert_eq!(*c.borrow(), (42, 'b'));
    /// ```
    #[inline]
    pub fn map<U: ?Sized, F>(orig: RefMut<'b, T>, f: F) -> RefMut<'b, U>
        where F: FnOnce(&mut T) -> &mut U
    {
        RefMut {
            value: f(orig.value),
            borrow: orig.borrow,
        }
    }
}

struct BorrowRefMut<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> Drop for BorrowRefMut<'b> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(borrow == WRITING);
        self.borrow.set(UNUSED);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        match borrow.get() {
            UNUSED => {
                borrow.set(WRITING);
                Some(BorrowRefMut { borrow: borrow })
            },
            _ => None,
        }
    }
}

/// A wrapper type for a mutably borrowed value from a `RefCell<T>`.
///
/// See the [module-level documentation](index.html) for more.
pub struct RefMut<'b, T: ?Sized + 'b> {
    value: &'b mut T,
    borrow: BorrowRefMut<'b>,
}

impl<'b, T: ?Sized> Deref for RefMut<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> DerefMut for RefMut<'b, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}


// Imported from src/libcore/fmt/mod.rs

impl<T: ?Sized + Debug> Debug for RefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.borrow_state() {
            BorrowState::Unused | BorrowState::Reading => {
                f.debug_struct("RefCell")
                    .field("value", &self.borrow())
                    .finish()
            }
            BorrowState::Writing => {
                f.debug_struct("RefCell")
                    .field("value", &"<borrowed>")
                    .finish()
            }
        }
    }
}

impl<'b, T: ?Sized + Debug> Debug for Ref<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<'b, T: ?Sized + Debug> Debug for RefMut<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&*(self.deref()), f)
    }
}
