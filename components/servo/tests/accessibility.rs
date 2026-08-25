/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView API unit tests.
mod common;

use std::collections::VecDeque;
use std::rc::Rc;

use accesskit::{NodeId, Rect, Role, TreeId, TreeUpdate};
use accesskit_consumer::TreeChangeHandler;
use euclid::Scale;
use servo::{
    DiagnosticsLoggingOption, LoadStatus, Opts, Preferences, Scroll, WebViewBuilder, WebViewPoint,
    WebViewVector,
};
use url::Url;
use webrender_api::units::{DevicePoint, DeviceVector2D};

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
fn test_accessibility_basic_role_from_attribute() {
    let url = "data:text/html,<!DOCTYPE html><div role='blockquote'></div>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let first_child = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_eq!(first_child.role(), Role::Blockquote);
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
        "document.getElementById('h2').remove();",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    // The `<html>` element is also re-sent, because removing the `<h2>` made the document shorter
    // and therefore changed its bounds.
    assert_eq!(update.nodes.len(), 2);
    let root_web_area = find_node_with_role(&update, Role::RootWebArea);
    assert_eq!(root_web_area.children().len(), 1);
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
        "div1.moveBefore(h1,null); div2.appendChild(h2);",
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
        "h1.firstChild.appendData(', now with more text');",
    );
    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    // Appending text always re-sends the two nodes whose contents changed
    let _ = find_node_with_role(&update, Role::TextRun);
    let heading = find_node_with_role(&update, Role::Heading);
    assert_eq!(heading.children().len(), 1);
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

#[test]
fn test_accessibility_role_mutations() {
    let url = "data:text/html,<!DOCTYPE html>\
        <div id='div1' role='emphasis'>Servo</div>\
        <aside id='aside'>Aside</aside>\
        <main id='main' role='status'>Main</main>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 3);
    let div = children[0];
    assert_eq!(
        div.role(),
        Role::Emphasis,
        "<div> should respect author's override"
    );
    let aside = children[1];
    assert_eq!(
        aside.role(),
        Role::Complementary,
        "<aside> should default to implicit ARIA role"
    );
    let main = children[2];
    assert_eq!(
        main.role(),
        Role::Status,
        "<main> should respect author's override"
    );

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "\
            div1.role = 'blockquote';\
            aside.role = 'presentation';\
            main.role = null;\
        ",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    assert_eq!(update.nodes.len(), 3);
    assert_eq!(
        update.nodes[0].1.role(),
        Role::Main,
        "<main> should fallback to implicit role when explicit role is removed"
    );
    assert_eq!(
        update.nodes[1].1.role(),
        Role::Blockquote,
        "<div> should have the new explicit role"
    );
    assert_eq!(
        update.nodes[2].1.role(),
        Role::GenericContainer,
        "<aside> should use the explicit role instead of the implicit role"
    );
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 3);
    assert_eq!(children[0].role(), Role::Blockquote);
    assert_eq!(children[1].role(), Role::GenericContainer);
    assert_eq!(children[2].role(), Role::Main);
}

#[test]
fn test_accessibility_partial_subtree_move_and_delete() {
    let url = "data:text/html,<!DOCTYPE html>\
               <header id='header'>\
                 <div id='div'>\
                   <h1 id='h1'>This is an h1</h1>\
                   <p id='p'>This is a paragraph</p>\
                 </div>\
               </header>\
               <article id='article'><h2 id='h2'>This is an h2</h2></article>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 2);
    let header = children[0];
    assert_eq!(header.role(), Role::Banner);
    let h1 = find_all_matching_nodes(header, |node| node.role() == Role::Heading)[0];
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));
    let _p = find_all_matching_nodes(header, |node| node.role() == Role::Paragraph)
        .pop()
        .expect("Should be exactly one Paragraph node");

    let article = children[1];
    assert_eq!(article.role(), Role::Article);
    let h2 = find_all_matching_nodes(article, |node| node.role() == Role::Heading)[0];
    assert_eq!(h2.label(), Some("This is an h2".to_owned()));

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "article.moveBefore(div, h2);\
         article.moveBefore(h1, div);\
         p.remove();\
         header.remove();",
    );
    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 1);
    let article = children[0];
    assert_eq!(article.role(), Role::Article);
    let children: Vec<accesskit_consumer::Node> = article.children().collect();
    assert_eq!(children.len(), 3);
    let h1 = children[0];
    assert_eq!(h1.role(), Role::Heading);
    assert_eq!(h1.label(), Some("This is an h1".to_owned()));
    let div = children[1];
    assert_eq!(div.role(), Role::GenericContainer);
    let h2 = children[2];
    assert_eq!(h2.role(), Role::Heading);
    assert_eq!(h2.label(), Some("This is an h2".to_owned()));
}

