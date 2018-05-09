/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod unrooted_must_root {
    /**
    ```
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[derive(Clone, Debug)]  // derive should not be checked
    #[must_root] struct Foo(i32);
    #[must_root] struct Bar(Foo);

    impl Foo {
        fn new() -> Box<Foo> {
            let ret = Box::new(Foo(0));  // Box should be allowed as a binding type within constructors
            ret
        }
    }

    trait NewWith {
        fn new_with(x: i32) -> Self;
    }

    impl NewWith for Foo {
        fn new_with(x: i32) -> Foo {
            Foo(x)
        }
    }

    #[must_root] struct SomeContainer<#[must_root] T>(T);

    impl<#[must_root] T: NewWith> SomeContainer<T> {
        fn new(val: i32) -> SomeContainer<T> {
            SomeContainer(T::new_with(val))
        }
    }

    fn baz() {
        SomeContainer::<Foo>::new(3);
    }

    fn foo1(_: &Foo) {}
    fn foo2(_: &()) -> &Foo { unimplemented!() }
    fn new_foo() -> Foo { Foo::new_with(0) }

    fn foox<#[must_root] T>() { }
    fn barx<#[must_root] U>() { foox::<U>(); }

    fn main() {}
    ```
    */
    pub fn ok() {} // do we want to split it into smaller tests?

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    struct Bar(Foo);

    fn main() {}
    ```
    */
    pub fn struct_field() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    struct Bar<#[must_root] T>(T);

    fn test() {
        let _ = &Bar(Foo(3));
    }

    fn main() {}
    ```
    */
    pub fn generic_struct_field() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    fn foo<T>() { }
    fn bar<#[must_root] U>() { foo::<U>(); }

    fn main() {}
    ```
    */
    pub fn generic_function_calling() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    fn foo<#[must_root] T>() { }
    fn bar<#[must_root] U>() {
        (|| { foo::<U>(); }) ();
    }

    fn main() {}
    ```
    */
    pub fn ban_clousures() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);

    fn foo1(_: Foo) {}

    fn main() {}
    ```
    */
    pub fn parameter() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);

    fn foo2() -> Foo { unimplemented!() }

    fn main() {}
    ```
    */
    pub fn return_type() {}

    /**
    ```
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo([i32; 42]);

    fn foo(x: &Foo) -> i32 {
        let y = Foo(x.0).0;
        y[4]
    }

    fn main() {}
    ```
    */
    pub fn allow_subexpression() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);

    fn foo(x: &Foo) -> i32 {
        let y = Foo(x.0 + 3);
        y.0
    }

    fn main() {}
    ```
    */
    pub fn local_var() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);

    fn foo(x: &Foo) -> i32 {
        let y = Box::new(Foo(x.0 + 3));
        y.0
    }

    fn main() {}
    ```
    */
    pub fn ban_box() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    #[allow(unrooted_must_root)] struct Bar(Foo);

    fn foo(bar: Bar) -> bool {
        match bar {
            Bar(f @ Foo(_)) => f.0 == 4,
            _ => false,
        }
    }

    fn main() {}
    ```
    */
    pub fn pattern_binding() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    struct SomeContainer<T>(T);

    fn test() {
        SomeContainer(Foo(3));
    }

    fn main() {}
    ```
    */
    pub fn generic_container() {}

    /**
    ```compile_fail
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    struct SomeContainer<T>(T);

    impl<T> SomeContainer<T> { // impl should provide #[must_root] T
        fn foo(val: T) {
            SomeContainer(val);
        }
    }

    fn test() {
        SomeContainer::foo(Foo(3));
    }

    fn main() {}
    ```
    */
    pub fn generic_impl() {}

    /**
    ```
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);
    #[allow_unrooted_interior] struct SomeContainer<T>(T);

    impl<#[must_root] T> SomeContainer<T> {
        fn foo(val: &T) {
            SomeContainer(val);
        }
    }

    fn test() {
        SomeContainer(Foo(3));
        SomeContainer::foo(&Foo(3));
    }

    fn main() {}
    ```
    */
    pub fn allowing_unrooted_interior() {}

    /**
    ```
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[must_root] struct Foo(i32);

    #[must_root] //-- this is needed!
    trait Bar {
        fn extract(&self) -> i32;
    }

    impl Bar for Foo {
        fn extract(&self) -> i32 { self.0 }
    }

    fn test() {
        Foo(3).extract();
    }

    fn main() {}
    ```
    */
    pub fn allow_impl_for_must_root() {}

    /* *
    ```
    #![feature(plugin)]
    #![plugin(script_plugins)]

    #[derive(Default)]
    #[must_root] struct Foo(i32);
    #[derive(Default)]
    #[must_root] struct Bar(i32);

    fn create_foo<#[must_root] T: Default>() -> T {
        Default::default()
    }

    fn test() -> i32 {
        //let factory = &create_foo::<Foo>;
        //factory().0
        let elem = &create_foo::<Foo>();
        elem.0
    }

    fn test2() -> i32 {
        //let factory = &create_foo::<Foo>;
        //factory().0
        let elem = &create_foo::<Bar>();
        elem.0
    }

    fn main() {}
    ```
    */
    //pub fn derive_default() {}
    // ^ do we want to allow derivation?
}
