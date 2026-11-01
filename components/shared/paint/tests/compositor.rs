/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use euclid::Size2D;
use paint_api::display_list::{
    AxesScrollSensitivity, ScrollTree, ScrollType, ScrollableNodeInfo, SpatialTreeNodeInfo,
    TouchAction,
};
use servo_base::id::ScrollTreeNodeId;
use webrender_api::units::LayoutVector2D;
use webrender_api::{ExternalScrollId, PipelineId, ScrollLocation};

fn add_mock_scroll_node(tree: &mut ScrollTree) -> (ScrollTreeNodeId, ExternalScrollId) {
    let pipeline_id = PipelineId(0, 0);
    let num_nodes = tree.nodes.len();
    let parent = if num_nodes > 0 {
        Some(ScrollTreeNodeId {
            index: num_nodes - 1,
        })
    } else {
        None
    };

    let external_id = ExternalScrollId(num_nodes as u64, pipeline_id);
    let scroll_node_id = tree.add_scroll_tree_node(
        parent,
        SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
            external_id,
            content_rect: Size2D::new(200.0, 200.0).into(),
            clip_rect: Size2D::new(100.0, 100.0).into(),
            scroll_sensitivity: AxesScrollSensitivity {
                x: ScrollType::Script | ScrollType::InputEvents,
                y: ScrollType::Script | ScrollType::InputEvents,
            },
            touch_action: TouchAction::Auto,
            offset: LayoutVector2D::zero(),
            offset_changed: Cell::new(false),
        }),
    );
    (scroll_node_id, external_id)
}

#[test]
fn test_scroll_tree_simple_scroll() {
    let mut scroll_tree = ScrollTree::default();
    let (id, external_id) = add_mock_scroll_node(&mut scroll_tree);

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::Script,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(20.0, 40.0);
    assert_eq!(scrolled_id, external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(scroll_tree.get_node(id).offset(), Some(expected_offset));

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            external_id,
            ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
            ScrollType::Script,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(0.0, 0.0);
    assert_eq!(scrolled_id, external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(scroll_tree.get_node(id).offset(), Some(expected_offset));

    // Scroll offsets must be positive.
    let result = scroll_tree.scroll_node_or_ancestor(
        external_id,
        ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
        ScrollType::Script,
    );
    assert!(result.is_none());
    assert_eq!(
        scroll_tree.get_node(id).offset(),
        Some(LayoutVector2D::new(0.0, 0.0))
    );
}

#[test]
fn test_scroll_tree_simple_scroll_chaining() {
    let mut scroll_tree = ScrollTree::default();

    let pipeline_id = PipelineId(0, 0);
    let (parent_id, parent_external_id) = add_mock_scroll_node(&mut scroll_tree);

    let unscrollable_external_id = ExternalScrollId(100 as u64, pipeline_id);
    let unscrollable_child_id = scroll_tree.add_scroll_tree_node(
        Some(parent_id),
        SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
            external_id: unscrollable_external_id,
            content_rect: Size2D::new(100.0, 100.0).into(),
            clip_rect: Size2D::new(100.0, 100.0).into(),
            scroll_sensitivity: AxesScrollSensitivity {
                x: ScrollType::Script | ScrollType::InputEvents,
                y: ScrollType::Script | ScrollType::InputEvents,
            },
            touch_action: TouchAction::Auto,
            offset: LayoutVector2D::zero(),
            offset_changed: Cell::new(false),
        }),
    );

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            unscrollable_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::Script,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(20.0, 40.0);
    assert_eq!(scrolled_id, parent_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(parent_id).offset(),
        Some(expected_offset)
    );

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            unscrollable_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(10.0, 15.0)),
            ScrollType::Script,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(30.0, 55.0);
    assert_eq!(scrolled_id, parent_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(parent_id).offset(),
        Some(expected_offset)
    );
    assert_eq!(
        scroll_tree.get_node(unscrollable_child_id).offset(),
        Some(LayoutVector2D::zero())
    );
}

