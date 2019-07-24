/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use style::selector_parser::RestyleDamage;

#[allow(unsafe_code)]
pub unsafe trait HasBaseFlow {}

pub trait GetBaseFlow {
    fn base(&self) -> &BaseFlow;
    fn mut_base(&mut self) -> &mut BaseFlow;
}

impl<T: HasBaseFlow + ?Sized> GetBaseFlow for T {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn base(&self) -> &BaseFlow {
        let ptr: *const Self = self;
        let ptr = ptr as *const BaseFlow;
        unsafe { &*ptr }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn mut_base(&mut self) -> &mut BaseFlow {
        let ptr: *mut Self = self;
        let ptr = ptr as *mut BaseFlow;
        unsafe { &mut *ptr }
    }
}

pub trait Flow: HasBaseFlow + fmt::Debug + Sync + Send + 'static {}

pub struct BaseFlow {
    pub restyle_damage: RestyleDamage,
}
