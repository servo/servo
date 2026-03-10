/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::collections::VecDeque;
use std::rc::Rc;

use servo::{LoadStatus, Preferences, WebViewBuilder};
use url::Url;

use crate::common::{ServoTest, WebViewDelegateImpl};

struct NoOpChangeHandler;

impl accesskit_consumer::TreeChangeHandler for NoOpChangeHandler {
    fn node_added(&mut self, _: &accesskit_consumer::Node) {}
    fn node_updated(&mut self, _: &accesskit_consumer::Node, _: &accesskit_consumer::Node) {}
    fn focus_moved(
        &mut self,
        _: Option<&accesskit_consumer::Node>,
        _: Option<&accesskit_consumer::Node>,
    ) {
    }
    fn node_removed(&mut self, _: &accesskit_consumer::Node) {}
}

#[test]
fn test_basic_accessibility_update() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.accessibility_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .build();

    webview.set_accessibility_active(true);

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    let tree = build_tree(updates);
    let root_node = tree.state().root();
    find_first_matching_node(root_node, |node| node.role() == accesskit::Role::ScrollView)
        .expect("Tree should include a scroll view corresponding to the WebView.");
}

fn wait_for_min_updates(
    servo_test: &ServoTest,
    delegate: Rc<WebViewDelegateImpl>,
    min_num_updates: usize,
) -> Vec<accesskit::TreeUpdate> {
    let captured_delegate = delegate.clone();
    servo_test.spin(move || {
        captured_delegate.last_accesskit_tree_updates.borrow().len() < min_num_updates
    });

    delegate
        .last_accesskit_tree_updates
        .borrow_mut()
        .drain(..)
        .collect()
}

fn build_tree(tree_updates: Vec<accesskit::TreeUpdate>) -> accesskit_consumer::Tree {
    let first_update = tree_updates[0].clone();
    let tree_id = first_update.tree_id;

    // We need to make a TreeUpdate with a TreeId of ROOT, which can have the subtrees grafted in
    let root_node_id = accesskit::NodeId(0x0);
    let mut root_node = accesskit::Node::new(accesskit::Role::GenericContainer);

    // We need to make a graft node so that we have a non-graft node to set as the initial focused
    // node for the tree.
    let graft_node_id = accesskit::NodeId(0x1);
    let mut graft_node = accesskit::Node::new(accesskit::Role::GenericContainer);
    graft_node.set_tree_id(tree_id);

    root_node.set_children(vec![graft_node_id]);

    let root_tree = accesskit::Tree {
        root: root_node_id,
        toolkit_name: None,
        toolkit_version: None,
    };

    let root_update = accesskit::TreeUpdate {
        nodes: vec![(root_node_id, root_node), (graft_node_id, graft_node)],
        tree: Some(root_tree),
        tree_id: accesskit::TreeId::ROOT,
        focus: root_node_id,
    };

    let mut tree = accesskit_consumer::Tree::new(root_update, true /* is_host_focused */);

    for tree_update in tree_updates {
        tree.update_and_process_changes(tree_update, &mut NoOpChangeHandler);
    }
    tree
}

fn find_first_matching_node(
    root_node: accesskit_consumer::Node<'_>,
    mut pred: impl FnMut(&accesskit_consumer::Node) -> bool,
) -> Option<accesskit_consumer::Node<'_>> {
    let mut children = root_node.children().collect::<VecDeque<_>>();
    let mut result: Option<accesskit_consumer::Node> = None;
    while let Some(candidate) = children.pop_front() {
        if pred(&candidate) {
            result = Some(candidate);
            break;
        }
        for child in candidate.children() {
            children.push_back(child);
        }
    }
    result
}