#[test]
fn test_scroll_tree_chain_when_at_extent() {
    let mut scroll_tree = ScrollTree::default();

    let (parent_id, parent_external_id) = add_mock_scroll_node(&mut scroll_tree);
    let (child_id, child_external_id) = add_mock_scroll_node(&mut scroll_tree);

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(child_external_id, ScrollLocation::End, ScrollType::Script)
        .unwrap();

    let expected_offset = LayoutVector2D::new(0.0, 100.0);
    assert_eq!(scrolled_id, child_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(child_id).offset(),
        Some(expected_offset)
    );

    // The parent will have scrolled because the child is already at the extent
    // of its scroll area in the y axis.
    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            child_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(0.0, 10.0)),
            ScrollType::Script,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(0.0, 10.0);
    assert_eq!(scrolled_id, parent_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(parent_id).offset(),
        Some(expected_offset)
    );
}

#[test]
fn test_scroll_tree_chain_through_overflow_hidden() {
    let mut scroll_tree = ScrollTree::default();

    // Create a tree with a scrollable leaf, but make its `scroll_sensitivity`
    // reflect `overflow: hidden` ie not responsive to non-script scroll events.
    let (parent_id, parent_external_id) = add_mock_scroll_node(&mut scroll_tree);
    let (overflow_hidden_id, overflow_hidden_external_id) = add_mock_scroll_node(&mut scroll_tree);
    let node = scroll_tree.get_node_mut(overflow_hidden_id);

    if let SpatialTreeNodeInfo::Scroll(ref mut scroll_node_info) = node.info {
        scroll_node_info.scroll_sensitivity = AxesScrollSensitivity {
            x: ScrollType::Script,
            y: ScrollType::Script,
        };
    }

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            overflow_hidden_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::InputEvents,
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(20.0, 40.0);
    assert_eq!(scrolled_id, parent_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(parent_id).offset(),
        Some(expected_offset)
    );
    assert_eq!(
        scroll_tree.get_node(overflow_hidden_id).offset(),
        Some(LayoutVector2D::new(0.0, 0.0))
    );
}

/// Add a scroll node with the given geometry and sensitivity under `parent`, returning its
/// [`ExternalScrollId`].
fn add_sized_scroll_node(
    tree: &mut ScrollTree,
    parent: Option<ScrollTreeNodeId>,
    id: u64,
    clip_size: (f32, f32),
    content_size: (f32, f32),
    sensitivity: ScrollType,
) -> (ScrollTreeNodeId, ExternalScrollId) {
    let external_id = ExternalScrollId(id, PipelineId(0, 0));
    let node_id = tree.add_scroll_tree_node(
        parent,
        SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
            external_id,
            content_rect: Size2D::new(content_size.0, content_size.1).into(),
            clip_rect: Size2D::new(clip_size.0, clip_size.1).into(),
            scroll_sensitivity: AxesScrollSensitivity {
                x: sensitivity,
                y: sensitivity,
            },
            touch_action: TouchAction::Auto,
            offset: LayoutVector2D::zero(),
            offset_changed: Cell::new(false),
        }),
    );
    (node_id, external_id)
}

/// The sensitivity of an `overflow: auto` node.
fn scrollable() -> ScrollType {
    ScrollType::Script | ScrollType::InputEvents | ScrollType::Touch
}

/// A viewport that does not respond to input events, as with `overflow: hidden`, with two
/// inner scrollers: a large one covering most of the viewport and a small one.
fn tree_with_unscrollable_root() -> (
    ScrollTree,
    ExternalScrollId,
    ExternalScrollId,
    ExternalScrollId,
) {
    let mut tree = ScrollTree::default();
    let (root_id, root) = add_sized_scroll_node(
        &mut tree,
        None,
        0,
        (500., 500.),
        (500., 3000.),
        ScrollType::Script,
    );
    let (_, large) = add_sized_scroll_node(
        &mut tree,
        Some(root_id),
        1,
        (500., 450.),
        (500., 5000.),
        scrollable(),
    );
    let (_, small) = add_sized_scroll_node(
        &mut tree,
        Some(root_id),
        2,
        (100., 100.),
        (100., 400.),
        scrollable(),
    );
    (tree, root, large, small)
}