#[test]
fn test_accessibility_children_of_heading_change() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h2>Activating accessibility for a <code>WebView</code></h2>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 1);
    let heading = children[0];
    assert_eq!(heading.role(), Role::Heading);
    assert_eq!(
        heading.label(),
        Some("Activating accessibility for a WebView".to_owned())
    );
    let heading_children: Vec<_> = heading.children().collect();
    assert_eq!(heading_children.len(), 2);
    assert_eq!(heading_children[0].role(), Role::TextRun);
    assert_eq!(heading_children[1].role(), Role::GenericContainer);

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.querySelector('code').remove();",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let heading = find_first_matching_node(root, |node| node.role() == Role::Heading)
        .expect("Heading should still be in the tree");
    assert_eq!(
        heading.label(),
        Some("Activating accessibility for a".to_owned())
    );
}

#[test]
fn test_accessibility_descendants_of_heading_change() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h1>We really <em>really <strong>really</strong></em> like owls</h1>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 1);
    let heading = children[0];
    assert_eq!(heading.role(), Role::Heading);
    assert_eq!(
        heading.label(),
        Some("We really really really like owls".to_owned())
    );
    let heading_children: Vec<_> = heading.children().collect();
    assert_eq!(heading_children.len(), 3);
    assert_eq!(heading_children[0].role(), Role::TextRun);
    assert_eq!(heading_children[1].role(), Role::GenericContainer);
    assert_eq!(heading_children[2].role(), Role::TextRun);

    let em = heading_children[1];
    assert_eq!(em.children().collect::<Vec<_>>().len(), 2);

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.querySelector('strong').remove();",
    );

    let mut updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    assert_eq!(updates.len(), 1);
    let update = updates.pop().expect("Guaranteed by assert above");
    tree.update_and_process_changes(update, &mut NoOpChangeHandler);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let heading = find_first_matching_node(root, |node| node.role() == Role::Heading)
        .expect("Heading should still be in the tree");
    assert_eq!(
        heading.label(),
        Some("We really really  like owls".to_owned())
    );
}

#[test]
fn test_accessibility_bounds() {
    let url = "data:text/html,<!DOCTYPE html>\
               <div id='box' style='position:absolute;left:10px;top:20px;\
               width:100px;height:50px'></div>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);

    let scroll_view = find_first_matching_node(tree.state().root(), |node| {
        node.role() == Role::ScrollView
    })
    .expect(
        "Tree should include a scroll view corresponding to the WebView. (covers whole viewport)",
    );
    assert_rect_eq(
        scroll_view
            .raw_bounds()
            .expect("WebView node should have bounds"),
        Rect::new(0.0, 0.0, TEST_VIEWPORT_SIZE, TEST_VIEWPORT_SIZE),
    );

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let div = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_eq!(div.data().html_tag(), Some("div"));

    // No transforms between div and container; bounds are viewport-relative CSS pixels
    let expected = Rect::new(10.0, 20.0, 110.0, 70.0);
    assert_rect_eq(div.raw_bounds().expect("div should have bounds"), expected);
    assert_rect_eq(
        div.bounding_box().expect("div should have a bounding box"),
        expected,
    );
}

