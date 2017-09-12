/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use selectors::gecko_like_types as dummies;
use servo_arc::Arc;
use std::mem::{size_of, align_of};
use style;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::data::{ElementData, ElementStyles, RestyleData};
use style::gecko::selector_parser as real;
use style::properties::ComputedValues;
use style::rule_tree::{RuleNode, StrongRuleNode};
use style::values::computed;
use style::values::specified;

#[test]
fn size_of_selectors_dummy_types() {
    assert_eq!(size_of::<dummies::PseudoClass>(), size_of::<real::NonTSPseudoClass>());
    assert_eq!(align_of::<dummies::PseudoClass>(), align_of::<real::NonTSPseudoClass>());

    assert_eq!(size_of::<dummies::PseudoElement>(), size_of::<real::PseudoElement>());
    assert_eq!(align_of::<dummies::PseudoElement>(), align_of::<real::PseudoElement>());

    assert_eq!(size_of::<dummies::Atom>(), size_of::<style::Atom>());
    assert_eq!(align_of::<dummies::Atom>(), align_of::<style::Atom>());
}

// The size of this is critical to performance on the bloom-basic microbenchmark.
// When iterating over a large Rule array, we want to be able to fast-reject
// selectors (with the inline hashes) with as few cache misses as possible.
size_of_test!(test_size_of_rule, style::stylist::Rule, 32);

// Large pages generate tens of thousands of ComputedValues.
size_of_test!(test_size_of_cv, ComputedValues, 272);

size_of_test!(test_size_of_option_arc_cv, Option<Arc<ComputedValues>>, 8);
size_of_test!(test_size_of_option_rule_node, Option<StrongRuleNode>, 8);

size_of_test!(test_size_of_element_styles, ElementStyles, 16);
size_of_test!(test_size_of_restyle_data, RestyleData, 8);
size_of_test!(test_size_of_element_data, ElementData, 24);

size_of_test!(test_size_of_property_declaration, style::properties::PropertyDeclaration, 32);

size_of_test!(test_size_of_application_declaration_block, ApplicableDeclarationBlock, 24);

// FIXME(bholley): This can shrink with a little bit of work.
// See https://github.com/servo/servo/issues/17280
size_of_test!(test_size_of_rule_node, RuleNode, 80);

// This is huge, but we allocate it on the stack and then never move it,
// we only pass `&mut SourcePropertyDeclaration` references around.
size_of_test!(test_size_of_parsed_declaration, style::properties::SourcePropertyDeclaration, 704);

size_of_test!(test_size_of_computed_image, computed::image::Image, 40);
size_of_test!(test_size_of_specified_image, specified::image::Image, 40);

// FIXME(bz): These can shrink if we move the None_ value inside the
// enum instead of paying an extra word for the Either discriminant.
size_of_test!(test_size_of_computed_image_layer, computed::image::ImageLayer, 48);
size_of_test!(test_size_of_specified_image_layer, specified::image::ImageLayer, 48);
