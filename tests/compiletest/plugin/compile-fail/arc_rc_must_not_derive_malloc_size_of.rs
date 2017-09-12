/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate malloc_size_of;
extern crate servo_arc;

use malloc_size_of::{MallocShallowSizeOf, MallocSizeOf};

fn sizeable<T: MallocSizeOf>() {
}

fn shallow_sizeable<T: MallocShallowSizeOf>() {
}

fn main() {
    sizeable::<::servo_arc::Arc<i32>>();
    //~^ ERROR the trait bound `servo_arc::Arc<i32>: malloc_size_of::MallocSizeOf` is not satisfied

    sizeable::<::std::sync::Arc<i32>>();
    //~^ ERROR the trait bound `std::sync::Arc<i32>: malloc_size_of::MallocSizeOf` is not satisfied

    sizeable::<::std::rc::Rc<i32>>();
    //~^ ERROR the trait bound `std::rc::Rc<i32>: malloc_size_of::MallocSizeOf` is not satisfied

    shallow_sizeable::<::servo_arc::Arc<i32>>();
    //~^ ERROR the trait bound `servo_arc::Arc<i32>: malloc_size_of::MallocShallowSizeOf` is not satisfied

    shallow_sizeable::<::std::sync::Arc<i32>>();
    //~^ ERROR the trait bound `std::sync::Arc<i32>: malloc_size_of::MallocShallowSizeOf` is not satisfied

    shallow_sizeable::<::std::rc::Rc<i32>>();
    //~^ ERROR the trait bound `std::rc::Rc<i32>: malloc_size_of::MallocShallowSizeOf` is not satisfied
}
