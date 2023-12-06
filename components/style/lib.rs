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
#[macro_use]
#[cfg(feature = "gecko")]
extern crate gecko_profiler;
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
extern crate static_assertions;
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
pub mod color;
#[path = "properties/computed_value_flags.rs"]
pub mod computed_value_flags;
pub mod context;
pub mod counter_style;
pub mod custom_properties;
pub mod data;
pub mod dom;
pub mod dom_apis;
pub mod driver;
#[cfg(feature = "servo")]
mod encoding_support;
pub mod error_reporting;
pub mod font_face;
pub mod font_metrics;
#[cfg(feature = "gecko")]
#[allow(unsafe_code)]
pub mod gecko_bindings;
pub mod global_style_data;
pub mod invalidation;
#[allow(missing_docs)] // TODO.
pub mod logical_geometry;
pub mod matching;
pub mod media_queries;
pub mod parallel;
pub mod parser;
pub mod piecewise_linear;
pub mod properties_and_values;
#[macro_use]
pub mod queries;
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

use std::hash::{BuildHasher, Hash};

#[macro_use]
pub mod properties;

#[cfg(feature = "gecko")]
#[allow(unsafe_code)]
pub mod gecko;

// uses a macro from properties
#[cfg(feature = "servo")]
#[allow(unsafe_code)]
pub mod servo;

macro_rules! reexport_computed_values {
    ( $( { $name: ident } )+ ) => {
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

/// A trait implementing a function to tell if the number is zero without a percent
pub trait ZeroNoPercent {
    /// So, `0px` should return `true`, but `0%` or `1px` should return `false`
    fn is_zero_no_percent(&self) -> bool;
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

/// An allocation error.
///
/// TODO(emilio): Would be nice to have more information here, or for SmallVec
/// to return the standard error type (and then we can just return that).
///
/// But given we use these mostly to bail out and ignore them, it's not a big
/// deal.
#[derive(Debug)]
pub struct AllocErr;

impl From<smallvec::CollectionAllocErr> for AllocErr {
    #[inline]
    fn from(_: smallvec::CollectionAllocErr) -> Self {
        Self
    }
}

impl From<std::collections::TryReserveError> for AllocErr {
    #[inline]
    fn from(_: std::collections::TryReserveError) -> Self {
        Self
    }
}

/// Shrink the capacity of the collection if needed.
pub(crate) trait ShrinkIfNeeded {
    fn shrink_if_needed(&mut self);
}

/// We shrink the capacity of a collection if we're wasting more than a 25% of
/// its capacity, and if the collection is arbitrarily big enough
/// (>= CAPACITY_THRESHOLD entries).
#[inline]
fn should_shrink(len: usize, capacity: usize) -> bool {
    const CAPACITY_THRESHOLD: usize = 64;
    capacity >= CAPACITY_THRESHOLD && len + capacity / 4 < capacity
}

impl<K, V, H> ShrinkIfNeeded for std::collections::HashMap<K, V, H>
where
    K: Eq + Hash,
    H: BuildHasher,
{
    fn shrink_if_needed(&mut self) {
        if should_shrink(self.len(), self.capacity()) {
            self.shrink_to_fit();
        }
    }
}

impl<T, H> ShrinkIfNeeded for std::collections::HashSet<T, H>
where
    T: Eq + Hash,
    H: BuildHasher,
{
    fn shrink_if_needed(&mut self) {
        if should_shrink(self.len(), self.capacity()) {
            self.shrink_to_fit();
        }
    }
}

// TODO(emilio): Measure and see if we're wasting a lot of memory on Vec /
// SmallVec, and if so consider shrinking those as well.
