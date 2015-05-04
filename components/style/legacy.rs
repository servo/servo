/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

use std::sync::Arc;

use selectors::tree::{TElement, TNode};
use selectors::matching::DeclarationBlock;
use node::TElementAttributes;
use values::{CSSFloat, specified};
use properties::DeclaredValue::SpecifiedValue;
use properties::PropertyDeclaration;
use properties::longhands::{self, border_spacing};
use selector_matching::Stylist;

use util::geometry::Au;
use util::smallvec::VecLike;
use util::str::LengthOrPercentageOrAuto;

/// Legacy presentational attributes that take a length as defined in HTML5 § 2.4.4.4.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LengthAttribute {
    /// `<td width>`
    Width,
}

/// Legacy presentational attributes that take an integer as defined in HTML5 § 2.4.4.2.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IntegerAttribute {
    /// `<input size>`
    Size,
    Cols,
    Rows,
}

/// Legacy presentational attributes that take a nonnegative integer as defined in HTML5 § 2.4.4.2.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UnsignedIntegerAttribute {
    /// `<td border>`
    Border,
    /// `<table cellspacing>`
    CellSpacing,
    /// `<td colspan>`
    ColSpan,
}

/// Extension methods for `Stylist` that cause rules to be synthesized for legacy attributes.
pub trait PresentationalHintSynthesis {
    /// Synthesizes rules from various HTML attributes (mostly legacy junk from HTML4) that confer
    /// *presentational hints* as defined in the HTML5 specification. This handles stuff like
    /// `<body bgcolor>`, `<input size>`, `<td width>`, and so forth.
    ///
    /// NB: Beware! If you add an attribute to this list, be sure to add it to
    /// `common_style_affecting_attributes` or `rare_style_affecting_attributes` as appropriate. If
    /// you don't, you risk strange random nondeterministic failures due to false positives in
    /// style sharing.
    fn synthesize_presentational_hints_for_legacy_attributes<'a,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where N: TNode<'a>,
                                                                   N::Element: TElementAttributes,
                                                                   V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    /// Synthesizes rules for the legacy `border` attribute.
    fn synthesize_presentational_hint_for_legacy_border_attribute<'a,E,V>(
                                                                  &self,
                                                                  element: E,
                                                                  matching_rules_list: &mut V,
                                                                  shareable: &mut bool)
                                                                  where
                                                                    E: TElement<'a> +
                                                                       TElementAttributes,
                                                                    V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
}

