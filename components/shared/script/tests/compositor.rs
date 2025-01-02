/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use webrender_api::units::LayoutVector2D;
use webrender_api::{ExternalScrollId, PipelineId, ScrollLocation, SpatialId};
use webrender_traits::display_list::{
    ScrollSensitivity, ScrollTree, ScrollTreeNodeId, ScrollableNodeInfo,
};

fn add_mock_scroll_node(tree: &mut ScrollTree) -> ScrollTreeNodeId {
    let pipeline_id = PipelineId(0, 0);
    let num_nodes = tree.nodes.len();
    let parent = if num_nodes > 0 {
        Some(ScrollTreeNodeId {
            index: num_nodes - 1,
            spatial_id: SpatialId::new(num_nodes - 1, pipeline_id),
        })
    } else {
        None
    };

    tree.add_scroll_tree_node(
        parent.as_ref(),
        SpatialId::new(num_nodes, pipeline_id),
        Some(ScrollableNodeInfo {
            external_id: ExternalScrollId(num_nodes as u64, pipeline_id),
            scrollable_size: Size2D::new(100.0, 100.0),
            scroll_sensitivity: ScrollSensitivity::ScriptAndInputEvents,
            offset: LayoutVector2D::zero(),
        }),
    )
}

#[test]
fn test_scroll_tree_simple_scroll() {
    let mut scroll_tree = ScrollTree::default();
    let pipeline_id = PipelineId(0, 0);
    let id = add_mock_scroll_node(&mut scroll_tree);

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            &id,
            ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(-20.0, -40.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(scroll_tree.get_node(&id).offset(), Some(expected_offset));

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(&id, ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)))
        .unwrap();
    let expected_offset = LayoutVector2D::new(0.0, 0.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(scroll_tree.get_node(&id).offset(), Some(expected_offset));

    // Scroll offsets must be negative.
    let result = scroll_tree
        .scroll_node_or_ancestor(&id, ScrollLocation::Delta(LayoutVector2D::new(20.0, 40.0)));
    assert!(result.is_none());
    assert_eq!(
        scroll_tree.get_node(&id).offset(),
        Some(LayoutVector2D::new(0.0, 0.0))
    );
}

#[test]
fn test_scroll_tree_simple_scroll_chaining() {
    let mut scroll_tree = ScrollTree::default();

    let pipeline_id = PipelineId(0, 0);
    let parent_id = add_mock_scroll_node(&mut scroll_tree);
    let unscrollable_child_id =
        scroll_tree.add_scroll_tree_node(Some(&parent_id), SpatialId::new(1, pipeline_id), None);

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            &unscrollable_child_id,
            ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(-20.0, -40.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(&parent_id).offset(),
        Some(expected_offset)
    );

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            &unscrollable_child_id,
            ScrollLocation::Delta(LayoutVector2D::new(-10.0, -15.0)),
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(-30.0, -55.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(&parent_id).offset(),
        Some(expected_offset)
    );
    assert_eq!(scroll_tree.get_node(&unscrollable_child_id).offset(), None);
}

#[test]
fn test_scroll_tree_chain_when_at_extent() {
    let mut scroll_tree = ScrollTree::default();

    let pipeline_id = PipelineId(0, 0);
    let parent_id = add_mock_scroll_node(&mut scroll_tree);
    let child_id = add_mock_scroll_node(&mut scroll_tree);

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(&child_id, ScrollLocation::End)
        .unwrap();

    let expected_offset = LayoutVector2D::new(0.0, -100.0);
    assert_eq!(scrolled_id, ExternalScrollId(1, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(&child_id).offset(),
        Some(expected_offset)
    );

    // The parent will have scrolled because the child is already at the extent
    // of its scroll area in the y axis.
    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            &child_id,
            ScrollLocation::Delta(LayoutVector2D::new(0.0, -10.0)),
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(0.0, -10.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(&parent_id).offset(),
        Some(expected_offset)
    );
}

#[test]
fn test_scroll_tree_chain_through_overflow_hidden() {
    let mut scroll_tree = ScrollTree::default();

    // Create a tree with a scrollable leaf, but make its `scroll_sensitivity`
    // reflect `overflow: hidden` ie not responsive to non-script scroll events.
    let pipeline_id = PipelineId(0, 0);
    let parent_id = add_mock_scroll_node(&mut scroll_tree);
    let overflow_hidden_id = add_mock_scroll_node(&mut scroll_tree);
    scroll_tree
        .get_node_mut(&overflow_hidden_id)
        .scroll_info
        .as_mut()
        .map(|info| {
            info.scroll_sensitivity = ScrollSensitivity::Script;
        });

    let (scrolled_id, offset) = scroll_tree
        .scroll_node_or_ancestor(
            &overflow_hidden_id,
            ScrollLocation::Delta(LayoutVector2D::new(-20.0, -40.0)),
        )
        .unwrap();
    let expected_offset = LayoutVector2D::new(-20.0, -40.0);
    assert_eq!(scrolled_id, ExternalScrollId(0, pipeline_id));
    assert_eq!(offset, expected_offset);
    assert_eq!(
        scroll_tree.get_node(&parent_id).offset(),
        Some(expected_offset)
    );
    assert_eq!(
        scroll_tree.get_node(&overflow_hidden_id).offset(),
        Some(LayoutVector2D::new(0.0, 0.0))
    );
}
