/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::id::ScrollTreeNodeId;
use euclid::{Box2D, Rect, Size2D};
use paint_api::display_list::{
    AxesScrollSensitivity, ScrollTree, ScrollType, ScrollableNodeInfo, SpatialTreeNodeInfo,
};
use webrender_api::units::{LayoutRect, LayoutSize, LayoutVector2D};
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
            offset: LayoutVector2D::zero(),
            offset_changed: Cell::new(false),
            linked_nodes: None,
        }),
    );
    (scroll_node_id, external_id)
}

fn set_mock_scroll_node_rects(
    tree: &mut ScrollTree,
    id: ScrollTreeNodeId,
    clip_rect: LayoutRect,
    content_rect: LayoutRect,
) {
    let node = tree.get_node_mut(id);
    let scroll_info = node.as_scroll_info_mut().unwrap();
    scroll_info.clip_rect = clip_rect;
    scroll_info.content_rect = content_rect;
}

fn add_mock_scroll_node_with_sizes(
    tree: &mut ScrollTree,
    clip_rect: LayoutSize,
    content_rect: LayoutSize,
) -> (ScrollTreeNodeId, ExternalScrollId) {
    let (scroll_node_id, external_id) = add_mock_scroll_node(tree);
    set_mock_scroll_node_rects(tree, scroll_node_id, clip_rect.into(), content_rect.into());
    (scroll_node_id, external_id)
}

// Though there are multiple ways to layout the scrollbar rectangle, here we are placing it by assuming that it is purely
// bases on the ratio of the clip rect and content rect.
fn add_mock_scroll_node_with_scrollbars(
    tree: &mut ScrollTree,
    clip_rect: LayoutSize,
    content_rect: LayoutSize,
) -> (
    (ScrollTreeNodeId, ExternalScrollId),
    (ScrollTreeNodeId, ExternalScrollId),
    (ScrollTreeNodeId, ExternalScrollId),
) {
    let main_ids = add_mock_scroll_node_with_rects(tree, clip_rect, content_rect);

    let width_size_ratio = clip_rect.width / content_rect.width;
    let height_size_ratio = clip_rect.height / content_rect.height;

    let horizontal_scrollbar_ids = add_mock_scroll_node_with_rects(
        tree,
        Size2D::new(clip_rect.width * width_size_ratio, 10.0).into(),
        Size2D::new(content_rect.width * width_size_ratio, 10.0).into(),
    );

    let vertical_scrollbar_ids = add_mock_scroll_node_with_rects(
        tree,
        Size2D::new(10.0, clip_rect.height * height_size_ratio).into(),
        Size2D::new(10.0, content_rect.height * height_size_ratio).into(),
    );

    (main_ids, horizontal_scrollbar_ids, vertical_scrollbar_ids)
}

#[test]
fn test_scroll_tree_simple_scroll() {
    let mut scroll_tree = ScrollTree::default();
    let (id, external_id) = add_mock_scroll_node(&mut scroll_tree);

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::Script,
        )
        .unwrap()
        .first();
    let expected_offset = LayoutVector2D::new(20.0, 40.0);
    assert_eq!(scrolled_id, external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(scroll_tree.get_node(id).offset(), Some(expected_offset));

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            external_id,
            ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
            ScrollType::Script,
        )
        .unwrap()
        .first();
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
            offset: LayoutVector2D::zero(),
            offset_changed: Cell::new(false),
            linked_nodes: None,
        }),
    );

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            unscrollable_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::Script,
        )
        .unwrap()
        .first();
    let expected_offset = LayoutVector2D::new(20.0, 40.0);
    assert_eq!(scrolled_id, parent_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(parent_id).offset(),
        Some(expected_offset)
    );

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            unscrollable_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(10.0, 15.0)),
            ScrollType::Script,
        )
        .unwrap()
        .first();
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

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(child_external_id, ScrollLocation::End, ScrollType::Script)
        .unwrap()
        .first();

    let expected_offset = LayoutVector2D::new(0.0, 100.0);
    assert_eq!(scrolled_id, child_external_id);
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(child_id).offset(),
        Some(expected_offset)
    );

    // The parent will have scrolled because the child is already at the extent
    // of its scroll area in the y axis.
    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            child_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(0.0, 10.0)),
            ScrollType::Script,
        )
        .unwrap()
        .first();
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

    let (scrolled_id, offset) = *scroll_tree
        .scroll_node_or_ancestor(
            overflow_hidden_external_id,
            ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)),
            ScrollType::InputEvents,
        )
        .unwrap()
        .first();
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

