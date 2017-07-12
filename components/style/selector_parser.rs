/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The pseudo-classes and pseudo-elements supported by the style system.

#![deny(missing_docs)]

use cssparser::{Parser as CssParser, ParserInput};
use selectors::Element;
use selectors::parser::SelectorList;
use std::fmt::Debug;
use style_traits::ParseError;
use stylesheets::{Origin, Namespaces};

/// A convenient alias for the type that represents an attribute value used for
/// selector parser implementation.
pub type AttrValue = <SelectorImpl as ::selectors::SelectorImpl>::AttrValue;

#[cfg(feature = "servo")]
pub use servo::selector_parser::*;

#[cfg(feature = "gecko")]
pub use gecko::selector_parser::*;

#[cfg(feature = "servo")]
pub use servo::selector_parser::ServoElementSnapshot as Snapshot;

#[cfg(feature = "gecko")]
pub use gecko::snapshot::GeckoElementSnapshot as Snapshot;

#[cfg(feature = "servo")]
pub use servo::restyle_damage::ServoRestyleDamage as RestyleDamage;

#[cfg(feature = "gecko")]
pub use gecko::restyle_damage::GeckoRestyleDamage as RestyleDamage;

/// A type that represents the previous computed values needed for restyle
/// damage calculation.
#[cfg(feature = "servo")]
pub type PreExistingComputedValues = ::properties::ServoComputedValues;

/// A type that represents the previous computed values needed for restyle
/// damage calculation.
#[cfg(feature = "gecko")]
pub type PreExistingComputedValues = ::gecko_bindings::structs::nsStyleContext;

/// Servo's selector parser.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorParser<'a> {
    /// The origin of the stylesheet we're parsing.
    pub stylesheet_origin: Origin,
    /// The namespace set of the stylesheet.
    pub namespaces: &'a Namespaces,
}

impl<'a> SelectorParser<'a> {
    /// Parse a selector list with an author origin and without taking into
    /// account namespaces.
    ///
    /// This is used for some DOM APIs like `querySelector`.
    pub fn parse_author_origin_no_namespace(input: &str)
                                            -> Result<SelectorList<SelectorImpl>, ParseError> {
        let namespaces = Namespaces::default();
        let parser = SelectorParser {
            stylesheet_origin: Origin::Author,
            namespaces: &namespaces,
        };
        let mut input = ParserInput::new(input);
        SelectorList::parse(&parser, &mut CssParser::new(&mut input))
    }

    /// Whether we're parsing selectors in a user-agent stylesheet.
    pub fn in_user_agent_stylesheet(&self) -> bool {
        matches!(self.stylesheet_origin, Origin::UserAgent)
    }
}

/// This enumeration determines if a pseudo-element is eagerly cascaded or not.
///
/// If you're implementing a public selector for `Servo` that the end-user might
/// customize, then you probably need to make it eager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElementCascadeType {
    /// Eagerly cascaded pseudo-elements are "normal" pseudo-elements (i.e.
    /// `::before` and `::after`). They inherit styles normally as another
    /// selector would do, and they're computed as part of the cascade.
    Eager,
    /// Lazy pseudo-elements are affected by selector matching, but they're only
    /// computed when needed, and not before. They're useful for general
    /// pseudo-elements that are not very common.
    ///
    /// Note that in Servo lazy pseudo-elements are restricted to a subset of
    /// selectors, so you can't use it for public pseudo-elements. This is not
    /// the case with Gecko though.
    Lazy,
    /// Precomputed pseudo-elements skip the cascade process entirely, mostly as
    /// an optimisation since they are private pseudo-elements (like
    /// `::-servo-details-content`).
    ///
    /// This pseudo-elements are resolved on the fly using *only* global rules
    /// (rules of the form `*|*`), and applying them to the parent style.
    Precomputed,
}

/// An extension to rust-selector's `Element` trait.
pub trait ElementExt: Element<Impl=SelectorImpl> + Debug {
    /// Whether this element should match user and author rules.
    ///
    /// We use this for Native Anonymous Content in Gecko.
    fn matches_user_and_author_rules(&self) -> bool;
}

impl SelectorImpl {
    /// A helper to traverse each precomputed pseudo-element, executing `fun` on
    /// it.
    ///
    /// The optimization comment in `each_eagerly_cascaded_pseudo_element` also
    /// applies here.
    #[inline]
    pub fn each_precomputed_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement),
    {
        Self::each_simple_pseudo_element(|pseudo| {
            if pseudo.is_precomputed() {
                fun(pseudo)
            }
        })
    }
}
