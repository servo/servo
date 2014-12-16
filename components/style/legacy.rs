/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

use node::{TElement, TElementAttributes, TNode};
use properties::{BackgroundColorDeclaration, BorderBottomWidthDeclaration};
use properties::{BorderLeftWidthDeclaration, BorderRightWidthDeclaration};
use properties::{BorderTopWidthDeclaration, SpecifiedValue, WidthDeclaration, specified};
use selector_matching::{DeclarationBlock, Stylist};

use cssparser::RGBAColor;
use servo_util::geometry::Au;
use servo_util::smallvec::VecLike;
use servo_util::str::{AutoLpa, LengthLpa, PercentageLpa};

/// Legacy presentational attributes that take a length as defined in HTML5 § 2.4.4.4.
pub enum LengthAttribute {
    /// `<td width>`
    WidthLengthAttribute,
}

/// Legacy presentational attributes that take an integer as defined in HTML5 § 2.4.4.2.
pub enum IntegerAttribute {
    /// `<input size>`
    SizeIntegerAttribute,
}

/// Legacy presentational attributes that take a nonnegative integer as defined in HTML5 § 2.4.4.2.
pub enum UnsignedIntegerAttribute {
    /// `<td border>`
    BorderUnsignedIntegerAttribute,
    /// `<td colspan>`
    ColSpanUnsignedIntegerAttribute,
}

/// Legacy presentational attributes that take a simple color as defined in HTML5 § 2.4.6.
pub enum SimpleColorAttribute {
    /// `<body bgcolor>`
    BgColorSimpleColorAttribute,
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
                match element.get_length_attribute(WidthLengthAttribute) {
                    AutoLpa => {}
                    PercentageLpa(percentage) => {
                        let width_value = specified::LPA_Percentage(percentage);
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(width_value))));
                        *shareable = false
                    }
                    LengthLpa(length) => {
                        let width_value = specified::LPA_Length(specified::Au_(length));
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(width_value))));
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
                match element.get_integer_attribute(SizeIntegerAttribute) {
                    Some(value) if value != 0 => {
                        // Per HTML 4.01 § 17.4, this value is in characters if `type` is `text` or
                        // `password` and in pixels otherwise.
                        //
                        // FIXME(pcwalton): More use of atoms, please!
                        let value = match element.get_attr(&ns!(""), &atom!("type")) {
                            Some("text") | Some("password") => {
                                specified::ServoCharacterWidth(value)
                            }
                            _ => specified::Au_(Au::from_px(value as int)),
                        };
                        matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                                WidthDeclaration(SpecifiedValue(specified::LPA_Length(
                                            value)))));
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
        match element.get_simple_color_attribute(BgColorSimpleColorAttribute) {
            None => {}
            Some(color) => {
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        BackgroundColorDeclaration(SpecifiedValue(RGBAColor(color)))));
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
        match element.get_unsigned_integer_attribute(BorderUnsignedIntegerAttribute) {
            None => {}
            Some(length) => {
                let width_value = specified::Au_(Au::from_px(length as int));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        BorderTopWidthDeclaration(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        BorderLeftWidthDeclaration(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        BorderBottomWidthDeclaration(SpecifiedValue(width_value))));
                matching_rules_list.vec_push(DeclarationBlock::from_declaration(
                        BorderRightWidthDeclaration(SpecifiedValue(width_value))));
                *shareable = false
            }
        }
    }
}