#[test]
fn test_accessibility_webview_bounds_updated_after_hidpi_change() {
    let (servo_test, delegate, webview, mut tree) =
        build_webview_and_tree("data:text/html,<!DOCTYPE html>");

    webview.set_hidpi_scale_factor(Scale::new(2.0));

    for update in wait_for_min_updates(&servo_test, delegate, 1) {
        tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    }

    let scroll_view =
        find_first_matching_node(tree.state().root(), |node| node.role() == Role::ScrollView)
            .expect("Tree should include a scroll view corresponding to the WebView");
    assert_rect_eq(
        scroll_view
            .raw_bounds()
            .expect("WebView node should have bounds"),
        Rect::new(0.0, 0.0, TEST_VIEWPORT_SIZE / 2.0, TEST_VIEWPORT_SIZE / 2.0),
    );
}

#[test]
fn test_accessibility_text_run_bounds() {
    let url = "data:text/html,<!DOCTYPE html>\
               <h1 style='position:absolute;left:0;top:0;margin:0;\
               width:300px;height:40px'>Servo</h1>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let heading = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_eq!(heading.role(), Role::Heading);
    assert_rect_eq(
        heading.raw_bounds().expect("heading should have bounds"),
        Rect::new(0.0, 0.0, 300.0, 40.0),
    );

    // TODO(accessibility): Text nodes have no layout box of their own, so no bounds are computed
    // for them yet. They should eventually get the union of the rectangles of their own
    // `Fragment::Text` fragments. This assertion demonstrates the current behaviour; flip it once
    // text run bounds are implemented. See #47164.
    let text_run = find_first_matching_node(heading, |node| node.role() == Role::TextRun)
        .expect("Heading should contain a TextRun");
    assert!(
        text_run.raw_bounds().is_none(),
        "TextRun bounds are not implemented yet, but got {:?}",
        text_run.raw_bounds()
    );
}

#[test]
fn test_accessibility_bounds_omitted_for_display_contents() {
    let url = "data:text/html,<!DOCTYPE html>\
               <div style='position:absolute;left:0;top:0;width:200px;height:30px'>\
               <span style='display:contents'>hello</span></div>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let div = root
        .children()
        .next()
        .expect("Root web area should have at least one child.");
    assert_rect_eq(
        div.raw_bounds().expect("div should have bounds"),
        Rect::new(0.0, 0.0, 200.0, 30.0),
    );

    // TODO(accessibility): A `display: contents` element generates no box, so no bounds are
    // computed for it yet. Other engines (Blink, WebKit, Gecko) compute its bounds as the union of
    // the bounding boxes of its rendered descendants. This test demonstrates the current
    // behaviour; flip these assertions once that is implemented. See #47163.
    let span = find_first_matching_node(root, |node| node.data().html_tag() == Some("span"))
        .expect("Document should contain the `display: contents` span");
    assert!(
        span.raw_bounds().is_none(),
        "`display: contents` bounds are not implemented yet, but got {:?}",
        span.raw_bounds()
    );
    let text_run = find_first_matching_node(span, |node| node.role() == Role::TextRun)
        .expect("The `display: contents` span should contain a TextRun");
    assert!(
        text_run.raw_bounds().is_none(),
        "TextRun bounds are not implemented yet, but got {:?}",
        text_run.raw_bounds()
    );
}

