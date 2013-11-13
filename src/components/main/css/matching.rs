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
        let (applicable_declarations, pseudo_applicable_declarations) = do self.with_imm_element |element| {
            let style_attribute = match element.style_attribute {
                None => None,
                Some(ref style_attribute) => Some(style_attribute)
            };
            stylist.get_applicable_declarations(self, style_attribute)
        };
        let cell = Cell::new(applicable_declarations);
        do self.write_layout_data |data| {
            data.applicable_declarations = cell.take();
        }
        if pseudo_applicable_declarations.len() > 0 {
            let pseudo_cell = Cell::new(pseudo_applicable_declarations);
            do self.write_layout_data |data| {
                data.pseudo_applicable_declarations = pseudo_cell.take();
            }
        }
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
        let computed_values = do self.write_layout_data |data| {
            let computed_values = cascade(data.applicable_declarations, parent_style);
            if data.pseudo_applicable_declarations.len() > 0 {
                let pseudo_computed_values = cascade(data.pseudo_applicable_declarations, Some(&computed_values));
                let cell = Cell::new(pseudo_computed_values);
                data.pseudo_style = Some(cell.take());
            }
            computed_values
        };
        let cell = Cell::new(computed_values);
        do self.write_layout_data |data| {
            let style = cell.take();
            // If there was an existing style, compute the damage that
            // incremental layout will need to fix.
            match data.style {
                None => (),
                Some(ref previous_style) => self.set_restyle_damage(
                    incremental::compute_damage(previous_style, &style))
            }
            data.style = Some(style);
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