#[test]
fn test_main_scroll_node_prefers_scrollable_root() {
    let mut tree = ScrollTree::default();
    let (root_id, root) = add_sized_scroll_node(
        &mut tree,
        None,
        0,
        (500., 500.),
        (500., 3000.),
        scrollable(),
    );
    add_sized_scroll_node(
        &mut tree,
        Some(root_id),
        1,
        (500., 500.),
        (500., 9000.),
        scrollable(),
    );
    assert_eq!(tree.main_scroll_node(root, None, 0.25), Some(root));
}

#[test]
fn test_main_scroll_node_falls_back_to_largest_inner_scroller() {
    let (tree, root, large, _) = tree_with_unscrollable_root();
    assert_eq!(tree.main_scroll_node(root, None, 0.25), Some(large));
}

#[test]
fn test_main_scroll_node_rejects_scrollers_below_area_floor() {
    let mut tree = ScrollTree::default();
    let (root_id, root) =
        add_sized_scroll_node(&mut tree, None, 0, (500., 500.), (500., 500.), scrollable());
    let (_, small) = add_sized_scroll_node(
        &mut tree,
        Some(root_id),
        1,
        (100., 100.),
        (100., 400.),
        scrollable(),
    );
    assert_eq!(tree.main_scroll_node(root, None, 0.25), None);
    // Nor is a small scroller kept just because it was chosen before.
    assert_eq!(tree.main_scroll_node(root, Some(small), 0.25), None);
}

#[test]
fn test_main_scroll_node_keeps_valid_previous_choice() {
    let (mut tree, root, large, _) = tree_with_unscrollable_root();
    let (_, other) = add_sized_scroll_node(
        &mut tree,
        Some(ScrollTreeNodeId { index: 0 }),
        3,
        (500., 480.),
        (500., 900.),
        scrollable(),
    );
    // `other` is now the largest, but the previous choice still qualifies.
    assert_eq!(tree.main_scroll_node(root, None, 0.25), Some(other));
    assert_eq!(tree.main_scroll_node(root, Some(large), 0.25), Some(large));
}

#[test]
fn test_main_scroll_node_drops_stale_previous_choice() {
    let (mut tree, root, large, _) = tree_with_unscrollable_root();

    // A previous choice that is no longer in the tree.
    let gone = ExternalScrollId(42, PipelineId(0, 0));
    assert_eq!(tree.main_scroll_node(root, Some(gone), 0.25), Some(large));

    // A previous choice that can no longer be scrolled.
    let (_, shrunk) = add_sized_scroll_node(
        &mut tree,
        Some(ScrollTreeNodeId { index: 0 }),
        4,
        (500., 480.),
        (500., 480.),
        scrollable(),
    );
    assert_eq!(tree.main_scroll_node(root, Some(shrunk), 0.25), Some(large));
}

#[test]
fn test_main_scroll_node_empty_tree() {
    let tree = ScrollTree::default();
    let root = ExternalScrollId(0, PipelineId(0, 0));
    assert_eq!(tree.main_scroll_node(root, None, 0.25), None);
}

#[test]
fn test_scroll_node_info() {
    let (tree, root, large, _) = tree_with_unscrollable_root();
    let info = tree.scroll_node_info(large).unwrap();
    assert_eq!(info.external_id, large);
    assert_eq!(info.scrollable_size(), Size2D::new(0., 4550.));
    assert!(info.can_scroll(ScrollType::InputEvents));
    assert!(
        !tree
            .scroll_node_info(root)
            .unwrap()
            .can_scroll(ScrollType::InputEvents)
    );
    assert!(
        tree.scroll_node_info(root)
            .unwrap()
            .can_scroll(ScrollType::Script)
    );
    assert!(
        tree.scroll_node_info(ExternalScrollId(42, PipelineId(0, 0)))
            .is_none()
    );
}
