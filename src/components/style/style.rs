/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo#style:0.1"]
#![crate_type = "lib"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules)]

#![feature(phase)]
#[phase(syntax, link)] extern crate log;

extern crate cssparser;
extern crate collections;
extern crate encoding;
extern crate num;
extern crate serialize;
extern crate servo_util = "util";
extern crate sync;
extern crate url;


// Public API
pub use stylesheets::{Stylesheet, CSSRule, StyleRule};
pub use selector_matching::{Stylist, StylesheetOrigin, UserAgentOrigin, AuthorOrigin, UserOrigin};
pub use selector_matching::{MatchedProperty};
pub use properties::{cascade, PropertyDeclaration, ComputedValues, computed_values, style_structs};
pub use properties::{PropertyDeclarationBlock, parse_style_attribute};  // Style attributes
pub use properties::{initial_values, CSSFloat, DeclaredValue, PropertyDeclarationParseResult};
pub use properties::longhands;
pub use errors::with_errors_silenced;
pub use node::{TElement, TNode};
pub use selectors::{PseudoElement, Before, After, AttrSelector, SpecificNamespace, AnyNamespace};
pub use selectors::{NamespaceConstraint, Selector, CompoundSelector, SimpleSelector, Combinator};
pub use namespaces::NamespaceMap;
pub use media_queries::{MediaRule, MediaQueryList, MediaQuery, Device, MediaType, MediaQueryType};

mod stylesheets;
mod errors;
mod selectors;
mod selector_matching;
mod properties;
mod namespaces;
mod node;
mod media_queries;
mod parsing_utils;
