/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! define_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        __define_css_keyword_enum__add_serde!($name [ $( $css => $variant ),+ ]);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        __define_css_keyword_enum__add_serde!($name [ $( $css => $variant ),+ ]);
    };
}

#[cfg(feature = "serde-serialization")]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_serde {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__add_heapsize! {
            $name [ Deserialize, Serialize ] [ $( $css => $variant ),+ ]
        }
    };
}

#[cfg(not(feature = "serde-serialization"))]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_serde {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__add_heapsize! {
            $name [] [ $( $css => $variant ),+ ]
        }
    };
}

#[cfg(feature = "heap_size")]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_heapsize {
    ($name: ident [ $( $derived_trait: ident),* ]  [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__actual! {
            $name [  $( $derived_trait, )* HeapSizeOf ] [ $( $css => $variant ),+ ]
        }
    };
}

#[cfg(not(feature = "heap_size"))]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_heapsize {
    ($name: ident [ $( $derived_trait: ident),* ]  [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__actual! {
            $name [ $( $derived_trait ),* ] [ $( $css => $variant ),+ ]
        }
    };
}

#[macro_export]
macro_rules! __define_css_keyword_enum__actual {
    ($name: ident [ $( $derived_trait: ident),* ] [ $( $css: expr => $variant: ident ),+ ]) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug $(, $derived_trait )* )]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                                           $( $css => Ok($name::$variant), )+
                                           _ => Err(())
                }
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write {
                    match *self {
                        $( $name::$variant => dest.write_str($css) ),+
                    }
                }
        }
    }
}


pub mod specified {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AllowedNumericType {
        All,
        NonNegative
    }

    impl AllowedNumericType {
        #[inline]
        pub fn is_ok(&self, value: f32) -> bool {
            match *self {
                AllowedNumericType::All => true,
                AllowedNumericType::NonNegative => value >= 0.,
            }
        }
    }
}
