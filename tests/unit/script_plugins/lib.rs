/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[rustfmt::skip]
pub mod unrooted_must_root {
    /**
    ```
    #![feature(plugin, register_tool)]
    #![plugin(script_plugins)]
    #![register_tool(unrooted_must_root_lint)]

    #[unrooted_must_root_lint::must_root] struct Foo(i32);
    #[unrooted_must_root_lint::must_root] struct Bar(Foo);

    fn foo1(_: &Foo) {}
    fn foo2(_: &()) -> &Foo { unimplemented!() }

    fn main() {}
    ```
    */
    pub fn ok() {}

    /**
    ```compile_fail
    #![feature(plugin, register_tool)]
    #![plugin(script_plugins)]
    #![register_tool(unrooted_must_root_lint)]

    #[unrooted_must_root_lint::must_root] struct Foo(i32);
    struct Bar(Foo);

    fn main() {}
    ```
    */
    pub fn struct_field() {}

    /**
    ```compile_fail
    #![feature(plugin, register_tool)]
    #![plugin(script_plugins)]
    #![register_tool(unrooted_must_root_lint)]

    #[unrooted_must_root_lint::must_root] struct Foo(i32);

    fn foo1(_: Foo) {}

    fn main() {}
    ```
    */
    pub fn parameter() {}

    /**
    ```compile_fail
    #![feature(plugin, register_tool)]
    #![plugin(script_plugins)]
    #![register_tool(unrooted_must_root_lint)]

    #[unrooted_must_root_lint::must_root] struct Foo(i32);

    fn foo2() -> Foo { unimplemented!() }

    fn main() {}
    ```
    */
    pub fn return_type() {}

}
