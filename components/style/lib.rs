/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Calculate [specified][specified] and [computed values][computed] from a
//! tree of DOM nodes and a set of stylesheets.
//!
//! [computed]: https://drafts.csswg.org/css-cascade/#computed
//! [specified]: https://drafts.csswg.org/css-cascade/#specified
//!
//! In particular, this crate contains the definitions of supported properties,
//! the code to parse them into specified values and calculate the computed
//! values based on the specified values, as well as the code to serialize both
//! specified and computed values.
//!
//! The main entry point is [`recalc_style_at`][recalc_style_at].
//!
//! [recalc_style_at]: traversal/fn.recalc_style_at.html
//!
//! Major dependencies are the [cssparser][cssparser] and [selectors][selectors]
//! crates.
//!
//! [cssparser]: ../cssparser/index.html
//! [selectors]: ../selectors/index.html

#![deny(missing_docs)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cssparser;
#[macro_use]
extern crate debug_unreachable;
#[macro_use]
extern crate derive_more;
#[cfg(feature = "gecko")]
#[macro_use]
pub mod gecko_string_cache;
#[cfg(feature = "servo")]
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
#[cfg(feature = "gecko")]
pub use nsstring;
#[cfg(feature = "gecko")]
extern crate num_cpus;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate serde;
pub use servo_arc;
#[cfg(feature = "servo")]
#[macro_use]
extern crate servo_atoms;
#[macro_use]
extern crate style_derive;
#[macro_use]
extern crate to_shmem_derive;

#[macro_use]
mod macros;

pub mod animation;
pub mod applicable_declarations;
#[allow(missing_docs)] // TODO.
#[cfg(feature = "servo")]
pub mod attr;
pub mod author_styles;
pub mod bezier;
pub mod bloom;
#[path = "properties/computed_value_flags.rs"]
pub mod computed_value_flags;
pub mod context;
pub mod counter_style;
pub mod custom_properties;
pub mod data;
pub mod dom;
pub mod dom_apis;
pub mod driver;
pub mod element_state;
#[cfg(feature = "servo")]
mod encoding_support;
pub mod error_reporting;
pub mod font_face;
pub mod font_metrics;
#[cfg(feature = "gecko")]
#[allow(unsafe_code)]
pub mod gecko_bindings;
pub mod global_style_data;
pub mod hash;
pub mod invalidation;
#[allow(missing_docs)] // TODO.
pub mod logical_geometry;
pub mod matching;
#[macro_use]
pub mod media_queries;
pub mod parallel;
pub mod parser;
pub mod rule_cache;
pub mod rule_collector;
pub mod rule_tree;
pub mod scoped_tls;
pub mod selector_map;
pub mod selector_parser;
pub mod shared_lock;
pub mod sharing;
pub mod str;
pub mod style_adjuster;
pub mod style_resolver;
pub mod stylesheet_set;
pub mod stylesheets;
pub mod stylist;
pub mod thread_state;
pub mod traversal;
pub mod traversal_flags;
pub mod use_counters;
#[macro_use]
#[allow(non_camel_case_types)]
pub mod values;

#[cfg(feature = "gecko")]
pub use crate::gecko_string_cache as string_cache;
#[cfg(feature = "gecko")]
pub use crate::gecko_string_cache::Atom;
/// The namespace prefix type for Gecko, which is just an atom.
#[cfg(feature = "gecko")]
pub type Prefix = crate::values::AtomIdent;
/// The local name of an element for Gecko, which is just an atom.
#[cfg(feature = "gecko")]
pub type LocalName = crate::values::AtomIdent;
#[cfg(feature = "gecko")]
pub use crate::gecko_string_cache::Namespace;

#[cfg(feature = "servo")]
pub use servo_atoms::Atom;

#[cfg(feature = "servo")]
#[allow(missing_docs)]
pub type LocalName = crate::values::GenericAtomIdent<html5ever::LocalNameStaticSet>;
#[cfg(feature = "servo")]
#[allow(missing_docs)]
pub type Namespace = crate::values::GenericAtomIdent<html5ever::NamespaceStaticSet>;
#[cfg(feature = "servo")]
#[allow(missing_docs)]
pub type Prefix = crate::values::GenericAtomIdent<html5ever::PrefixStaticSet>;

pub use style_traits::arc_slice::ArcSlice;
pub use style_traits::owned_slice::OwnedSlice;
pub use style_traits::owned_str::OwnedStr;

/// The CSS properties supported by the style system.
/// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
#[deny(missing_docs)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}

#[cfg(feature = "gecko")]
#[allow(unsafe_code)]
pub mod gecko;

// uses a macro from properties
#[cfg(feature = "servo")]
#[allow(unsafe_code)]
pub mod servo;

#[cfg(feature = "gecko")]
#[allow(unsafe_code, missing_docs)]
pub mod gecko_properties {
    include!(concat!(env!("OUT_DIR"), "/gecko_properties.rs"));
}

macro_rules! reexport_computed_values {
    ( $( { $name: ident, $boxed: expr } )+ ) => {
        /// Types for [computed values][computed].
        ///
        /// [computed]: https://drafts.csswg.org/css-cascade/#computed
        pub mod computed_values {
            $(
                pub use crate::properties::longhands::$name::computed_value as $name;
            )+
            // Don't use a side-specific name needlessly:
            pub use crate::properties::longhands::border_top_style::computed_value as border_style;
        }
    }
}
longhand_properties_idents!(reexport_computed_values);

#[cfg(feature = "gecko")]
use crate::gecko_string_cache::WeakAtom;
#[cfg(feature = "servo")]
use servo_atoms::Atom as WeakAtom;

/// Extension methods for selectors::attr::CaseSensitivity
pub trait CaseSensitivityExt {
    /// Return whether two atoms compare equal according to this case sensitivity.
    fn eq_atom(self, a: &WeakAtom, b: &WeakAtom) -> bool;
}

impl CaseSensitivityExt for selectors::attr::CaseSensitivity {
    fn eq_atom(self, a: &WeakAtom, b: &WeakAtom) -> bool {
        match self {
            selectors::attr::CaseSensitivity::CaseSensitive => a == b,
            selectors::attr::CaseSensitivity::AsciiCaseInsensitive => a.eq_ignore_ascii_case(b),
        }
    }
}

/// A trait pretty much similar to num_traits::Zero, but without the need of
/// implementing `Add`.
pub trait Zero {
    /// Returns the zero value.
    fn zero() -> Self;

    /// Returns whether this value is zero.
    fn is_zero(&self) -> bool;
}

impl<T> Zero for T
where
    T: num_traits::Zero,
{
    fn zero() -> Self {
        <Self as num_traits::Zero>::zero()
    }

    fn is_zero(&self) -> bool {
        <Self as num_traits::Zero>::is_zero(self)
    }
}

/// A trait pretty much similar to num_traits::One, but without the need of
/// implementing `Mul`.
pub trait One {
    /// Reutrns the one value.
    fn one() -> Self;

    /// Returns whether this value is one.
    fn is_one(&self) -> bool;
}

impl<T> One for T
where
    T: num_traits::One + PartialEq,
{
    fn one() -> Self {
        <Self as num_traits::One>::one()
    }

    fn is_one(&self) -> bool {
        *self == One::one()
    }
}
