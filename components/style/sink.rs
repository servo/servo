/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use smallvec::{Array, SmallVec};
use std::marker::PhantomData;

pub trait Push<T> {
    fn push(&mut self, value: T);
}

impl<T> Push<T> for Vec<T> {
    fn push(&mut self, value: T) {
        Vec::push(self, value);
    }
}

impl<A: Array> Push<A::Item> for SmallVec<A> {
    fn push(&mut self, value: A::Item) {
        SmallVec::push(self, value);
    }
}

pub struct ForgetfulSink<T>(bool, PhantomData<T>);

impl<T> ForgetfulSink<T> {
    pub fn new() -> Self {
        ForgetfulSink(true, PhantomData)
    }

    pub fn is_empty(&self) -> bool {
        self.0
    }
}

impl<T> Push<T> for ForgetfulSink<T> {
    fn push(&mut self, _value: T) {
        self.0 = false;
    }
}