#[test]
fn test_accessibility_bounds_omitted_for_display_none() {
    // display: none: no geometry, text must not inherit ancestor bounds
    let url = "data:text/html,<!DOCTYPE html>\
               <div style='position:absolute;left:0;top:0;width:200px;height:30px'>\
               <span id='hidden' style='display:none'>hidden</span></div>";
    let (_servo_test, _delegate, _webview, tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    for node in find_all_matching_nodes(root, |node| {
        node.data().html_tag() == Some("span") || node.role() == Role::TextRun
    }) {
        assert!(
            node.raw_bounds().is_none(),
            "`display: none` content should have no bounds, but {:?} had {:?}",
            node.role(),
            node.raw_bounds()
        );
    }
}

#[test]
fn test_accessibility_bounds_updated_after_relayout() {
    let url = "data:text/html,<!DOCTYPE html>\
               <div id='box' style='position:absolute;left:10px;top:20px;\
               width:100px;height:50px'></div>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let div = root.children().next().expect("Should have a div child");
    let div_id = div.locate().0; // Maps to layout's NodeId
    assert_rect_eq(
        div.raw_bounds().expect("div should have bounds"),
        Rect::new(10.0, 20.0, 110.0, 70.0),
    );

    // Relayout with geometry changes auto-refreshes accessibility tree
    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "const box = document.getElementById('box');\
         box.style.left = '30px'; box.style.width = '200px';",
    );

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    let expected = Rect::new(30.0, 20.0, 230.0, 70.0);
    let updated_bounds = updates
        .iter()
        .flat_map(|update| update.nodes.iter())
        .filter(|(id, _)| *id == div_id)
        .filter_map(|(_, node)| node.bounds())
        .next_back()
        .expect("The div should have been re-sent with new bounds");
    assert_rect_eq(updated_bounds, expected);

    for update in updates {
        tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    }
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let div = root.children().next().expect("Should have a div child");
    assert_rect_eq(div.raw_bounds().expect("div should have bounds"), expected);
}

#[test]
fn test_accessibility_bounds_updated_after_renderer_scroll() {
    let url = "data:text/html,<!DOCTYPE html>\
               <main style='position:absolute;left:10px;top:100px;\
               width:100px;height:50px'>Target</main>\
               <div style='width:2000px;height:2000px'></div>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let main = find_first_matching_node(root, |node| node.role() == Role::Main)
        .expect("Document should contain a main element");
    let main_id = main.locate().0; // Maps to layout's NodeId
    assert_rect_eq(
        main.raw_bounds().expect("main should have bounds"),
        Rect::new(10.0, 100.0, 110.0, 150.0),
    );

    // A positive delta reveals more content at the bottom and right, so this is equivalent to
    // `window.scrollTo(20, 40)`.
    webview.notify_scroll_event(
        Scroll::Delta(WebViewVector::Device(DeviceVector2D::new(20.0, 40.0))),
        WebViewPoint::Device(DevicePoint::new(250.0, 250.0)),
    );

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    let expected = Rect::new(-10.0, 60.0, 90.0, 110.0);
    let updated_bounds = updates
        .iter()
        .flat_map(|update| update.nodes.iter())
        .filter(|(id, _)| *id == main_id)
        .filter_map(|(_, node)| node.bounds())
        .next_back()
        .expect("The main element should have been re-sent with new bounds");
    assert_rect_eq(updated_bounds, expected);

    for update in updates {
        tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    }
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let main = find_first_matching_node(root, |node| node.role() == Role::Main)
        .expect("Document should contain a main element");
    assert_rect_eq(
        main.raw_bounds().expect("main should have bounds"),
        expected,
    );
}

