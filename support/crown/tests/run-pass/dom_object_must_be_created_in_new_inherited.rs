/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

use std::rc::Rc;

#[expect(unused)]
trait DomObject {}

struct RegularStruct;

impl RegularStruct {
    fn new_inherited() -> Self {
        Self {

        }
    }

    // The types of arguments don't actually matter for this check
    fn new(cx: u32, global: u32) {
        reflect_dom_object(
            cx,
            Box::new(Self::new_inherited()),
            global,
        )
    }

    fn new_weak(cx: u32, global: u32) {
        reflect_weak_referenceable_dom_object_with_wrap(
            cx,
            Rc::new(Self::new_inherited()),
            global,
        )
    }
}

struct ChildStruct {
    _regular_struct: RegularStruct,
}

impl ChildStruct {
    fn new_inherited() -> Self {
        Self {
            _regular_struct: RegularStruct::new_inherited(),
        }
    }

    // The types of arguments don't actually matter for this check
    fn new(cx: u32, global: u32) {
        reflect_dom_object_child(
            cx,
            Box::new(Self::new_inherited()),
            global,
        )
    }
}

fn reflect_dom_object(_cx: u32, _struct: Box<RegularStruct>, _global: u32) {}
fn reflect_dom_object_child(_cx: u32, _struct: Box<ChildStruct>, _global: u32) {}
fn reflect_weak_referenceable_dom_object_with_wrap(_cx: u32, _struct: Rc<RegularStruct>, _global: u32) {}

fn main() {
    let _ = RegularStruct::new(0, 0);
    let _ = RegularStruct::new_weak(0, 0);
    let _ = ChildStruct::new(0, 0);
}
