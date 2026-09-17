/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![deny(crown::jscontext_first_arg)]

struct JSContext {}

fn second_arg_mutable(_cx: &mut JSContext, _str: String) {}
fn second_arg_ref(_cx: &JSContext, _str: String) {}

struct SomeImpl {}

impl SomeImpl {
    fn with_self(&self, _cx: &mut JSContext, _str: String) {}
    fn with_mutable_self(&mut self, _cx: &mut JSContext, _str: String) {}
    fn with_owned_self(self, _cx: &mut JSContext, _str: String) {}
    fn without_self(_cx: &mut JSContext, _str: String) {}
}

fn main() {
    second_arg_mutable(&mut JSContext {}, String::new());
    second_arg_ref(&JSContext {}, String::new());
    let mut some_impl = SomeImpl {};
    some_impl.with_self(&mut JSContext {}, String::new());
    some_impl.with_mutable_self(&mut JSContext {}, String::new());
    some_impl.with_owned_self(&mut JSContext {}, String::new());
    SomeImpl::without_self(&mut JSContext {}, String::new());
}
