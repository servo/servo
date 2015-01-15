/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![feature(int_uint)]
#![feature(box_syntax)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(missing_copy_implementations)]
#![allow(unstable)]

#[macro_use] extern crate log;
#[no_link] #[macro_use] #[plugin] extern crate string_cache_macros;

extern crate collections;
extern crate geom;
extern crate serialize;
extern crate text_writer;
extern crate url;

#[macro_use]
extern crate cssparser;

#[macro_use]
extern crate matches;

extern crate encoding;
extern crate string_cache;

#[macro_use]
extern crate lazy_static;

extern crate "util" as servo_util;


pub use media_queries::{Device, MediaType};
pub use stylesheets::{Stylesheet, iter_font_face_rules};
pub use selector_matching::{Stylist};
pub use selector_matching::{DeclarationBlock, CommonStyleAffectingAttributes};
pub use selector_matching::{CommonStyleAffectingAttributeInfo, CommonStyleAffectingAttributeMode};
pub use selector_matching::{matches, matches_simple_selector, common_style_affecting_attributes};
pub use selector_matching::{rare_style_affecting_attributes};
pub use selector_matching::{RECOMMENDED_SELECTOR_BLOOM_FILTER_SIZE, SELECTOR_WHITESPACE};
pub use properties::{cascade, cascade_anonymous, longhands_from_shorthand};
pub use properties::{is_supported_property, make_inline};
pub use properties::{PropertyDeclaration};
pub use properties::{computed_values, ComputedValues, style_structs};
pub use properties::{PropertyDeclarationBlock, parse_style_attribute};  // Style attributes
pub use properties::{DeclaredValue, PropertyDeclarationParseResult};
pub use values::CSSFloat;
pub use values::specified::{Angle, AngleOrCorner, HorizontalDirection, VerticalDirection};
pub use values::computed;
pub use node::{TElement, TElementAttributes, TNode};
pub use selectors::{PseudoElement, SelectorList};
pub use selectors::{AttrSelector, NamespaceConstraint};
pub use selectors::{SimpleSelector, parse_author_origin_selector_list_from_str};
pub use cssparser::{Color, RGBA};
pub use legacy::{IntegerAttribute, LengthAttribute};
pub use legacy::{SimpleColorAttribute, UnsignedIntegerAttribute};
pub use font_face::Source;
pub use stylesheets::Origin as StylesheetOrigin;

pub mod stylesheets;
pub mod parser;
pub mod selectors;
pub mod selector_matching;
#[macro_use] pub mod values;
pub mod properties;
pub mod namespaces;
pub mod node;
pub mod media_queries;
pub mod font_face;
pub mod legacy;
