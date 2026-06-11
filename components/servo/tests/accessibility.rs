/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::collections::VecDeque;
use std::rc::Rc;

use accesskit::{NodeId, Role, TreeId, TreeUpdate};
use accesskit_consumer::TreeChangeHandler;
use servo::{DiagnosticsLoggingOption, LoadStatus, Opts, Preferences, WebViewBuilder};
use url::Url;

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

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
    let servo_test = build_test();

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
    let servo_test = build_test();

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
    let servo_test = build_test();

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
    let servo_test = build_test();

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

#[test]
fn test_accessibility_basic_mapping() {
    let mut element_role_pairs = VecDeque::from([
        ("article", Role::Article),
        ("aside", Role::Complementary),
        ("footer", Role::ContentInfo),
        ("h1", Role::Heading),
        ("h2", Role::Heading),
        ("h3", Role::Heading),
        ("h4", Role::Heading),
        ("h5", Role::Heading),
        ("h6", Role::Heading),
        ("header", Role::Banner),
        ("hr", Role::Splitter),
        ("main", Role::Main),
        ("nav", Role::Navigation),
        ("p", Role::Paragraph),
    ]);

    let mut url: String = "data:text/html,<!DOCTYPE html>".to_owned();
    for (element, _) in element_role_pairs.iter() {
        url.push_str(format!("<{element}></{element}>").as_str());
    }

    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url.as_str());

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    assert_eq!(root.children().len(), element_role_pairs.len());
    for child in root.children() {
        let Some((tag, role)) = element_role_pairs.pop_front() else {
            panic!("Number of children of root node should match number of tag/role pairs");
        };
        assert_eq!(child.data().html_tag(), Some(tag));
        assert_eq!(child.role(), role);
    }
    assert!(
        element_role_pairs.is_empty(),
        "Number of children of root node should match number of tag/role pairs"
    );
}

#[test]
fn test_accessibility_basic_name_from_contents() {
    let url = "data:text/html,<!DOCTYPE html><h1>Servo</h1>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let first_child = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_eq!(first_child.role(), Role::Heading);
    assert_eq!(first_child.label(), Some("Servo".to_owned()));
}

#[test]
fn test_accessibility_name_from_contents_subtree() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h1>Servo aims to empower <code>developers</code> with a <em>lightweight</em>, \
               <strong>high-performance</strong> alternative for <span>embedding \
               <span>web technologies</span> in <span>applications</span></span>.</h1>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let heading = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_eq!(heading.role(), Role::Heading);
    let heading_children: Vec<accesskit_consumer::Node> = heading.children().collect();
    assert_eq!(heading_children.len(), 9);
    assert_eq!(
        heading.label(),
        Some(
            "Servo aims to empower developers with a lightweight, high-performance alternative for \
             embedding web technologies in applications."
                .to_owned()
        ),
        "Heading label should be composed of the text contents of all of its descendant text nodes"
    );
}

#[test]
fn test_accessibility_basic_mutation() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h1 id='h1'>This is an h1</h1>\
               <h2 id='h2'>This is an h2</h2>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<_> = root.children().collect();
    assert_eq!(children.len(), 2);
    let h1 = children[0];
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));
    let h2 = children[1];
    assert_eq!(h2.label(), Some("This is an h2".to_owned()));

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.getElementById('h2').remove();\
         window.ServoTestUtils.forceAccessibilityUpdate();",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    assert_eq!(update.nodes.len(), 1);
    assert_eq!(update.nodes[0].1.role(), Role::RootWebArea);
    assert_eq!(update.nodes[0].1.children().len(), 1);
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<_> = root.children().collect();
    assert_eq!(children.len(), 1);
    let h1 = children[0];
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));
}

#[test]
fn test_accessibility_with_mutation_move_nodes() {
    let url = "data:text/html,<!DOCTYPE html>\
               <div id='div1'></div>\
               <h1 id='h1'>This is an h1</h1>\
               <h2 id='h2'>This is an h2</h2>\
               <div id='div2'></div>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 4);
    let div1 = children[0];
    let h1 = children[1];
    let h2 = children[2];
    let div2 = children[3];
    assert_eq!(div1.role(), Role::GenericContainer);
    assert_eq!(h1.role(), Role::Heading);
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));
    assert_eq!(h2.role(), Role::Heading);
    assert_eq!(h2.label(), Some("This is an h2".to_owned()));
    assert_eq!(div2.role(), Role::GenericContainer);

    // use both moveBefore and appendChild to exercise the different ways nodes move.
    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "div1.moveBefore(h1,null); div2.appendChild(h2);\
         window.ServoTestUtils.forceAccessibilityUpdate();",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    assert_eq!(update.nodes.len(), 3);
    assert_eq!(update.nodes[0].1.role(), Role::GenericContainer);
    assert_eq!(update.nodes[0].1.children().len(), 1);
    assert_eq!(update.nodes[1].1.role(), Role::GenericContainer);
    assert_eq!(update.nodes[1].1.children().len(), 1);
    assert_eq!(update.nodes[2].1.role(), Role::RootWebArea);
    assert_eq!(update.nodes[2].1.children().len(), 2);
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    assert_eq!(root.children().count(), 2);
    let div1 = root.children().nth(0).unwrap();
    let div2 = root.children().nth(1).unwrap();
    assert_eq!(div1.children().count(), 1);
    assert_eq!(div2.children().count(), 1);
    let h1 = div1.children().nth(0).unwrap();
    let h2 = div2.children().nth(0).unwrap();
    assert_eq!(h1.role(), Role::Heading);
    assert_eq!(h1.label().as_deref(), Some("This is an h1"));
    assert_eq!(h2.role(), Role::Heading);
    assert_eq!(h2.label().as_deref(), Some("This is an h2"));
}

#[test]
fn test_accessibility_text_change() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h1 id='h1'>This is an h1</h1>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 1);
    let h1 = children[0];
    assert_eq!(h1.role(), Role::Heading);
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "h1.firstChild.appendData(', now with more text');\
         window.ServoTestUtils.forceAccessibilityUpdate();",
    );
    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    assert_eq!(update.nodes.len(), 2);
    assert_eq!(update.nodes[0].1.role(), Role::TextRun);
    assert_eq!(update.nodes[1].1.role(), Role::Heading);
    assert_eq!(update.nodes[1].1.children().len(), 1);
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 1);
    let h1 = children[0];
    assert_eq!(h1.role(), Role::Heading);
    assert_eq!(
        h1.label(),
        Some("This is an h1, now with more text".to_owned())
    );
}

fn build_test() -> ServoTest {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.accessibility_enabled = true;
        preferences.dom_servo_helpers_enabled = true;
        preferences.expensive_accessibility_test_assertions_enabled = true;
        let mut opts = Opts::default();
        opts.debug
            .toggle_option(DiagnosticsLoggingOption::AccessibilityTree, true);
        builder.preferences(preferences).opts(opts)
    });
    servo_test
}

fn build_webview_and_tree(
    url: &str,
) -> (
    ServoTest,
    Rc<WebViewDelegateImpl>,
    servo::WebView,
    accesskit_consumer::Tree,
) {
    let servo_test = build_test();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(Url::parse(url).unwrap())
        .build();
    webview.set_accessibility_active(true);
    let load_webview = webview.clone();
    servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 2);
    let tree = build_tree(updates);
    (servo_test, delegate, webview, tree)
}

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

    find_first_matching_node(graft_node, |node| node.role() == Role::RootWebArea)
        .expect("Should have a RootWebArea")
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
