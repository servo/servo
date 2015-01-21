/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

use node::{TElement, TElementAttributes, TNode};
use values::specified::CSSColor;
use values::{CSSFloat, specified};
use properties::DeclaredValue::SpecifiedValue;
use properties::PropertyDeclaration;
use selector_matching::{DeclarationBlock, Stylist};

use cssparser::Color;
use servo_util::geometry::Au;
use servo_util::smallvec::VecLike;
use servo_util::str::LengthOrPercentageOrAuto;

/// Legacy presentational attributes that take a length as defined in HTML5 § 2.4.4.4.
#[deriving(Copy, PartialEq, Eq)]
pub enum LengthAttribute {
    /// `<td width>`
    Width,
}

/// Legacy presentational attributes that take an integer as defined in HTML5 § 2.4.4.2.
#[deriving(Copy, PartialEq, Eq)]
pub enum IntegerAttribute {
    /// `<input size>`
    Size,
    Cols,
    Rows,
}

/// Legacy presentational attributes that take a nonnegative integer as defined in HTML5 § 2.4.4.2.
#[deriving(Copy, PartialEq, Eq)]
pub enum UnsignedIntegerAttribute {
    /// `<td border>`
    Border,
    /// `<td colspan>`
    ColSpan,
}

/// Legacy presentational attributes that take a simple color as defined in HTML5 § 2.4.6.
#[deriving(Copy, PartialEq, Eq)]
pub enum SimpleColorAttribute {
    /// `<body bgcolor>`
    BgColor,
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
    fn synthesize_presentational_hints_for_legacy_attributes<'a,E,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where E: TElement<'a> +
                                                                      TElementAttributes,
                                                                   N: TNode<'a,E>,
                                                                   V: VecLike<DeclarationBlock>;
    /// Synthesizes rules for the legacy `bgcolor` attribute.
    fn synthesize_presentational_hint_for_legacy_background_color_attribute<'a,E,V>(
                                                                            &self,
                                                                            element: E,
                                                                            matching_rules_list:
                                                                                &mut V,
                                                                            shareable: &mut bool)
                                                                            where
                                                                            E: TElement<'a> +
                                                                               TElementAttributes,
                                                                            V: VecLike<
                                                                                DeclarationBlock>;
    /// Synthesizes rules for the legacy `border` attribute.
    fn synthesize_presentational_hint_for_legacy_border_attribute<'a,E,V>(
                                                                  &self,
                                                                  element: E,
                                                                  matching_rules_list: &mut V,
                                                                  shareable: &mut bool)
                                                                  where
                                                                    E: TElement<'a> +
                                                                       TElementAttributes,
                                                                    V: VecLike<DeclarationBlock>;
}

impl PresentationalHintSynthesis for Stylist {
    fn synthesize_presentational_hints_for_legacy_attributes<'a,E,N,V>(
                                                             &self,
                                                             node: &N,
                                                             matching_rules_list: &mut V,
                                                             shareable: &mut bool)
                                                             where E: TElement<'a> +
                                                                      TElementAttributes,
                                                                   N: TNode<'a,E>,
                                                                   V: VecLike<DeclarationBlock> {
        let element = node.as_element();
        match element.get_local_name() {
            name if *name == atom!("td") => {
                match element.get_length_attribute(LengthAttribute::Width) {
                    LengthOrPercentageOrAuto::Auto => {}
                    LengthOrPercentageOrAuto::Percentage(percentage) => {
                        let width_value = specified::LengthOrPercentageOrAuto::Percentage(percentage);
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                    LengthOrPercentageOrAuto::Length(length) => {
                        let width_value = specified::LengthOrPercentageOrAuto::Length(specified::Length::Au(length));
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                }
                self.synthesize_presentational_hint_for_legacy_background_color_attribute(
                    element,
                    matching_rules_list,
                    shareable);
                self.synthesize_presentational_hint_for_legacy_border_attribute(
                    element,
                    matching_rules_list,
                    shareable);
            }
            name if *name == atom!("table") => {
                self.synthesize_presentational_hint_for_legacy_background_color_attribute(
                    element,
                    matching_rules_list,
                    shareable);
                self.synthesize_presentational_hint_for_legacy_border_attribute(
                    element,
                    matching_rules_list,
                    shareable);
            }
            name if *name == atom!("body") || *name == atom!("tr") || *name == atom!("thead") ||
                    *name == atom!("tbody") || *name == atom!("tfoot") => {
                self.synthesize_presentational_hint_for_legacy_background_color_attribute(
                    element,
                    matching_rules_list,
                    shareable);
            }
            name if *name == atom!("input") => {
                match element.get_integer_attribute(IntegerAttribute::Size) {
                    Some(value) if value != 0 => {
                        // Per HTML 4.01 § 17.4, this value is in characters if `type` is `text` or
                        // `password` and in pixels otherwise.
                        //
                        // FIXME(pcwalton): More use of atoms, please!
                        let value = match element.get_attr(&ns!(""), &atom!("type")) {
                            Some("text") | Some("password") => {
                                specified::Length::ServoCharacterWidth(value)
                            }
                            _ => specified::Length::Au(Au::from_px(value as int)),
                        };
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                PropertyDeclaration::Width(SpecifiedValue(
                                    specified::LengthOrPercentageOrAuto::Length(value)))));
                        *shareable = false
                    }
                    Some(_) | None => {}
                }
            }
            name if *name == atom!("textarea") => {
                match element.get_integer_attribute(IntegerAttribute::Cols) {
                    Some(value) if value != 0 => {
                        // TODO(mttr) ServoCharacterWidth uses the size math for <input type="text">, but
                        // the math for <textarea> is a little different since we need to take
                        // scrollbar size into consideration (but we don't have a scrollbar yet!)
                        //
                        // https://html.spec.whatwg.org/multipage/rendering.html#textarea-effective-width
                        let value = specified::Length::ServoCharacterWidth(value);
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
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
                        // https://html.spec.whatwg.org/multipage/rendering.html#textarea-effective-height
                        let value = specified::Length::Em(value as CSSFloat);
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                PropertyDeclaration::Height(SpecifiedValue(
                                    specified::LengthOrPercentageOrAuto::Length(value)))));
                        *shareable = false
                    }
                    Some(_) | None => {}
                }
            }
            _ => {}
        }
    }

    fn synthesize_presentational_hint_for_legacy_background_color_attribute<'a,E,V>(
                                                                            &self,
                                                                            element: E,
                                                                            matching_rules_list:
                                                                                &mut V,
                                                                            shareable: &mut bool)
                                                                            where
                                                                            E: TElement<'a> +
                                                                               TElementAttributes,
                                                                            V: VecLike<
                                                                                DeclarationBlock> {
        match element.get_simple_color_attribute(SimpleColorAttribute::BgColor) {
            None => {}
            Some(color) => {
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        PropertyDeclaration::BackgroundColor(SpecifiedValue(
                            CSSColor { parsed: Color::RGBA(color), authored: None }))));
                *shareable = false
            }
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
                                                                    V: VecLike<DeclarationBlock> {
        match element.get_unsigned_integer_attribute(UnsignedIntegerAttribute::Border) {
            None => {}
            Some(length) => {
                let width_value = specified::Length::Au(Au::from_px(length as int));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        PropertyDeclaration::BorderTopWidth(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        PropertyDeclaration::BorderLeftWidth(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        PropertyDeclaration::BorderBottomWidth(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        PropertyDeclaration::BorderRightWidth(SpecifiedValue(width_value))));
                *shareable = false
            }
        }
    }
}

