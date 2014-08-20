/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "style"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules)]

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

#[phase(plugin)]
extern crate servo_macros = "macros";
extern crate servo_util = "util";


// Public API
pub use stylesheets::{Stylesheet, iter_font_face_rules};
pub use selector_matching::{Stylist, StylesheetOrigin, UserAgentOrigin, AuthorOrigin, UserOrigin};
pub use selector_matching::{DeclarationBlock, matches};
pub use properties::{cascade, cascade_anonymous};
pub use properties::{PropertyDeclaration, ComputedValues, computed_values, style_structs};
pub use properties::{PropertyDeclarationBlock, parse_style_attribute};  // Style attributes
pub use properties::{CSSFloat, DeclaredValue, PropertyDeclarationParseResult};
pub use properties::longhands;
pub use node::{TElement, TNode};
pub use selectors::{PseudoElement, Before, After, SelectorList, parse_selector_list_from_str};
pub use selectors::{AttrSelector, NamespaceConstraint, SpecificNamespace, AnyNamespace};

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
