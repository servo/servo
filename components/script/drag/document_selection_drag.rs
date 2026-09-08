/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use js::context::{JSContext, NoGC};
use script_bindings::root::{Dom, DomRoot};
use servo_base::text::Utf32CodeUnitsOrNodeOffset;

use crate::dom::comparator::compare_dom_positions;
use crate::dom::inputevent::HitTestResult;
use crate::dom::selection::UsedUserSelect;
use crate::dom::traversal::FlatTreeForSelectionNoGcTraversal;
use crate::dom::{Node, NodeTraits};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentSelectionDragHandler {
    /// The node with `user-select: contain` that this selection must not leave, if any
    user_select_contain_node_for_selection_anchor: Option<Dom<Node>>,
}

impl DocumentSelectionDragHandler {
    pub(crate) fn new(user_select_contain_node: Option<&Node>) -> Self {
        Self {
            user_select_contain_node_for_selection_anchor: user_select_contain_node
                .map(Dom::from_ref),
        }
    }

    pub(crate) fn still_connected(&self) -> bool {
        true
    }

    /// Process a mouse move event on this [`DocumentSelectionDragHandler`].
    ///
    /// Returns `true` if the drag should continue and `false` otherwise.
    pub(crate) fn moved(&self, cx: &mut JSContext, hit_test_result: &HitTestResult) -> bool {
        let Some((container, offset)) = hit_test_result.dom_position_for_selection.as_ref() else {
            return true;
        };
        let Some(selection) = container.owner_document().selection() else {
            return true;
        };
        let (container, offset) = adjust_focus_for_user_select(
            cx,
            selection.composed_anchor_position(),
            container.clone(),
            *offset,
            self.user_select_contain_node_for_selection_anchor
                .as_deref(),
        );
        selection.collapse_or_extend_to_dom_position(cx, &container, offset);
        true
    }
}

/// Adjust the range boundary for the selection anchor (start of a drag gesture)
/// based on [`user-select`] of the container node and of its ancestors.
///
/// Returns `None` if no new drag gesture should be started (`user-select: none`), or:
///
/// * The new anchor container
/// * The new anchor offset
/// * The node with `user-select: contain` that the selection must not leave, if any
///
/// [`user-select`]: https://drafts.csswg.org/css-ui-4/#content-selection
pub(crate) fn adjust_anchor_for_user_select(
    no_gc: &NoGC,
    mut anchor_container_candidate: DomRoot<Node>,
    mut anchor_offset_candidate: Utf32CodeUnitsOrNodeOffset,
) -> Option<(
    DomRoot<Node>,
    Utf32CodeUnitsOrNodeOffset,
    Option<DomRoot<Node>>,
)> {
    let mut cache = Default::default();
    if anchor_container_candidate.used_user_select(no_gc, &mut cache) == UsedUserSelect::None {
        return None;
    }

    // The nearest inclusive ancestor with `user-select: contain`
    let mut nearest_with_user_select_contain = None;

    // The furthest inclusive ancestor with `user-select: all`, as long as all intermediate
    // inclusive ancestors also have `user-select: all`.
    let mut furthest_with_user_select_all = None;
    let mut each_so_far_has_user_select_all = true;

    for ancestor in anchor_container_candidate.inclusive_ancestors_in_flat_tree_unrooted(no_gc) {
        match ancestor.used_user_select(no_gc, &mut cache) {
            UsedUserSelect::Text | UsedUserSelect::None => {
                each_so_far_has_user_select_all = false;
            },
            UsedUserSelect::Contain => {
                nearest_with_user_select_contain = Some(ancestor.as_rooted());
                // `each_so_far_has_user_select_all` should be false but will no longer be used
                break;
            },
            UsedUserSelect::All => {
                if each_so_far_has_user_select_all {
                    furthest_with_user_select_all = Some(ancestor);
                }
            },
        }
    }

    if let Some(atomic) = furthest_with_user_select_all {
        // TODO: to properly implement `user-select: all` we should keep track of two potential
        // start boundaries: one with offset zero as is done here, and another with offset
        // `Node::len(atomic)` representing the end of that node. The latter would be used
        // when the range is backwards (if the end bounary is outside of and before `atomic`)
        // so that `atomic` would be entirely selected regardless of the range direction.
        anchor_container_candidate = atomic.as_rooted();
        anchor_offset_candidate = Utf32CodeUnitsOrNodeOffset(0);
    }
    Some((
        anchor_container_candidate,
        anchor_offset_candidate,
        nearest_with_user_select_contain,
    ))
}

