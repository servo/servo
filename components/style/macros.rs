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

macro_rules! define_keyword_type {
    ($name:ident, $css:expr) => {
        #[allow(missing_docs)]
        #[derive(
            Animate,
            Clone,
            ComputeSquaredDistance,
            Copy,
            MallocSizeOf,
            PartialEq,
            SpecifiedValueInfo,
            ToAnimatedValue,
            ToAnimatedZero,
            ToComputedValue,
            ToCss,
            ToResolvedValue,
            ToShmem,
        )]
        pub struct $name;

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($css)
            }
        }

        impl $crate::parser::Parse for $name {
            fn parse<'i, 't>(
                _context: &$crate::parser::ParserContext,
                input: &mut ::cssparser::Parser<'i, 't>,
            ) -> Result<$name, ::style_traits::ParseError<'i>> {
                input
                    .expect_ident_matching($css)
                    .map(|_| $name)
                    .map_err(|e| e.into())
            }
        }
    };
}

/// Place a Gecko profiler label on the stack.
///
/// The `label_type` argument must be the name of a variant of `ProfilerLabel`.
#[cfg(feature = "gecko_profiler")]
#[macro_export]
macro_rules! profiler_label {
    ($label_type:ident) => {
        let mut _profiler_label: $crate::gecko_bindings::structs::AutoProfilerLabel = unsafe {
            ::std::mem::uninitialized()
        };
        let _profiler_label = if $crate::gecko::profiler::profiler_is_active() {
            unsafe {
                Some($crate::gecko::profiler::AutoProfilerLabel::new(
                    &mut _profiler_label,
                    $crate::gecko::profiler::ProfilerLabel::$label_type,
                ))
            }
        } else {
            None
        };
    }
}

/// No-op when the Gecko profiler is not available.
#[cfg(not(feature = "gecko_profiler"))]
#[macro_export]
macro_rules! profiler_label {
    ($label_type:ident) => {}
}
