/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(globs, macro_rules, if_let)]

#![deny(unused_imports)]
#![deny(unused_variables)]

#![feature(phase)]
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate string_cache_macros;

extern crate collections;
extern crate geom;
extern crate serialize;
extern crate text_writer;
extern crate url;

extern crate cssparser;
extern crate encoding;
extern crate string_cache;

#[phase(plugin)]
extern crate string_cache_macros;

#[phase(plugin)]
extern crate plugins;

#[phase(plugin)]
extern crate lazy_static;

extern crate "util" as servo_util;


// Public API
pub use media_queries::{Device, MediaType};
pub use stylesheets::{Stylesheet, iter_font_face_rules};
pub use selector_matching::{Stylist, StylesheetOrigin};
pub use selector_matching::{DeclarationBlock, CommonStyleAffectingAttributes};
pub use selector_matching::{CommonStyleAffectingAttributeInfo, CommonStyleAffectingAttributeMode};
pub use selector_matching::{matches, matches_simple_selector, common_style_affecting_attributes};
pub use selector_matching::{rare_style_affecting_attributes};
pub use selector_matching::{RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE, SELECTOR_WHITESPACE};
pub use properties::{cascade, cascade_anonymous, computed, longhands_from_shorthand};
pub use properties::{is_supported_property, make_inline};
pub use properties::{PropertyDeclaration, ComputedValues, computed_values, style_structs};
pub use properties::{PropertyDeclarationBlock, parse_style_attribute};  // Style attributes
pub use properties::{CSSFloat, DeclaredValue, PropertyDeclarationParseResult};
pub use properties::{Angle, AngleOrCorner};
pub use properties::{HorizontalDirection, VerticalDirection};
pub use node::{TElement, TElementAttributes, TNode};
pub use selectors::{PseudoElement, ParserContext, SelectorList};
pub use selectors::{AttrSelector, NamespaceConstraint};
pub use selectors::{SimpleSelector, parse_selector_list_from_str};
pub use cssparser::{Color, RGBA};
pub use legacy::{IntegerAttribute, LengthAttribute};
pub use legacy::{SimpleColorAttribute, UnsignedIntegerAttribute};
pub use font_face::Source;

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
mod legacy;
