/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The pseudo-classes and pseudo-elements supported by the style system.

#![deny(missing_docs)]

use cssparser::{Parser as CssParser, ParserInput};
use selectors::parser::SelectorList;
use std::fmt::{self, Debug};
use style_traits::ParseError;
use stylesheets::{Origin, Namespaces, UrlExtraData};

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

/// Servo's selector parser.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorParser<'a> {
    /// The origin of the stylesheet we're parsing.
    pub stylesheet_origin: Origin,
    /// The namespace set of the stylesheet.
    pub namespaces: &'a Namespaces,
    /// The extra URL data of the stylesheet, which is used to look up
    /// whether we are parsing a chrome:// URL style sheet.
    pub url_data: Option<&'a UrlExtraData>,
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
            url_data: None,
        };
        let mut input = ParserInput::new(input);
        SelectorList::parse(&parser, &mut CssParser::new(&mut input))
    }

    /// Whether we're parsing selectors in a user-agent stylesheet.
    pub fn in_user_agent_stylesheet(&self) -> bool {
        matches!(self.stylesheet_origin, Origin::UserAgent)
    }

    /// Whether we're parsing selectors in a stylesheet that has chrome
    /// privilege.
    pub fn in_chrome_stylesheet(&self) -> bool {
        self.url_data.map_or(false, |d| d.is_chrome())
    }
}

/// This enumeration determines if a pseudo-element is eagerly cascaded or not.
///
/// If you're implementing a public selector for `Servo` that the end-user might
/// customize, then you probably need to make it eager.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// A per-functional-pseudo map, from a given pseudo to a `T`.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct PerPseudoElementMap<T> {
    entries: [Option<T>; SIMPLE_PSEUDO_COUNT],
}

impl<T> Default for PerPseudoElementMap<T> {
    fn default() -> Self {
        Self {
            entries: PseudoElement::simple_pseudo_none_array(),
        }
    }
}

impl<T> Debug for PerPseudoElementMap<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("[")?;
        let mut first = true;
        for entry in self.entries.iter() {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            entry.fmt(f)?;
        }
        f.write_str("]")
    }
}

impl<T> PerPseudoElementMap<T> {
    /// Get an entry in the map.
    pub fn get(&self, pseudo: &PseudoElement) -> Option<&T> {
        let index = match pseudo.simple_index() {
            Some(i) => i,
            None => return None,
        };
        self.entries[index].as_ref()
    }

    /// Clear this enumerated array.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Invokes a callback on each non-None entry.
    pub fn for_each<F: FnMut(&mut T)>(&mut self, mut f: F) {
        for entry in self.entries.iter_mut() {
            if entry.is_some() {
                f(entry.as_mut().unwrap());
            }
        }
    }

    /// Set an entry value.
    ///
    /// Returns an error if the element is not a simple pseudo.
    pub fn set(&mut self, pseudo: &PseudoElement, value: T) -> Result<(), ()> {
        let index = match pseudo.simple_index() {
            Some(i) => i,
            None => return Err(()),
        };
        self.entries[index] = Some(value);
        Ok(())
    }

    /// Get an entry for `pseudo`, or create it with calling `f`.
    pub fn get_or_insert_with<F>(
        &mut self,
        pseudo: &PseudoElement,
        f: F,
    ) -> Result<&mut T, ()>
    where
        F: FnOnce() -> T,
    {
        let index = match pseudo.simple_index() {
            Some(i) => i,
            None => return Err(()),
        };
        if self.entries[index].is_none() {
            self.entries[index] = Some(f());
        }
        Ok(self.entries[index].as_mut().unwrap())
    }

    /// Get an iterator for the entries.
    pub fn iter(&self) -> ::std::slice::Iter<Option<T>> {
        self.entries.iter()
    }
}
