/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Various macro helpers.

macro_rules! exclusive_value {
    (($value:ident, $set:expr) => $ident:path) => {
        if $value.intersects($set) {
            return Err(());
        } else {
            $ident
        }
    };
}

#[cfg(feature = "gecko")]
macro_rules! impl_gecko_keyword_conversions {
    ($name:ident, $utype:ty) => {
        impl From<$utype> for $name {
            fn from(bits: $utype) -> $name {
                $name::from_gecko_keyword(bits)
            }
        }

        impl From<$name> for $utype {
            fn from(v: $name) -> $utype {
                v.to_gecko_keyword()
            }
        }
    };
}

macro_rules! trivial_to_computed_value {
    ($name:ty) => {
        impl $crate::values::computed::ToComputedValue for $name {
            type ComputedValue = $name;

            fn to_computed_value(&self, _: &$crate::values::computed::Context) -> Self {
                self.clone()
            }

            fn from_computed_value(other: &Self) -> Self {
                other.clone()
            }
        }
    };
}

/// A macro to parse an identifier, or return an `UnexpectedIdent` error
/// otherwise.
///
/// FIXME(emilio): The fact that `UnexpectedIdent` is a `SelectorParseError`
/// doesn't make a lot of sense to me.
macro_rules! try_match_ident_ignore_ascii_case {
    ($input:expr, $( $match_body:tt )*) => {{
        let location = $input.current_source_location();
        let ident = $input.expect_ident_cloned()?;
        match_ignore_ascii_case! { &ident,
            $( $match_body )*
            _ => return Err(location.new_custom_error(
                ::selectors::parser::SelectorParseErrorKind::UnexpectedIdent(ident.clone())
            ))
        }
    }}
}

#[cfg(feature = "servo")]
macro_rules! local_name {
    ($s:tt) => {
        $crate::values::GenericAtomIdent(html5ever::local_name!($s))
    };
}

#[cfg(feature = "servo")]
macro_rules! ns {
    () => {
        $crate::values::GenericAtomIdent(html5ever::ns!())
    };
    ($s:tt) => {
        $crate::values::GenericAtomIdent(html5ever::ns!($s))
    };
}

#[cfg(feature = "gecko")]
macro_rules! local_name {
    ($s:tt) => {
        $crate::values::AtomIdent(atom!($s))
    };
}

/// Asserts the size of a type at compile time.
macro_rules! size_of_test {
    ($t: ty, $expected_size: expr) => {
        #[cfg(target_pointer_width = "64")]
        const_assert_eq!(std::mem::size_of::<$t>(), $expected_size);
    };
}
