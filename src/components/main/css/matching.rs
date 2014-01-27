/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::incremental;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;

use extra::arc::Arc;
use style::{TNode, Stylist, cascade};
use style::{Before, After};

pub trait MatchMethods {
    fn match_node(&self, stylist: &Stylist);
    fn match_subtree(&self, stylist: &Stylist);

    unsafe fn cascade_node(&self, parent: Option<LayoutNode>);
    fn cascade_subtree(&self, parent: Option<LayoutNode>);
}

impl<'ln> MatchMethods for LayoutNode<'ln> {
    fn match_node(&self, stylist: &Stylist) {
        let style_attribute = self.with_element(|element| {
            match *element.style_attribute() {
                None => None,
                Some(ref style_attribute) => Some(style_attribute)
            }
        });

        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            Some(ref mut layout_data) => {
                layout_data.data.applicable_declarations = stylist.get_applicable_declarations(
                    self, style_attribute, None);
                layout_data.data.before_applicable_declarations = stylist.get_applicable_declarations(
                    self, None, Some(Before));
                layout_data.data.after_applicable_declarations = stylist.get_applicable_declarations(
                    self, None, Some(After));
            }
            None => fail!("no layout data")
        }
    }

    fn match_subtree(&self, stylist: &Stylist) {
        for node in self.traverse_preorder() {
            if node.is_element() {
                node.match_node(stylist);
            }
        }
    }

    unsafe fn cascade_node(&self, parent: Option<LayoutNode>) {
        macro_rules! cascade_node(
            ($applicable_declarations: ident, $style: ident) => {{
                // Get our parent's style. This must be unsafe so that we don't touch the parent's
                // borrow flags.
                //
                // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
                // enforced safe, race-free access to the parent style.
                let parent_style = match parent {
                    None => None,
                    Some(parent_node) => {
                        let parent_layout_data = parent_node.borrow_layout_data_unchecked();
                        match *parent_layout_data {
                            None => fail!("no parent data?!"),
                            Some(ref parent_layout_data) => {
                                match parent_layout_data.data.style {
                                    None => fail!("parent hasn't been styled yet?!"),
                                    Some(ref style) => Some(style.get()),
                                }
                            }
                        }
                    }
                };

                let computed_values = {
                    let layout_data_ref = self.borrow_layout_data();
                    let layout_data = layout_data_ref.get().as_ref().unwrap();
                    Arc::new(cascade(layout_data.data.$applicable_declarations, parent_style))
                };

                let mut layout_data_ref = self.mutate_layout_data();
                match *layout_data_ref.get() {
                    None => fail!("no layout data"),
                    Some(ref mut layout_data) => {
                        let style = &mut layout_data.data.$style;
                        match *style {
                            None => (),
                            Some(ref previous_style) => {
                                layout_data.data.restyle_damage = Some(incremental::compute_damage(
                                    previous_style.get(), computed_values.get()).to_int())
                            }
                        }
                        *style = Some(computed_values)
                    }
                }
            }}
        );

        {
            let before_len = {
                let layout_data_ref = self.borrow_layout_data();
                layout_data_ref.get().as_ref().unwrap().data.before_applicable_declarations.len()
            };
            if before_len > 0 {
                cascade_node!(before_applicable_declarations, before_style);
            }
        }
        cascade_node!(applicable_declarations, style);
        {
            let after_len = {
                let layout_data_ref = self.borrow_layout_data();
                layout_data_ref.get().as_ref().unwrap().data.after_applicable_declarations.len()
            };
            if after_len > 0 {
                cascade_node!(after_applicable_declarations, after_style);
            }
        }
    }

    fn cascade_subtree(&self, parent: Option<LayoutNode>) {
        unsafe {
            self.cascade_node(parent);
        }

        for kid in self.children() {
            if kid.is_element() {
                kid.cascade_subtree(Some(*self));
            }
        }
    }
}