/// Adjust the range boundary for the selection focus (end of a drag gesture)
/// based on [`user-select`] of the container node and of its ancestors.
///
/// [`user-select`]: https://drafts.csswg.org/css-ui-4/#content-selection
pub(crate) fn adjust_focus_for_user_select(
    no_gc: &NoGC,
    anchor: Option<(DomRoot<Node>, u32)>,
    mut focus_container_candidate: DomRoot<Node>,
    mut focus_offset_candidate: Utf32CodeUnitsOrNodeOffset,
    user_select_contain_node_for_anchor: Option<&Node>,
) -> (DomRoot<Node>, Utf32CodeUnitsOrNodeOffset) {
    let mut cache = Default::default();
    let mut user_select_contain_node_for_anchor_is_inclusive_ancestor = false;
    let mut each_so_far_has_user_select_all = true;
    let mut furthest_node_with_user_select_all = None;
    let mut each_so_far_has_user_select_none = true;
    // Either `user-select: contain` the selection starts outside of, or `user-select: none`
    let mut furthest_node_to_avoid = None;

    for inclusive_ancestor in
        focus_container_candidate.inclusive_ancestors_in_flat_tree_unrooted(no_gc)
    {
        if user_select_contain_node_for_anchor == Some(&**inclusive_ancestor) {
            user_select_contain_node_for_anchor_is_inclusive_ancestor = true
        }
        match inclusive_ancestor.used_user_select(no_gc, &mut cache) {
            UsedUserSelect::Text => {
                each_so_far_has_user_select_all = false;
                each_so_far_has_user_select_none = false;
            },
            UsedUserSelect::None => {
                each_so_far_has_user_select_all = false;
                if each_so_far_has_user_select_none {
                    furthest_node_to_avoid = Some(inclusive_ancestor)
                }
            },
            UsedUserSelect::Contain => {
                each_so_far_has_user_select_all = false;
                each_so_far_has_user_select_none = false;
                if !user_select_contain_node_for_anchor_is_inclusive_ancestor {
                    furthest_node_to_avoid = Some(inclusive_ancestor)
                }
            },
            UsedUserSelect::All => {
                each_so_far_has_user_select_none = false;
                if each_so_far_has_user_select_all {
                    furthest_node_with_user_select_all = Some(inclusive_ancestor)
                }
            },
        }
        if !each_so_far_has_user_select_all &&
            !each_so_far_has_user_select_none &&
            user_select_contain_node_for_anchor_is_inclusive_ancestor
        {
            // Nothing else to find in ancestors
            break;
        }
    }
    let selection_is_backward = || {
        anchor
            .as_ref()
            .is_some_and(|(anchor_container, anchor_offset)| {
                let (ordering, _containment) =
                    compare_dom_positions::<FlatTreeForSelectionNoGcTraversal>(
                        no_gc,
                        anchor_container,
                        *anchor_offset,
                        &focus_container_candidate,
                        focus_offset_candidate.0,
                    );
                ordering == Some(Ordering::Greater)
            })
    };
    if let Some(contain_for_anchor) = user_select_contain_node_for_anchor &&
        !user_select_contain_node_for_anchor_is_inclusive_ancestor
    {
        // `focus_container` is outside of `contain_for_anchor`:
        // find the closest position within `contain_for_anchor`: either its start or end
        focus_offset_candidate = if selection_is_backward() {
            Utf32CodeUnitsOrNodeOffset(0)
        } else {
            Utf32CodeUnitsOrNodeOffset(Node::len(contain_for_anchor))
        };
        focus_container_candidate = DomRoot::from_ref(contain_for_anchor);
    } else if let Some(focus_ancestor_to_avoid) = furthest_node_to_avoid {
        focus_offset_candidate = if selection_is_backward() {
            Utf32CodeUnitsOrNodeOffset(Node::len(&focus_ancestor_to_avoid))
        } else {
            Utf32CodeUnitsOrNodeOffset(0)
        };
        focus_container_candidate = focus_ancestor_to_avoid.as_rooted();
    } else if let Some(with_user_select_all) = furthest_node_with_user_select_all {
        // anchor was snapped to the start of the `user-select: all` element,
        // so snap the focus to the end unconditionally
        focus_offset_candidate = Utf32CodeUnitsOrNodeOffset(Node::len(&with_user_select_all));
        focus_container_candidate = with_user_select_all.as_rooted();
    }

    (focus_container_candidate, focus_offset_candidate)
}
