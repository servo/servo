/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use selectors;
use servo_arc::Arc;
use style;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::data::{ElementData, ElementStyles};
use style::gecko::selector_parser::{self, SelectorImpl};
use style::properties::ComputedValues;
use style::rule_tree::{RuleNode, StrongRuleNode};
use style::values::computed;
use style::values::specified;

size_of_test!(size_of_selector, selectors::parser::Selector<SelectorImpl>, 8);
size_of_test!(size_of_pseudo_element, selector_parser::PseudoElement, 24);

size_of_test!(size_of_component, selectors::parser::Component<SelectorImpl>, 32);
size_of_test!(size_of_pseudo_class, selector_parser::NonTSPseudoClass, 24);

// The size of this is critical to performance on the bloom-basic microbenchmark.
// When iterating over a large Rule array, we want to be able to fast-reject
// selectors (with the inline hashes) with as few cache misses as possible.
size_of_test!(test_size_of_rule, style::stylist::Rule, 32);

// Large pages generate tens of thousands of ComputedValues.
size_of_test!(test_size_of_cv, ComputedValues, 248);

size_of_test!(test_size_of_option_arc_cv, Option<Arc<ComputedValues>>, 8);
size_of_test!(test_size_of_option_rule_node, Option<StrongRuleNode>, 8);

size_of_test!(test_size_of_element_styles, ElementStyles, 16);
size_of_test!(test_size_of_element_data, ElementData, 24);

size_of_test!(test_size_of_property_declaration, style::properties::PropertyDeclaration, 32);

size_of_test!(test_size_of_application_declaration_block, ApplicableDeclarationBlock, 16);
size_of_test!(test_size_of_rule_node, RuleNode, 72);

// This is huge, but we allocate it on the stack and then never move it,
// we only pass `&mut SourcePropertyDeclaration` references around.
size_of_test!(test_size_of_parsed_declaration, style::properties::SourcePropertyDeclaration, 608);

size_of_test!(test_size_of_computed_image, computed::image::Image, 32);
size_of_test!(test_size_of_specified_image, specified::image::Image, 32);

// FIXME(bz): These can shrink if we move the None_ value inside the
// enum instead of paying an extra word for the Either discriminant.
size_of_test!(test_size_of_computed_image_layer, computed::image::ImageLayer, 32);
size_of_test!(test_size_of_specified_image_layer, specified::image::ImageLayer, 32);
