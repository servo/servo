/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
```
extern crate malloc_size_of;
extern crate servo_arc;

fn sizeable<T: malloc_size_of::MallocSizeOf>() {}
fn shallow_sizeable<T: malloc_size_of::MallocShallowSizeOf>() {}
fn cloneable<T: Clone>() {}

fn main() {
    cloneable::<servo_arc::Arc<i32>>();
    cloneable::<std::sync::Arc<i32>>();
    cloneable::<std::rc::Rc<i32>>();
}
```
*/
pub fn imports_ok() {}

pub mod does_not_impl_malloc_size_of {
    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;
    extern crate servo_arc;

    fn sizeable<T: malloc_size_of::MallocSizeOf>() {}

    fn main() {
        sizeable::<servo_arc::Arc<i32>>();
    }
    ```
    */
    pub fn servo_arc() {}

    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;

    fn sizeable<T: malloc_size_of::MallocSizeOf>() {}

    fn main() {
        sizeable::<std::sync::Arc<i32>>();
    }
    ```
    */
    pub fn std_arc() {}

    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;

    fn sizeable<T: malloc_size_of::MallocSizeOf>() {}

    fn main() {
        sizeable::<std::rc::Rc<i32>>();
    }
    ```
    */
    pub fn rc() {}
}

pub mod does_not_impl_malloc_shallow_size_of {
    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;
    extern crate servo_arc;

    fn shallow_sizeable<T: malloc_size_of::MallocShallowSizeOf>() {}

    fn main() {
        shallow_sizeable::<servo_arc::Arc<i32>>();
    }
    ```
    */
    pub fn servo_arc() {}

    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;

    fn shallow_sizeable<T: malloc_size_of::MallocShallowSizeOf>() {}

    fn main() {
        shallow_sizeable::<std::sync::Arc<i32>>();
    }
    ```
    */
    pub fn std_arc() {}

    /**
    ```compile_fail,E0277
    extern crate malloc_size_of;

    fn shallow_sizeable<T: malloc_size_of::MallocShallowSizeOf>() {}

    fn main() {
        shallow_sizeable::<std::rc::Rc<i32>>();
    }
    ```
    */
    pub fn rc() {}
}
