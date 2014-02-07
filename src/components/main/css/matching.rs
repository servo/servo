/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::extra::LayoutAuxMethods;
use layout::incremental;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;

use extra::arc::Arc;
use script::layout_interface::LayoutChan;
use servo_util::smallvec::{SmallVec, SmallVec0, SmallVec16};
use style::{After, Before, ComputedValues, PropertyDeclaration, Stylist, TNode, cascade};

pub struct ApplicableDeclarations {
    normal: SmallVec16<Arc<~[PropertyDeclaration]>>,
    before: SmallVec0<Arc<~[PropertyDeclaration]>>,
    after: SmallVec0<Arc<~[PropertyDeclaration]>>,
}

impl ApplicableDeclarations {
    pub fn new() -> ApplicableDeclarations {
        ApplicableDeclarations {
            normal: SmallVec16::new(),
            before: SmallVec0::new(),
            after: SmallVec0::new(),
        }
    }

    pub fn clear(&mut self) {
        self.normal = SmallVec16::new();
        self.before = SmallVec0::new();
        self.after = SmallVec0::new();
    }
}

pub trait MatchMethods {
    /// Performs aux initialization, selector matching, and cascading sequentially.
    fn match_and_cascade_subtree(&self,
                                 stylist: &Stylist,
                                 layout_chan: &LayoutChan,
                                 applicable_declarations: &mut ApplicableDeclarations,
                                 initial_values: &ComputedValues,
                                 parent: Option<LayoutNode>);

    fn match_node(&self, stylist: &Stylist, applicable_declarations: &mut ApplicableDeclarations);

    unsafe fn cascade_node(&self,
                           parent: Option<LayoutNode>,
                           initial_values: &ComputedValues,
                           applicable_declarations: &ApplicableDeclarations);
}

impl<'ln> MatchMethods for LayoutNode<'ln> {
    fn match_node(&self, stylist: &Stylist, applicable_declarations: &mut ApplicableDeclarations) {
        let style_attribute = self.with_element(|element| {
            match *element.style_attribute() {
                None => None,
                Some(ref style_attribute) => Some(style_attribute)
            }
        });

        stylist.push_applicable_declarations(self,
                                             style_attribute,
                                             None,
                                             &mut applicable_declarations.normal);
        stylist.push_applicable_declarations(self,
                                             None,
                                             Some(Before),
                                             &mut applicable_declarations.before);
        stylist.push_applicable_declarations(self,
                                             None,
                                             Some(After),
                                             &mut applicable_declarations.after);
    }

    fn match_and_cascade_subtree(&self,
                                 stylist: &Stylist,
                                 layout_chan: &LayoutChan,
                                 applicable_declarations: &mut ApplicableDeclarations,
                                 initial_values: &ComputedValues,
                                 parent: Option<LayoutNode>) {
        self.initialize_layout_data((*layout_chan).clone());

        if self.is_element() {
            self.match_node(stylist, applicable_declarations);
        }

        unsafe {
            self.cascade_node(parent, initial_values, applicable_declarations)
        }

        applicable_declarations.clear();

        for kid in self.children() {
            kid.match_and_cascade_subtree(stylist,
                                          layout_chan,
                                          applicable_declarations,
                                          initial_values,
                                          Some(*self))
        }
    }

    unsafe fn cascade_node(&self,
                           parent: Option<LayoutNode>,
                           initial_values: &ComputedValues,
                           applicable_declarations: &ApplicableDeclarations) {
        macro_rules! cascade_node(
            ($applicable_declarations: expr, $style: ident) => {{
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
                                    Some(ref style) => Some(style),
                                }
                            }
                        }
                    }
                 };

                 let computed_values = match parent_style {
                     Some(ref style) => {
                         Arc::new(cascade($applicable_declarations.as_slice(),
                                          Some(style.get()),
                                          initial_values))
                     }
                     None => Arc::new(cascade($applicable_declarations.as_slice(),
                                              None,
                                              initial_values)),
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

        cascade_node!(applicable_declarations.normal, style);
        if applicable_declarations.before.len() > 0 {
            cascade_node!(applicable_declarations.before, before_style);
        }
        if applicable_declarations.after.len() > 0 {
            cascade_node!(applicable_declarations.after, after_style);
        }
    }
}

