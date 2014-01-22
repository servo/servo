/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::incremental;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;
use servo_util::task::spawn_named;

use extra::arc::{Arc, RWArc};
use std::cast;
use std::libc::uintptr_t;
use std::rt;
use std::vec;
use style::{TNode, Stylist, cascade};
use style::{Before, After};
use servo_net::history_task::HistoryTask;

pub trait MatchMethods {
    fn match_node(&self, stylist: &Stylist, history_task: &HistoryTask);
    fn match_subtree(&self, stylist: RWArc<Stylist>, history_task: HistoryTask);

    fn cascade_subtree(&self, parent: Option<LayoutNode>);
}

impl<'ln> MatchMethods for LayoutNode<'ln> {
    fn match_node(&self, stylist: &Stylist, history_task: &HistoryTask) {
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
                    self, style_attribute, None, history_task);
                layout_data.data.before_applicable_declarations = stylist.get_applicable_declarations(
                    self, None, Some(Before), history_task);
                layout_data.data.after_applicable_declarations = stylist.get_applicable_declarations(
                    self, None, Some(After), history_task);
            }
            None => fail!("no layout data")
        }
    }
    fn match_subtree(&self, stylist: RWArc<Stylist>, history_task: HistoryTask) {
        let num_tasks = rt::default_sched_threads() * 2;
        let mut node_count = 0;
        let mut nodes_per_task = vec::from_elem(num_tasks, ~[]);
        for node in self.traverse_preorder() {
            if node.is_element() {
                nodes_per_task[node_count % num_tasks].push(node);
                node_count += 1;
            }
        }

        let (port, chan) = SharedChan::new();
        let mut num_spawned = 0;

        for nodes in nodes_per_task.move_iter() {
            if nodes.len() > 0 {
                let chan = chan.clone();
                let stylist = stylist.clone();
                let history_task = history_task.clone();
                // FIXME(pcwalton): This transmute is to work around the fact that we have no
                // mechanism for safe fork/join parallelism. If we had such a thing, then we could
                // close over the lifetime-bounded `LayoutNode`. But we can't, so we force it with
                // a transmute.
                let evil: uintptr_t = unsafe {
                    cast::transmute(nodes)
                };

                let evil = Some(evil);
                spawn_named("MatchMethods for LayoutNode", proc() {
                    let mut evil = evil;
                    let nodes: ~[LayoutNode] = unsafe {
                        cast::transmute(evil.take_unwrap())
                    };
                    stylist.read(|stylist| {
                        for node in nodes.iter() {
                            node.match_node(stylist, &history_task);
                        }
                    });
                    chan.send(());
                });
                num_spawned += 1;
            }
        }
        for _ in range(0, num_spawned) {
            port.recv();
        }
    }

    fn cascade_subtree(&self, parent: Option<LayoutNode>) {
        macro_rules! cascade_node(
            ($applicable_declarations: ident, $style: ident) => {{
                let parent_style = match parent {
                    Some(ref parent) => Some(parent.style()),
                    None => None
                };

                let computed_values = {
                    let layout_data_ref = self.borrow_layout_data();
                    let layout_data = layout_data_ref.get().as_ref().unwrap();
                    Arc::new(cascade(layout_data.data.$applicable_declarations, parent_style.map(|parent_style| parent_style.get())))
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

        for kid in self.children() {
            if kid.is_element() {
                kid.cascade_subtree(Some(*self));
            }
        }
    }
}