impl PresentationalHintSynthesis for Stylist {
    fn synthesize_presentational_hints_for_legacy_attributes<'a,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where N: TNode<'a>,
                                                                   N::Element: TElementAttributes,
                                                                   V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>> {
        let element = node.as_element();

        let length = matching_rules_list.len();
        element.synthesize_presentational_hints_for_legacy_attributes(matching_rules_list);
        if matching_rules_list.len() != length {
            // Never share style for elements with preshints
            *shareable = false;
        }

        match element.get_local_name() {
            name if *name == atom!("td") => {
                match element.get_length_attribute(LengthAttribute::Width) {
                    LengthOrPercentageOrAuto::Auto => {}
                    LengthOrPercentageOrAuto::Percentage(percentage) => {
                        let width_value = specified::LengthOrPercentageOrAuto::Percentage(percentage);
                        matching_rules_list.push(from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                    LengthOrPercentageOrAuto::Length(length) => {
                        let width_value = specified::LengthOrPercentageOrAuto::Length(specified::Length::Absolute(length));
                        matching_rules_list.push(from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                }
                self.synthesize_presentational_hint_for_legacy_border_attribute(
                    element,
                    matching_rules_list,
                    shareable);
            }
            name if *name == atom!("table") => {
                self.synthesize_presentational_hint_for_legacy_border_attribute(
                    element,
                    matching_rules_list,
                    shareable);
                match element.get_unsigned_integer_attribute(
                        UnsignedIntegerAttribute::CellSpacing) {
                    None => {}
                    Some(length) => {
                        let width_value = specified::Length::Absolute(Au::from_px(length as isize));
                        matching_rules_list.push(from_declaration(
                                PropertyDeclaration::BorderSpacing(
                                    SpecifiedValue(
                                        border_spacing::SpecifiedValue {
                                            horizontal: width_value,
                                            vertical: width_value,
                                        }))));
                        *shareable = false
                    }
                }
            }
            name if *name == atom!("input") => {
                // FIXME(pcwalton): More use of atoms, please!
                match element.get_attr(&ns!(""), &atom!("type")) {
                    Some("text") | Some("password") => {
                        match element.get_integer_attribute(IntegerAttribute::Size) {
                            Some(value) if value != 0 => {
                                let value = specified::Length::ServoCharacterWidth(
                                    specified::CharacterWidth(value));
                                matching_rules_list.push(from_declaration(
                                        PropertyDeclaration::Width(SpecifiedValue(
                                            specified::LengthOrPercentageOrAuto::Length(value)))));
                                *shareable = false
                            }
                            Some(_) | None => {}
                        }
                    }
                    _ => {}
                };
            }
            name if *name == atom!("textarea") => {
                match element.get_integer_attribute(IntegerAttribute::Cols) {
                    Some(value) if value != 0 => {
                        // TODO(mttr) ServoCharacterWidth uses the size math for <input type="text">, but
                        // the math for <textarea> is a little different since we need to take
                        // scrollbar size into consideration (but we don't have a scrollbar yet!)
                        //
                        // https://html.spec.whatwg.org/multipage/#textarea-effective-width
                        let value = specified::Length::ServoCharacterWidth(specified::CharacterWidth(value));
                        matching_rules_list.push(from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(
                                    specified::LengthOrPercentageOrAuto::Length(value)))));
                        *shareable = false
                    }
                    Some(_) | None => {}
                }
                match element.get_integer_attribute(IntegerAttribute::Rows) {
                    Some(value) if value != 0 => {
                        // TODO(mttr) This should take scrollbar size into consideration.
                        //
                        // https://html.spec.whatwg.org/multipage/#textarea-effective-height
                        let value = specified::Length::FontRelative(specified::FontRelativeLength::Em(value as CSSFloat));
                        matching_rules_list.push(from_declaration(
                                PropertyDeclaration::Height(SpecifiedValue(
                                    longhands::height::SpecifiedValue(
                                        specified::LengthOrPercentageOrAuto::Length(value))))));
                        *shareable = false
                    }
                    Some(_) | None => {}
                }
            }
            _ => {}
        }
    }

    fn synthesize_presentational_hint_for_legacy_border_attribute<'a,E,V>(
                                                                  &self,
                                                                  element: E,
                                                                  matching_rules_list: &mut V,
                                                                  shareable: &mut bool)
                                                                  where
                                                                    E: TElement<'a> +
                                                                       TElementAttributes,
                                                                    V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>> {
        match element.get_unsigned_integer_attribute(UnsignedIntegerAttribute::Border) {
            None => {}
            Some(length) => {
                let width_value = specified::Length::Absolute(Au::from_px(length as isize));
                matching_rules_list.push(from_declaration(
                        PropertyDeclaration::BorderTopWidth(SpecifiedValue(
                            longhands::border_top_width::SpecifiedValue(width_value)))));
                matching_rules_list.push(from_declaration(
                        PropertyDeclaration::BorderLeftWidth(SpecifiedValue(
                            longhands::border_left_width::SpecifiedValue(width_value)))));
                matching_rules_list.push(from_declaration(
                        PropertyDeclaration::BorderBottomWidth(SpecifiedValue(
                            longhands::border_bottom_width::SpecifiedValue(width_value)))));
                matching_rules_list.push(from_declaration(
                        PropertyDeclaration::BorderRightWidth(SpecifiedValue(
                            longhands::border_right_width::SpecifiedValue(width_value)))));
                *shareable = false
            }
        }
    }
}


/// A convenience function to create a declaration block from a single declaration. This is
/// primarily used in `synthesize_rules_for_legacy_attributes`.
#[inline]
pub fn from_declaration(rule: PropertyDeclaration) -> DeclarationBlock<Vec<PropertyDeclaration>> {
    DeclarationBlock::from_declarations(Arc::new(vec![rule]))
}
