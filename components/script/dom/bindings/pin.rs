/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Pins, also known as immovable roots.

use dom::bindings::reflector::DomObject;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::trace::JSTraceable;
use js::jsapi::JSTracer;
use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, Drop};

#[allow(unrooted_must_root)]
#[allow_unrooted_interior]
pub struct Pin<'this, T>
where
    T: JSTraceable + 'static,
{
    marker: PhantomData<&'this ()>,
    cell: Option<PinCell<T>>,
}

impl<'this, T> Pin<'this, T>
where
    T: JSTraceable,
{
    #[inline]
    pub unsafe fn new() -> Self {
        Self { marker: PhantomData, cell: None }
    }

    pub fn pin<U>(&'this mut self, traced: U) -> Pinned<'this, T>
    where
        T: UntracedFrom<U>,
    {
        unsafe {
            self.cell = Some(PinCell::new(T::untraced_from(traced)));
            self.cell.as_mut().unwrap().pin()
        }
    }

    pub fn pin_default(&'this mut self) -> Pinned<'this, T>
    where
        T: UntracedDefault,
    {
        unsafe {
            self.cell = Some(PinCell::new(T::untraced_default()));
            self.cell.as_mut().unwrap().pin()
        }
    }
}

#[allow_unrooted_interior]
pub struct Pinned<'pin, T>
where
    T: 'static,
{
    value: &'pin T,
}

impl<'pin, T> Deref for Pinned<'pin, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'pin, T> Pinned<'pin, Mut<T>> {
    /// FIXME(nox): Not exactly entirely sound given that `&mut T` most probably
    /// wasn't acquired in a sound way, but when life gives you lemons…
    pub fn swap(&mut self, other: &mut T) {
        unsafe { mem::swap(self.as_mut(), other) }
    }
}

impl<'pin, T> Pinned<'pin, Mut<T>>
where
    T: UntracedSink,
{
    #[inline]
    pub fn clear(&mut self) {
        unsafe { self.as_mut().clear() }
    }

    #[inline]
    pub fn push<U>(&mut self, value: U)
    where
        <T as UntracedSink>::Item: UntracedFrom<U>,
    {
        unsafe {
            let untraced = UntracedFrom::untraced_from(value);
            self.as_mut().push(untraced);
        }
    }
}

impl<'pin, T, U> Extend<U> for Pinned<'pin, Mut<T>>
where
    T: UntracedSink,
    <T as UntracedSink>::Item: UntracedFrom<U>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = U>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

#[derive(JSTraceable)]
pub struct Mut<T> {
    value: UnsafeCell<T>,
}

impl<T> Mut<T> {
    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.value.get()
    }
}

impl<T> Deref for Mut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value.get() }
    }
}

pub trait UntracedDefault: 'static {
    unsafe fn untraced_default() -> Self;
}

impl<T> UntracedDefault for Mut<T>
where
    T: UntracedDefault
{
    #[inline]
    unsafe fn untraced_default() -> Self {
        Mut { value: UnsafeCell::new(T::untraced_default()) }
    }
}

macro_rules! impl_untraceddefault_as_default {
    (for<$($param:ident),*> $ty:ty) => {
        impl<$($param),*> UntracedDefault for $ty
        where
            $($param: 'static),*
        {
            #[inline]
            unsafe fn untraced_default() -> Self {
                Default::default()
            }
        }
    };
}

impl_untraceddefault_as_default!(for<T> Vec<T>);

pub trait UntracedFrom<T>: 'static {
    unsafe fn untraced_from(traced: T) -> Self;
}

impl<'a, T> UntracedFrom<&'a mut T> for T
where
    T: UntracedDefault + 'static,
{
    #[inline]
    unsafe fn untraced_from(traced: &'a mut T) -> Self {
        mem::replace(traced, T::untraced_default())
    }
}

impl<'a, T> UntracedFrom<&'a T> for Dom<T>
where
    T: DomObject + 'static,
{
    #[allow(unrooted_must_root)]
    #[inline]
    unsafe fn untraced_from(traced: &'a T) -> Self {
        Dom::from_ref(traced)
    }
}

impl<'a, T> UntracedFrom<&'a Dom<T>> for Dom<T>
where
    T: DomObject + 'static,
{
    #[allow(unrooted_must_root)]
    #[inline]
    unsafe fn untraced_from(traced: &'a Dom<T>) -> Self {
        Dom::from_ref(&**traced)
    }
}

impl<T> UntracedFrom<DomRoot<T>> for Dom<T>
where
    T: DomObject + 'static,
{
    #[allow(unrooted_must_root)]
    #[inline]
    unsafe fn untraced_from(traced: DomRoot<T>) -> Self {
        Dom::from_ref(&*traced)
    }
}

pub unsafe trait UntracedSink {
    type Item;

    fn clear(&mut self);
    fn push(&mut self, value: Self::Item);
}

unsafe impl<T> UntracedSink for Vec<T> {
    type Item = T;

    #[inline]
    fn clear(&mut self) {
        self.clear();
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        self.push(value);
    }
}

pub unsafe fn initialize() {
    PINNED_TRACEABLES.with(|cell| {
        let mut cell = cell.borrow_mut();
        assert!(cell.is_none(), "pin list has already been initialized");
        *cell = Some(None);
    });
}

pub unsafe fn trace(tracer: *mut JSTracer) {
    trace!("tracing stack-rooted pins");
    PINNED_TRACEABLES.with(|ref cell| {
        let cell = cell.borrow();
        let mut head = cell.unwrap();
        while let Some(current) = head {
            (*current).value.trace(tracer);
            head = (*current).prev;
        }
    });
}

thread_local! {
    static PINNED_TRACEABLES: RefCell<Option<Option<*const PinCell<JSTraceable>>>> =
        Default::default();
}

struct PinCell<T>
where
    T: JSTraceable + ?Sized + 'static,
{
    prev: Option<*const PinCell<JSTraceable>>,
    value: T,
}

impl<T> PinCell<T>
where
    T: JSTraceable + 'static,
{
    unsafe fn new(untraced: T) -> Self {
        Self { prev: None, value: untraced }
    }

    unsafe fn pin<'pin>(&'pin mut self) -> Pinned<'pin, T> {
        let this = self as &PinCell<JSTraceable> as *const _;
        PINNED_TRACEABLES.with(|cell| {
            self.prev = mem::replace(
                cell.borrow_mut().as_mut().unwrap(),
                Some(this),
            );
        });
        Pinned { value: &self.value }
    }
}

impl<T> Drop for PinCell<T>
where
    T: JSTraceable + ?Sized + 'static,
{
    fn drop(&mut self) {
        PINNED_TRACEABLES.with(|cell| {
            *cell.borrow_mut().as_mut().unwrap() = self.prev;
        });
    }
}
