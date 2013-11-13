/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use std::cell::Cell;
use std::comm;
use std::rt::default_sched_threads;
use std::task;
use std::vec;
use extra::arc::RWArc;

use css::node_style::StyledNode;
use css::node_util::NodeUtil;
use layout::incremental;
use layout::util::LayoutDataAccess;

use script::dom::node::{AbstractNode, LayoutView};
use style::Stylist;
use style::cascade;
use servo_util::tree::TreeNodeRef;

pub trait MatchMethods {
    fn match_node(&self, stylist: &Stylist);
    fn match_subtree(&self, stylist: RWArc<Stylist>);

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
        self.layout_data().applicable_declarations.set(applicable_declarations)
    }
    fn match_subtree(&self, stylist: RWArc<Stylist>) {
        let num_tasks = default_sched_threads() * 2;
        let mut node_count = 0;
        let mut nodes_per_task = vec::from_elem(num_tasks, ~[]);

        for node in self.traverse_preorder() {
            if node.is_element() {
                nodes_per_task[node_count % num_tasks].push(node);
                node_count += 1;
            }
        }

        let (port, chan) = comm::stream();
        let chan = comm::SharedChan::new(chan);
        let mut num_spawned = 0;

        for nodes in nodes_per_task.move_iter() {
            if nodes.len() > 0 {
                let chan = chan.clone();
                let stylist = stylist.clone();
                do task::spawn_with((nodes, stylist)) |(nodes, stylist)| {
                    let nodes = Cell::new(nodes);
                    do stylist.read |stylist| {
                        for node in nodes.take().move_iter() {
                            node.match_node(stylist);
                        }
                    }
                    chan.send(());
                }
                num_spawned += 1;
            }
        }
        for _ in range(0, num_spawned) {
            port.recv();
        }
    }

    fn cascade_node(&self, parent: Option<AbstractNode<LayoutView>>) {
        let parent_style = match parent {
            Some(parent) => Some(parent.style()),
            None => None
        };

        let layout_data = self.layout_data();
        let computed_values = cascade(*layout_data.applicable_declarations.borrow().ptr,
                                      parent_style);
        let style = layout_data.style.mutate();
        match *style.ptr {
            None => (),
            Some(ref previous_style) => {
                self.set_restyle_damage(incremental::compute_damage(previous_style,
                                                                    &computed_values))
            }
        }
        *style.ptr = Some(computed_values)
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
