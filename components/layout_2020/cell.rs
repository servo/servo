/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::Deref;

use atomic_refcell::AtomicRefCell;
use serde::{Serialize, Serializer};
use servo_arc::Arc;

pub(crate) struct ArcRefCell<T> {
    value: Arc<AtomicRefCell<T>>,
}

impl<T> ArcRefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(AtomicRefCell::new(value)),
        }
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

impl<T> Serialize for ArcRefCell<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.borrow().serialize(serializer)
    }
}
