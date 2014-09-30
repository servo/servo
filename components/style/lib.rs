/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules)]

#![deny(unused_imports, unused_variable)]

#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate debug;
extern crate collections;
extern crate geom;
extern crate num;
extern crate serialize;
extern crate sync;
extern crate url;

extern crate cssparser;
extern crate encoding;
extern crate string_cache;

#[phase(plugin)]
extern crate string_cache_macros;

#[phase(plugin)]
extern crate lazy_static;

extern crate "util" as servo_util;


// Public API
pub use stylesheets::{Stylesheet, iter_font_face_rules};
pub use selector_matching::{Stylist, StylesheetOrigin, UserAgentOrigin, AuthorOrigin, UserOrigin};
pub use selector_matching::{DeclarationBlock, matches,matches_simple_selector};
pub use selector_matching::{RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE,SELECTOR_WHITESPACE};
pub use properties::{cascade, cascade_anonymous};
pub use properties::{PropertyDeclaration, ComputedValues, computed_values, style_structs};
pub use properties::{PropertyDeclarationBlock, parse_style_attribute};  // Style attributes
pub use properties::{CSSFloat, DeclaredValue, PropertyDeclarationParseResult};
pub use properties::longhands;
pub use node::{TElement, TNode};
pub use selectors::{PseudoElement, Before, After, SelectorList, parse_selector_list_from_str};
pub use selectors::{AttrSelector, NamespaceConstraint, SpecificNamespace, AnyNamespace};
pub use selectors::{SimpleSelector,LocalNameSelector};
pub use cssparser::{Color, RGBA};

mod stylesheets;
mod errors;
mod selectors;
mod selector_matching;
mod properties;
mod namespaces;
mod node;
mod media_queries;
mod parsing_utils;
mod font_face;
