/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use std::cell::Cell;
use css::node_style::StyledNode;

use script::dom::node::{AbstractNode, LayoutView};
use style::Stylist;
use style::cascade;
use servo_util::tree::TreeNodeRef;

pub trait MatchMethods {
    fn match_node(&self, stylist: &Stylist);
    fn match_subtree(&self, stylist: &Stylist);

    fn cascade_node(&self, parent: Option<AbstractNode<LayoutView>>);
    fn cascade_subtree(&self, parent: Option<AbstractNode<LayoutView>>);
}

impl MatchMethods for AbstractNode<LayoutView> {
    fn match_node(&self, stylist: &Stylist) {
        let applicable_declarations = do self.with_imm_element |element| {
            let style_attribute = match element.style_attribute {
                None => None,
                Some(ref style_attribute) => Some(style_attribute)
            };
            stylist.get_applicable_declarations(self, style_attribute, None)
        };
        let cell = Cell::new(applicable_declarations);
        do self.write_layout_data |data| {
            data.applicable_declarations = cell.take();
        }
    }
    fn match_subtree(&self, stylist: &Stylist) {
        self.match_node(stylist);

        for kid in self.children() {
            if kid.is_element() {
                kid.match_subtree(stylist); 
            }
        }
    }

    fn cascade_node(&self, parent: Option<AbstractNode<LayoutView>>) {
        let parent_style = match parent {
            Some(parent) => Some(parent.style()),
            None => None
        };
        let computed_values = do self.read_layout_data |data| {
            cascade(data.applicable_declarations, parent_style)
        };
        let cell = Cell::new(computed_values);
        do self.write_layout_data |data| {
            data.style = Some(cell.take());
        }
    }
    fn cascade_subtree(&self, parent: Option<AbstractNode<LayoutView>>) {
        self.cascade_node(parent);

        for kid in self.children() {
            if kid.is_element() {
                kid.cascade_subtree(Some(*self));
            }
        }
    }
}