#[test]
fn test_link_scroll_nodes() {
    let mut scroll_tree = ScrollTree::default();

    let (
        (main_id, main_external_id),
        (horizontal_scrollbar_id, horizontal_scrollbar_external_id),
        (vertical_scrollbar_id, vertical_scrollbar_external_id),
    ) = add_mock_scroll_node_with_scrollbars(
        tree,
        Size2D::new(100.0, 200.0).into(),
        Size2D::new(500.0, 800.0).into(),
    );

    scroll_tree.set_linked_scrollbar_nodes(
        main_id,
        Some(horizontal_scrollbar_id),
        Some(vertical_scrollbar_id),
    );

    // Test that the offsets of the linked nodes is being reflected properly.
    assert_eq!(
        scroll_tree.get_node(main_id).offset(),
        Some(LayoutVector2D::new(0.0, 0.0))
    );
    assert_eq!(
        scroll_tree.get_node(horizontal_scrollbar_id).offset(),
        Some(
            scroll_tree
                .get_node(horizontal_scrollbar_id)
                .as_scroll_info()
                .unwrap()
                .scrollable_size()
                .to_vector()
        )
    );
    assert_eq!(
        scroll_tree.get_node(vertical_scrollbar_id).offset(),
        Some(
            scroll_tree
                .get_node(vertical_scrollbar_id)
                .as_scroll_info()
                .unwrap()
                .scrollable_size()
                .to_vector()
        )
    );
}

#[test]
fn test_scroll_linked_scroll_nodes() {
    let mut scroll_tree = ScrollTree::default();

    let (
        (main_id, main_external_id),
        (horizontal_scrollbar_id, horizontal_scrollbar_external_id),
        (vertical_scrollbar_id, vertical_scrollbar_external_id),
    ) = add_mock_scroll_node_with_scrollbars(
        tree,
        Size2D::new(100.0, 200.0).into(),
        Size2D::new(500.0, 800.0).into(),
    );

    scroll_tree.set_linked_scrollbar_nodes(
        main_id,
        Some(horizontal_scrollbar_id),
        Some(vertical_scrollbar_id),
    );

    // Helper function to asserts the offsets of all three nodes.
    let asserts_scroll_nodes_offset =
        |scroll_tree: &ScrollTree,
         scroll_results: Vec<&(ExternalScrollId, LayoutVector2D)>,
         main_offset: LayoutVector2D,
         horizontal_scrollbar_offset: LayoutVector2D,
         vertical_scrollbar_offset: LayoutVector2D| {
            // The first element must be the main scroll node.
            assert_eq!(*scroll_results[0], (main_external_id, main_offset));

            // Otherwise we looks for other scroll nodes.
            assert_eq!(
                **scroll_results
                    .iter()
                    .find(|result| result.0 == horizontal_scrollbar_external_id)
                    .unwrap(),
                (
                    horizontal_scrollbar_external_id,
                    horizontal_scrollbar_offset
                )
            );
            assert_eq!(
                **scroll_results
                    .iter()
                    .find(|result| result.0 == vertical_scrollbar_external_id)
                    .unwrap(),
                (vertical_scrollbar_external_id, vertical_scrollbar_offset)
            );

            // Additionally, check that the ScrollResult represent the current scroll offset.
            let main_node = scroll_tree.get_node(main_id);
            let horizontal_scrollbar_node = scroll_tree.get_node(horizontal_scrollbar_id);
            let vertical_scrollbar_node = scroll_tree.get_node(vertical_scrollbar_id);

            assert_eq!(main_node.offset(), Some(main_offset));
            assert_eq!(
                horizontal_scrollbar_node.offset(),
                Some(horizontal_scrollbar_offset)
            );
            assert_eq!(
                vertical_scrollbar_node.offset(),
                Some(vertical_scrollbar_offset)
            );
        };

    // Scroll to a certain offsets.
    {
        let scroll_results = scroll_tree
            .scroll_node_or_ancestor(
                main_external_id,
                ScrollLocation::Delta(LayoutVector2D::new(100.0, 200.0)),
                ScrollType::InputEvents,
            )
            .unwrap();

        let scroll_results = scroll_results.iter().collect::<Vec<_>>();

        asserts_scroll_nodes_offset(
            &scroll_tree,
            scroll_results,
            LayoutVector2D::new(100.0, 200.0),
            LayoutVector2D::new(60.0, 0.0),
            LayoutVector2D::new(0.0, 100.0),
        );
    }

    // Scroll to maximum, beyond the scrollable size.
    {
        let scroll_results = scroll_tree
            .scroll_node_or_ancestor(
                main_external_id,
                ScrollLocation::Delta(LayoutVector2D::new(10000.0, 10000.0)),
                ScrollType::InputEvents,
            )
            .unwrap();

        let scroll_results = scroll_results.iter().collect::<Vec<_>>();

        asserts_scroll_nodes_offset(
            &scroll_tree,
            scroll_results,
            LayoutVector2D::new(400.0, 600.0),
            LayoutVector2D::new(0.0, 0.0),
            LayoutVector2D::new(0.0, 0.0),
        );
    }
}