#[test]
fn test_accessibility_bounds_updated_after_script_scroll() {
    let url = "data:text/html,<!DOCTYPE html>\
               <main style='position:absolute;left:10px;top:100px;\
               width:100px;height:50px'>Target</main>\
               <div style='width:2000px;height:2000px'></div>";
    let (servo_test, delegate, webview, mut tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let main = find_first_matching_node(root, |node| node.role() == Role::Main)
        .expect("Document should contain a main element");
    let main_id = main.locate().0; // Maps to layout's NodeId
    assert_rect_eq(
        main.raw_bounds().expect("main should have bounds"),
        Rect::new(10.0, 100.0, 110.0, 150.0),
    );

    // Scrolling the viewport down and to the right shifts every viewport-relative bound up and to
    // the left by the same amount, mirroring `..._after_renderer_scroll` but driven from script.
    let _ = evaluate_javascript(&servo_test, webview.clone(), "window.scrollTo(20, 40);");

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    let expected = Rect::new(-10.0, 60.0, 90.0, 110.0);
    let updated_bounds = updates
        .iter()
        .flat_map(|update| update.nodes.iter())
        .filter(|(id, _)| *id == main_id)
        .filter_map(|(_, node)| node.bounds())
        .next_back()
        .expect("The main element should have been re-sent with new bounds");
    assert_rect_eq(updated_bounds, expected);

    for update in updates {
        tree.update_and_process_changes(update, &mut NoOpChangeHandler);
    }
    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let main = find_first_matching_node(root, |node| node.role() == Role::Main)
        .expect("Document should contain a main element");
    assert_rect_eq(
        main.raw_bounds().expect("main should have bounds"),
        expected,
    );
}

#[test]
fn test_accessibility_unchanged_bounds_are_not_resent() {
    // Absolutely positioned divs; resizing one doesn't affect the other
    let url = "data:text/html,<!DOCTYPE html>\
               <div id='a' style='position:absolute;left:0;top:0;width:10px;height:10px'></div>\
               <div id='b' style='position:absolute;left:100px;top:100px;\
               width:10px;height:10px'></div>";
    let (servo_test, delegate, webview, tree) = build_webview_and_tree(url);

    let root = assert_tree_structure_and_get_root_web_area(&tree);
    let children: Vec<accesskit_consumer::Node> = root.children().collect();
    assert_eq!(children.len(), 2);
    let (node_a, node_b) = (children[0], children[1]);
    assert_rect_eq(
        node_a.raw_bounds().expect("a should have bounds"),
        Rect::new(0.0, 0.0, 10.0, 10.0),
    );
    assert_rect_eq(
        node_b.raw_bounds().expect("b should have bounds"),
        Rect::new(100.0, 100.0, 110.0, 110.0),
    );
    let node_b_id = node_b.locate().0;

    let _ = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.getElementById('a').style.width = '50px';",
    );

    let updates = wait_for_min_updates(&servo_test, delegate.clone(), 1);
    let resent_ids: Vec<NodeId> = updates
        .iter()
        .flat_map(|update| update.nodes.iter())
        .map(|(id, _)| *id)
        .collect();
    assert!(
        !resent_ids.contains(&node_b_id),
        "A node whose bounds did not change should not be re-serialized, but got {resent_ids:?}"
    );
}

// ************************************************************************************************
// If you're adding a new test here, consider adding a matching test in
// tests/wpt/mozilla/tests/accessibility-tree/
// ************************************************************************************************

/// Rendering context size in device pixels (HiDPI scale = 1.0 in tests, so also CSS pixels).
const TEST_VIEWPORT_SIZE: f64 = 500.0;

/// Find the single node with the given role in a [`TreeUpdate`]. The order of the nodes in an
/// update is unspecified, so tests must not depend on it.
#[track_caller]
fn find_node_with_role(update: &TreeUpdate, role: Role) -> &accesskit::Node {
    let mut matches = update.nodes.iter().filter(|(_, node)| node.role() == role);
    let node = matches
        .next()
        .unwrap_or_else(|| panic!("Update should contain a node with role {role:?}"));
    assert!(
        matches.next().is_none(),
        "Update should contain exactly one node with role {role:?}"
    );
    &node.1
}

#[track_caller]
fn assert_rect_eq(actual: Rect, expected: Rect) {
    // Bounds are converted from `Au`, which has a resolution of 1/60th of a CSS pixel.
    const EPSILON: f64 = 0.05;
    assert!(
        (actual.x0 - expected.x0).abs() < EPSILON &&
            (actual.y0 - expected.y0).abs() < EPSILON &&
            (actual.x1 - expected.x1).abs() < EPSILON &&
            (actual.y1 - expected.y1).abs() < EPSILON,
        "expected bounds {expected:?} but got {actual:?}"
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
    //
    // This stands in for the node an embedder builds to graft in a WebView's tree (see
    // `ports/servoshell/desktop/gui.rs`). It deliberately has no transform: the WebView is at the
    // origin of the window and the HiDPI scale factor is 1.0 in tests, so composed bounds are
    // equal to the viewport-relative CSS pixel bounds that layout produces. It also sets no
    // bounds, matching the embedder, as AccessKit consumers exclude graft nodes from the
    // presented tree and never read them.
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
