/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "macros"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(macro_rules, plugin_registrar, quote, phase)]

//! Exports macros for use in other Servo crates.

#[cfg(test)]
extern crate sync;

#[cfg(test)]
#[phase(plugin)]
extern crate macros;

extern crate rustc;
extern crate syntax;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;
use rustc::plugin::Registry;
use std::gc::{Gc, GC};


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("bit_struct"),
        base::IdentTT(box base::BasicIdentMacroExpander {
            expander: expand_bit_struct,
            span: None,
        }, None));
}

fn expand_bit_struct(cx: &mut ExtCtxt, sp: Span, name: ast::Ident, tts: Vec<ast::TokenTree>)
                     -> Box<base::MacResult> {
    let mut fields = Vec::new();
    for (i, e) in tts.iter().enumerate() {
        if i & 1 == 1 {
            match *e {
                ast::TTTok(_, token::COMMA) => (),
                _ => {
                    cx.span_err(sp, "bit_struct! expecting comma.");
                    return base::DummyResult::any(sp);
                }
            }
        } else {
            match *e {
                ast::TTTok(_, token::IDENT(ident, _)) => {
                    fields.push(ident)
                }
                _ => {
                    cx.span_err(sp, "bit_struct! requires ident args.");
                    return base::DummyResult::any(sp);
                }
            }
        }
    }
    let bits_per_word =
        if cfg!(target_word_size = "64") { 64 }
        else if cfg!(target_word_size = "32") { 32 }
        else { fail!("Unexpected target word size") };
    let nb_words = (fields.len() - 1 + bits_per_word) / bits_per_word;

    let struct_def = quote_item!(&*cx,
        pub struct $name {
            storage: [uint, ..$nb_words]
        }
    ).unwrap();
    let impl_def = quote_item!(&*cx,
        impl $name {
            #[inline]
            pub fn new() -> $name {
                $name { storage: [0, ..$nb_words] }
            }
        }
    ).unwrap();

    // Unwrap from Gc<T>, which does not implement DerefMut
    let mut impl_def = (*impl_def).clone();
    match impl_def.node {
        ast::ItemImpl(_, _, _, ref mut methods) => {
            for (i, field) in fields.iter().enumerate() {
                let setter_name = "set_".to_string() + field.as_str();
                let setter = token::str_to_ident(setter_name.as_slice());
                let word = i / bits_per_word;
                let bit = i % bits_per_word;
                let additional_impl = quote_item!(&*cx,
                    impl $name {
                        #[allow(non_snake_case_functions)]
                        #[inline]
                        pub fn $field(&self) -> bool {
                            (self.storage[$word] & (1 << $bit)) != 0
                        }
                        #[allow(non_snake_case_functions)]
                        #[inline]
                        pub fn $setter(&mut self, new_value: bool) {
                            if new_value {
                                self.storage[$word] |= 1 << $bit
                            } else {
                                self.storage[$word] &= !(1 << $bit)
                            }
                        }
                    }
                ).unwrap();
                match additional_impl.node {
                    ast::ItemImpl(_, _, _, ref additional_methods) => {
                        methods.push_all(additional_methods.as_slice());
                    }
                    _ => fail!()
                }
            }
        }
        _ => fail!()
    }
    // Re-wrap.
    let impl_def = box(GC) impl_def;

    // FIXME(SimonSapin) replace this with something from libsyntax
    // if/when https://github.com/rust-lang/rust/issues/16723 is fixed
    struct MacItems {
        items: Vec<Gc<ast::Item>>
    }

    impl MacResult for MacItems {
        fn make_items(&self) -> Option<SmallVector<Gc<ast::Item>>> {
            Some(SmallVector::many(self.items.clone()))
        }
    }

    box MacItems { items: vec![struct_def, impl_def] } as Box<MacResult>
}


#[macro_export]
macro_rules! bitfield(
    ($bitfieldname:ident, $getter:ident, $setter:ident, $value:expr) => (
        impl $bitfieldname {
            #[inline]
            pub fn $getter(self) -> bool {
                let $bitfieldname(this) = self;
                (this & $value) != 0
            }

            #[inline]
            pub fn $setter(&mut self, value: bool) {
                let $bitfieldname(this) = *self;
                *self = $bitfieldname((this & !$value) | (if value { $value } else { 0 }))
            }
        }
    )
)


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
                        static mut s: *const $T = 0 as *const $T;
                        static mut ONCE: ::sync::one::Once = ::sync::one::ONCE_INIT;
                        ONCE.doit(|| {
                            s = ::std::mem::transmute::<Box<$T>, *const $T>(box () ($e));
                        });
                        &*s
                    }
                }
            }

        )*
    )
)


#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::collections::hashmap::HashMap;
    use std::mem::size_of;

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

    bit_struct! TestStruct64 {
        f01, f02, f03, f04, f05, f06, f07, f08, f09, f10,
        f11, f12, f13, f14, f15, f16, f17, f18, f19, f20,
        f21, f22, f23, f24, f25, f26, f27, f28, f29, f30,
        f31, f32, f33, f34, f35, f36, f37, f38, f39, f40,
        f41, f42, f43, f44, f45, f46, f47, f48, f49, f50,
        f51, f52, f53, f54, f55, f56, f57, f58, f59, f60,
        f61, f62, f63, f64,
    }

    bit_struct! TestStruct65 {
        f01, f02, f03, f04, f05, f06, f07, f08, f09, f10,
        f11, f12, f13, f14, f15, f16, f17, f18, f19, f20,
        f21, f22, f23, f24, f25, f26, f27, f28, f29, f30,
        f31, f32, f33, f34, f35, f36, f37, f38, f39, f40,
        f41, f42, f43, f44, f45, f46, f47, f48, f49, f50,
        f51, f52, f53, f54, f55, f56, f57, f58, f59, f60,
        f61, f62, f63, f64, f65,
    }

    #[test]
    fn test_bit_struct() {
        if cfg!(target_word_size = "64") {
            // One and two 8-byte words
            assert_eq!(size_of::<TestStruct64>(), 8)
            assert_eq!(size_of::<TestStruct65>(), 16)
        } else {
            // Two and three 4-byte words
            assert_eq!(size_of::<TestStruct64>(), 8)
            assert_eq!(size_of::<TestStruct65>(), 12)
        }

        let mut foo = TestStruct65::new();
        assert_eq!(foo.f01(), false);
        assert_eq!(foo.f32(), false);
        assert_eq!(foo.f33(), false);
        assert_eq!(foo.f64(), false);
        assert_eq!(foo.f65(), false);

        foo.set_f33(true);
        assert_eq!(foo.f01(), false);
        assert_eq!(foo.f32(), false);
        assert_eq!(foo.f33(), true);
        assert_eq!(foo.f64(), false);
        assert_eq!(foo.f65(), false);

        foo.set_f01(false);
        assert_eq!(foo.f01(), false);
        assert_eq!(foo.f32(), false);
        assert_eq!(foo.f33(), true);
        assert_eq!(foo.f64(), false);
        assert_eq!(foo.f65(), false);

        foo.set_f65(true);
        assert_eq!(foo.f01(), false);
        assert_eq!(foo.f32(), false);
        assert_eq!(foo.f33(), true);
        assert_eq!(foo.f64(), false);
        assert_eq!(foo.f65(), true);

        foo.set_f33(false);
        assert_eq!(foo.f01(), false);
        assert_eq!(foo.f32(), false);
        assert_eq!(foo.f33(), false);
        assert_eq!(foo.f64(), false);
        assert_eq!(foo.f65(), true);
    }
}
