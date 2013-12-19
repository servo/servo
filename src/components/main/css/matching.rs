/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// High-level interface to CSS selector matching.

use css::node_style::StyledNode;
use layout::incremental;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;
// FIXME(ksh8281) this is from upstream rust. if rust in servo is upgrade, we should change this
use servo_util::deque;
use servo_util::deque::Data;

use extra::arc::{Arc, RWArc};
use std::cast;
use std::cell::Cell;
use std::comm;
use std::rt;
use std::task;
use style::{TNode, Stylist, cascade};

pub trait MatchMethods {
    fn match_node(&self, stylist: &Stylist);
    fn match_subtree(&self, stylist: RWArc<Stylist>);

    fn cascade_node(&self, parent: Option<LayoutNode>);
    fn cascade_subtree(&self, parent: Option<LayoutNode>);
}

impl<'self> MatchMethods for LayoutNode<'self> {
    fn match_node(&self, stylist: &Stylist) {
        let applicable_declarations = do self.with_element |element| {
            let style_attribute = match *element.style_attribute() {
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
        let num_tasks = rt::default_sched_threads();

        let mut pool = deque::BufferPool::new();
        let mut worker_list = ~[];
        let mut stealer_list = ~[];
        let mut node_list = ~[];

        for _ in range(0,num_tasks) {
            let (worker,stealer) = pool.deque();
            worker_list.push(worker);
            stealer_list.push(stealer);
        }

        for node in self.traverse_preorder() {
            if node.is_element() {
                node_list.push(node);
            }
        }

        let mut worker_count = 0;
        let split_count = 16;
        let mut split_index = 0;
        while split_index < node_list.len() {
            let end_index = if split_index + split_count > node_list.len() {
                node_list.len()
            } else {
                split_index + split_count
            };

            // FIXME(pcwalton): This transmute is to work around the fact that we have no
            // mechanism for safe fork/join parallelism. If we had such a thing, then we could
            // close over the lifetime-bounded `LayoutNode`. But we can't, so we force it with
            // a transmute.
            let evil: (int,int) = unsafe {
                cast::transmute(node_list.slice(split_index, end_index))
            };

            worker_list[worker_count % num_tasks].push(evil);
            worker_count += 1;
            split_index = end_index;
        }

        let (port, chan) = comm::stream();
        let chan = comm::SharedChan::new(chan);

        for _ in range(0, num_tasks) {
            let chan = chan.clone();
            let stylist = stylist.clone();
            let worker_data =
                Cell::new((worker_list.remove(0), stealer_list.clone(), stylist, chan));
            do task::spawn_sched(task::SingleThreaded) {
                    let (mut worker, mut stealer_list, stylist, chan) = worker_data.take();
                    do stylist.read |stylist| {
                        loop {
                            let mut is_processed = false;
                            match worker.pop() {
                                Some(evil) => {
                                    let node_list: &[LayoutNode] = unsafe {
                                        cast::transmute(evil)
                                    };
                                    for node in node_list.iter() {
                                        node.match_node(stylist);
                                    }
                                    is_processed = true;
                                },
                                None => {
                                    for stealer in stealer_list.mut_iter() {
                                        match stealer.steal() {
                                            Data(evil) => {
                                                is_processed = true;
                                                let node_list: &[LayoutNode] = unsafe {
                                                    cast::transmute(evil)
                                                };
                                                for node in node_list.iter() {
                                                    node.match_node(stylist);
                                                }
                                            },
                                            _ => { }
                                        }
                                    }
                                }
                            }

                            if !is_processed {
                                break;
                            }
                        }
                    }
                    chan.send(());
                }
        }
        for _ in range(0, num_tasks) {
            port.recv();
        }
    }

    fn cascade_node(&self, parent: Option<LayoutNode>) {
        let parent_style = match parent {
            Some(ref parent) => Some(parent.style()),
            None => None
        };

        let computed_values = unsafe {
            Arc::new(cascade(self.borrow_layout_data_unchecked()
                                 .as_ref()
                                 .unwrap()
                                 .applicable_declarations,
                             parent_style.map(|parent_style| parent_style.get())))
        };

        match *self.mutate_layout_data().ptr {
            None => fail!("no layout data"),
            Some(ref mut layout_data) => {
                let style = &mut layout_data.style;
                match *style {
                    None => (),
                    Some(ref previous_style) => {
                        layout_data.restyle_damage =
                            Some(incremental::compute_damage(previous_style.get(),
                                                             computed_values.get()).to_int())
                    }
                }
                *style = Some(computed_values)
            }
        }
    }

    fn cascade_subtree(&self, parent: Option<LayoutNode>) {
        self.cascade_node(parent);

        for kid in self.children() {
            if kid.is_element() {
                kid.cascade_subtree(Some(*self));
            }
        }
    }
}
