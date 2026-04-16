/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::collections::VecDeque;
use std::rc::Rc;

use accesskit::{NodeId, Role, TreeId, TreeUpdate};
use accesskit_consumer::TreeChangeHandler;
use servo::{LoadStatus, Preferences, WebViewBuilder};
use url::Url;

use crate::common::{ServoTest, WebViewDelegateImpl};

struct NoOpChangeHandler;

impl TreeChangeHandler for NoOpChangeHandler {
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

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    let tree = build_tree(updates);
    let _ = assert_tree_structure_and_get_root_web_area(&tree);
}

#[test]
fn test_activate_accessibility_after_layout() {
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

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    webview.set_accessibility_active(true);

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    let tree = build_tree(updates);
    let _ = assert_tree_structure_and_get_root_web_area(&tree);
}

#[test]
fn test_navigate_creates_new_accessibility_update() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.accessibility_enabled = true;
        builder.preferences(preferences)
    });

    let page_1_url = Url::parse("data:text/html,<!DOCTYPE html> page 1").unwrap();
    let page_2_url = Url::parse("data:text/html,<!DOCTYPE html> page 2").unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(page_1_url)
        .build();
    webview.set_accessibility_active(true);

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    let mut tree = build_tree(updates);

    let root_web_area = assert_tree_structure_and_get_root_web_area(&tree);

    let result = find_first_matching_node(root_web_area, |node| {
        node.role() == accesskit::Role::TextRun
    });
    let text_node = result.expect("Should be exactly one TextRun in the tree");

    assert_eq!(text_node.value().as_deref(), Some("page 1"));

    let load_webview = webview.clone();
    webview.load(page_2_url.clone());
    servo_test.spin(move || load_webview.url() != Some(page_2_url.clone()));

    let new_updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    for tree_update in new_updates {
        tree.update_and_process_changes(tree_update, &mut NoOpChangeHandler);
    }

    let root_node = tree.state().root();
    let result =
        find_first_matching_node(root_node, |node| node.role() == accesskit::Role::TextRun);
    let text_node = result.expect("Should be exactly one TextRun in the tree");

    assert_eq!(text_node.value().as_deref(), Some("page 2"));
}

// FIXME(accessibility): when clicking back and forward, we currently rely on
// layout and the accessibility tree being rebuilt from scratch, so that the full
// a11y tree can be resent.
// But if bfcache navigations stop redoing layout, or we implement incremental
// a11y tree building, this test will break.
#[test]
fn test_accessibility_after_navigate_and_back() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.accessibility_enabled = true;
        builder.preferences(preferences)
    });

    let page_1_url = Url::parse("data:text/html,<!DOCTYPE html> page 1").unwrap();
    let page_2_url = Url::parse("data:text/html,<!DOCTYPE html> page 2").unwrap();

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(page_1_url.clone())
        .build();
    webview.set_accessibility_active(true);

    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    let mut tree = build_tree(updates);

    let root_web_area = assert_tree_structure_and_get_root_web_area(&tree);

    let result = find_all_matching_nodes(root_web_area, |node| {
        node.role() == accesskit::Role::TextRun
    });
    assert_eq!(result.len(), 1);
    let text_node = result[0];

    assert_eq!(text_node.value().as_deref(), Some("page 1"));

    let load_webview = webview.clone();
    webview.load(page_2_url.clone());
    servo_test.spin(move || load_webview.url() != Some(page_2_url.clone()));

    let new_updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    for tree_update in new_updates {
        tree.update_and_process_changes(tree_update, &mut NoOpChangeHandler);
    }

    let root_node = tree.state().root();
    let result = find_all_matching_nodes(root_node, |node| node.role() == accesskit::Role::TextRun);
    assert_eq!(result.len(), 1);
    let text_node = result[0];

    assert_eq!(text_node.value().as_deref(), Some("page 2"));

    let back_webview = webview.clone();
    webview.go_back(1);
    servo_test.spin(move || back_webview.url() != Some(page_1_url.clone()));

    let new_updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    for tree_update in new_updates {
        tree.update_and_process_changes(tree_update, &mut NoOpChangeHandler);
    }

    let root_node = tree.state().root();
    let result = find_all_matching_nodes(root_node, |node| node.role() == accesskit::Role::TextRun);
    assert_eq!(result.len(), 1);
    let text_node = result[0];

    assert_eq!(text_node.value().as_deref(), Some("page 1"));
}

