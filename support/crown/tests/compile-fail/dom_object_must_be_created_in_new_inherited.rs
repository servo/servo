/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#[expect(unused)]
trait DomObject {}

struct DomStruct;

impl crate::DomObject for DomStruct {}

impl DomStruct {
    fn some_other_method() -> Self {
        Self {
        //~^ ERROR: 15:9: 17:10: dom objects must only be constructed inside a `new_inherited` method on that dom object [crown::domstruct_in_new_inherited]
        }
    }

    fn new_inherited() -> Self {
        Self { }
    }

    // The types of arguments don't actually matter for this check
    fn new(cx: u32, global: u32) -> DomStruct {
        reflect_dom_object(
            cx,
            Box::new(Self::some_other_method()),
            global,
        )
    }
    fn new2(_cx: u32, _global: u32) -> DomStruct {
        let foo = Self::new_inherited();
        //~^ ERROR: 33:19: 33:40: new_inherited must be passed to `Box::new` and inside a `reflect_dom_object` [crown::domstruct_in_new_inherited]
        foo
    }
    fn new3(cx: u32, global: u32) -> DomStruct {
        let foo = Box::new(Self::new_inherited());
        //~^ ERROR: 38:28: 38:49: new_inherited must be passed to `Box::new` and inside a `reflect_dom_object` [crown::domstruct_in_new_inherited]
        reflect_dom_object(
            cx,
            foo,
            global,
        )
    }
}

fn outside_method() -> DomStruct {
    DomStruct {
    //~^ ERROR: 49:5: 51:6: dom objects must only be constructed inside a `new_inherited` method on that dom object [crown::domstruct_in_new_inherited]
    }
}

fn reflect_dom_object(_cx: u32, _struct: Box<DomStruct>, _global: u32) -> DomStruct {
    todo!()
}

struct DifferentStruct {}

impl DifferentStruct {
    fn new_inherited() -> DomStruct {
        DomStruct {
        //~^ ERROR: 62:9: 64:10: dom objects must only be constructed inside a `new_inherited` method on that dom object [crown::domstruct_in_new_inherited]
        }
    }
    fn new(cx: u32, global: u32) -> DomStruct {
        reflect_dom_object(
            cx,
            Box::new(Self::new_inherited()),
            global,
        )
    }
}

fn main() {
    let _ = outside_method();
    let _ = DomStruct::new(0, 0);
    let _ = DomStruct::new2(0, 0);
    let _ = DomStruct::new3(0, 0);
    let _ = DifferentStruct::new(0, 0);
    let _ = DomStruct {
    //~^ ERROR: 81:13: 83:6: dom objects must only be constructed inside a `new_inherited` method on that dom object [crown::domstruct_in_new_inherited]
    };
}
