/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Weak};

use atomic_refcell::AtomicRefCell;
use malloc_size_of_derive::MallocSizeOf;

#[derive(MallocSizeOf)]
pub struct ArcRefCell<T> {
    #[conditional_malloc_size_of]
    value: Arc<AtomicRefCell<T>>,
}

impl<T> ArcRefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(AtomicRefCell::new(value)),
        }
    }

    pub(crate) fn downgrade(&self) -> WeakRefCell<T> {
        WeakRefCell {
            value: Arc::downgrade(&self.value),
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Clone for ArcRefCell<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T> Default for ArcRefCell<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: Arc::new(AtomicRefCell::new(Default::default())),
        }
    }
}

impl<T> Deref for ArcRefCell<T> {
    type Target = AtomicRefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> fmt::Debug for ArcRefCell<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(formatter)
    }
}

#[derive(Debug, MallocSizeOf)]
pub(crate) struct WeakRefCell<T> {
    value: Weak<AtomicRefCell<T>>,
}

impl<T> Clone for WeakRefCell<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T> WeakRefCell<T> {
    pub(crate) fn upgrade(&self) -> Option<ArcRefCell<T>> {
        self.value.upgrade().map(|value| ArcRefCell { value })
    }
}