// TODO(accessibility): write test for resend a11y tree when clicking back or forward

fn wait_for_min_updates(
    servo_test: &ServoTest,
    delegate: Rc<WebViewDelegateImpl>,
    min_num_updates: usize,
) -> Vec<TreeUpdate> {
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

fn build_tree(tree_updates: Vec<TreeUpdate>) -> accesskit_consumer::Tree {
    let first_update = tree_updates[0].clone();
    let tree_id = first_update.tree_id;

    // We need to make a TreeUpdate with a TreeId of ROOT, which can have the subtrees grafted in
    let root_node_id = NodeId(0x0);
    let mut root_node = accesskit::Node::new(Role::GenericContainer);

    // We need to make a graft node so that we have a non-graft node to set as the initial focused
    // node for the tree.
    let graft_node_id = NodeId(0x1);
    let mut graft_node = accesskit::Node::new(Role::GenericContainer);
    graft_node.set_tree_id(tree_id);

    root_node.set_children(vec![graft_node_id]);

    let root_tree = accesskit::Tree {
        root: root_node_id,
        toolkit_name: None,
        toolkit_version: None,
    };

    let root_update = TreeUpdate {
        nodes: vec![(root_node_id, root_node), (graft_node_id, graft_node)],
        tree: Some(root_tree),
        tree_id: TreeId::ROOT,
        focus: root_node_id,
    };

    let mut tree = accesskit_consumer::Tree::new(root_update, true /* is_host_focused */);

    for tree_update in tree_updates {
        tree.update_and_process_changes(tree_update, &mut NoOpChangeHandler);
    }
    tree
}

fn assert_tree_structure_and_get_root_web_area<'tree>(
    tree: &'tree accesskit_consumer::Tree,
) -> accesskit_consumer::Node<'tree> {
    let root_node = tree.state().root();
    let scroll_view = find_first_matching_node(root_node, |node| node.role() == Role::ScrollView)
        .expect("Tree should include a scroll view corresponding to the WebView.");
    let scroll_view_children: Vec<accesskit_consumer::Node<'_>> = scroll_view.children().collect();
    assert_eq!(scroll_view_children.len(), 1);
    let graft_node = scroll_view_children[0];
    assert!(graft_node.is_graft());

    let graft_node_children: Vec<accesskit_consumer::Node<'_>> = graft_node.children().collect();
    assert_eq!(graft_node_children.len(), 1);

    let root_web_area = graft_node_children[0];
    assert_eq!(root_web_area.role(), Role::RootWebArea);

    root_web_area
}

fn find_first_matching_node(
    root_node: accesskit_consumer::Node<'_>,
    mut pred: impl FnMut(&accesskit_consumer::Node) -> bool,
) -> Option<accesskit_consumer::Node<'_>> {
    let mut children = root_node.children().collect::<VecDeque<_>>();
    while let Some(candidate) = children.pop_front() {
        if pred(&candidate) {
            return Some(candidate);
        }
        for child in candidate.children() {
            children.push_back(child);
        }
    }
    None
}

fn find_all_matching_nodes(
    root_node: accesskit_consumer::Node<'_>,
    mut pred: impl FnMut(&accesskit_consumer::Node) -> bool,
) -> Vec<accesskit_consumer::Node<'_>> {
    let mut children = root_node.children().collect::<VecDeque<_>>();
    let mut result = vec![];
    while let Some(candidate) = children.pop_front() {
        if pred(&candidate) {
            result.push(candidate);
        }
        for child in candidate.children() {
            children.push_back(child);
        }
    }
    result
}
