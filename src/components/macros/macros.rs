/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo#macros:0.1"]
#![crate_type = "lib"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(macro_rules)]

#[cfg(test)]
extern crate sync;

#[macro_export]
macro_rules! lazy_init(
    ($(static ref $N:ident : $T:ty = $e:expr;)*) => (
        $(
            #[allow(non_camel_case_types)]
            struct $N {__unit__: ()}
            static $N: $N = $N {__unit__: ()};
            impl Deref<$T> for $N {
                fn deref<'a>(&'a self) -> &'a $T {
                    unsafe {
                        static mut s: *$T = 0 as *$T;
                        static mut ONCE: ::sync::one::Once = ::sync::one::ONCE_INIT;
                        ONCE.doit(|| {
                            s = ::std::mem::transmute::<Box<$T>, *$T>(box () ($e));
                        });
                        &*s
                    }
                }
            }

        )*
    )
)

#[cfg(test)]
mod tests {
    use std::collections::hashmap::HashMap;
    lazy_init! {
        static ref NUMBER: uint = times_two(3);
        static ref VEC: [Box<uint>, ..3] = [box 1, box 2, box 3];
        static ref OWNED_STRING: String = "hello".to_string();
        static ref HASHMAP: HashMap<uint, &'static str> = {
            let mut m = HashMap::new();
            m.insert(0u, "abc");
            m.insert(1, "def");
            m.insert(2, "ghi");
            m
        };
    }

    fn times_two(n: uint) -> uint {
        n * 2
    }

    #[test]
    fn test_basic() {
        assert_eq!(*OWNED_STRING, "hello".to_string());
        assert_eq!(*NUMBER, 6);
        assert!(HASHMAP.find(&1).is_some());
        assert!(HASHMAP.find(&3).is_none());
        assert_eq!(VEC.as_slice(), &[box 1, box 2, box 3]);
    }

    #[test]
    fn test_repeat() {
        assert_eq!(*NUMBER, 6);
        assert_eq!(*NUMBER, 6);
        assert_eq!(*NUMBER, 6);
    }
}
