/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use std::cell::Cell;
use std::comm;
use std::rt;
use std::task;
use std::vec;
use extra::arc::RWArc;

use css::node_style::StyledNode;
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

        match *self.mutate_layout_data().ptr {
            Some(ref mut layout_data) => {
                layout_data.applicable_declarations = applicable_declarations
            }
            None => fail!("no layout data")
        }
    }
    fn match_subtree(&self, stylist: RWArc<Stylist>) {
        // FIXME(pcwalton): Racy. Parallel CSS selector matching is disabled.
        //let num_tasks = rt::default_sched_threads() * 2;
        let num_tasks = 1;
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

        let computed_values = unsafe {
            cascade(self.borrow_layout_data_unchecked().as_ref().unwrap().applicable_declarations,
                    parent_style)
        };

        match *self.mutate_layout_data().ptr {
            None => fail!("no layout data"),
            Some(ref mut layout_data) => {
                let style = &mut layout_data.style;
                match *style {
                    None => (),
                    Some(ref previous_style) => {
                        layout_data.restyle_damage =
                            Some(incremental::compute_damage(previous_style,
                                                             &computed_values).to_int())
                    }
                }
                *style = Some(computed_values)
            }
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
