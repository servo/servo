/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![deny(crown::jscontext_first_arg)]

struct JSContext {}

fn second_arg_mutable(_str: String, _cx: &mut JSContext) {}
//~^ 10:1: 10:60: the first argument should be JSContext [crown::jscontext_first_arg]
fn second_arg_ref(_str: String, _cx: &JSContext) {}
//~^ 12:1: 12:52: the first argument should be JSContext [crown::jscontext_first_arg]

struct SomeImpl {}

impl SomeImpl {
    fn with_self(&self, _str: String, _cx: &mut JSContext) {}
    //~^ 18:5: 18:62: the first argument should be JSContext [crown::jscontext_first_arg]
    fn with_mutable_self(&mut self, _str: String, _cx: &mut JSContext) {}
    //~^ 20:5: 20:74: the first argument should be JSContext [crown::jscontext_first_arg]
    fn with_owned_self(self, _str: String, _cx: &mut JSContext) {}
    //~^ 22:5: 22:67: the first argument should be JSContext [crown::jscontext_first_arg]
    fn without_self(_str: String, _cx: &mut JSContext) {}
    //~^ 24:5: 24:58: the first argument should be JSContext [crown::jscontext_first_arg]
    fn as_middle_with_self(&self, _str: String, _cx: &mut JSContext, _str2: String) {}
    //~^ 26:5: 26:87: the first argument should be JSContext [crown::jscontext_first_arg]
    fn as_middle_without_self(_str: String, _cx: &mut JSContext, _str2: String) {}
    //~^ 28:5: 28:83: the first argument should be JSContext [crown::jscontext_first_arg]
}

fn main() {
    second_arg_mutable(String::new(), &mut JSContext {});
    second_arg_ref(String::new(), &JSContext {});
    let mut some_impl = SomeImpl {};
    some_impl.with_self(String::new(), &mut JSContext {});
    some_impl.with_mutable_self(String::new(), &mut JSContext {});
    some_impl.as_middle_with_self(String::new(), &mut JSContext {}, String::new());
    some_impl.with_owned_self(String::new(), &mut JSContext {});
    SomeImpl::without_self(String::new(), &mut JSContext {});
    SomeImpl::as_middle_without_self(String::new(), &mut JSContext {}, String::new());
}
