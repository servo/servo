/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use js::context::{JSContext, NoGC};
use rustc_hash::FxHashMap;
use script_bindings::dom::UnrootedDom;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use servo_base::text::Utf32CodeUnitsOrNodeOffset;
use style::values::computed::UserSelect;

use crate::dom::comparator::compare_dom_positions;
use crate::dom::inputevent::HitTestResult;
use crate::dom::selection::UsedUserSelect;
use crate::dom::traversal::FlatTreeForSelectionNoGcTraversal;
use crate::dom::{Element, Node, NodeTraits};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentSelectionDragHandler {
    /// The node with `user-select: contain` that this selection must not leave, if any.
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
        self.user_select_contain_node_for_selection_anchor
            .as_ref()
            .is_none_or(|node| node.is_connected())
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
                // `each_so_far_has_user_select_all` should be set to false but will no
                // longer be used after this break.
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
        // when the range is backwards (if the end boundary is outside of and before `atomic`)
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
                let (ordering, _containment) = compare_dom_positions::<
                    FlatTreeForSelectionNoGcTraversal,
                >(
                    no_gc,
                    anchor_container,
                    *anchor_offset,
                    &focus_container_candidate,
                    focus_container_candidate.to_sibling_or_utf16_offset(focus_offset_candidate),
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

impl Node {
    /// Returns the used value of <https://drafts.csswg.org/css-ui-4/#propdef-user-select>.
    ///
    /// It is the caller’s responsibility to ensure that style is up to date for this node and
    /// its (flat tree) ancestors.
    ///
    /// `cache` can be initialized with `Default::default()`, and should be shared across calls
    /// for nodes that may share some (flat tree) ancestors.
    pub(crate) fn used_user_select<'no_gc>(
        &self,
        no_gc: &'no_gc NoGC,
        cache: &mut FxHashMap<UnrootedDom<'no_gc, Node>, UsedUserSelect>,
    ) -> UsedUserSelect {
        let cache_key = UnrootedDom::from_ref(self, no_gc);
        if let Some(&used_value) = cache.get(&cache_key) {
            return used_value;
        }
        // > The used value is the same as the computed value, except:
        // >
        // > 1. on editable elements where the used value is always `contain`
        // >    regardless of the computed value
        // > 2. when the computed value is `auto`, in which case the used value
        // >    is one of the other values as defined below
        // >
        // > For the purpose of this specification, an editable element is
        // > either an editing host or a mutable form control with textual
        // > content, such as textarea.
        //
        // For form controls, we handle selection separately without looking at
        // `user-select`.
        if self.is_editing_host() {
            let used_value = UsedUserSelect::Contain;
            cache.insert(cache_key, used_value);
            return used_value;
        }
        let computed_value = self
            .downcast()
            .and_then(Element::computed_user_select)
            // Non-element nodes and unstyled elements: use the initial value
            .unwrap_or(UserSelect::Auto);
        let used_value = match computed_value {
            UserSelect::Text => UsedUserSelect::Text,
            UserSelect::None => UsedUserSelect::None,
            UserSelect::Contain => UsedUserSelect::Contain,
            UserSelect::All => UsedUserSelect::All,
            UserSelect::Auto => {
                let parent_used_value = self
                    .parent_in_flat_tree(no_gc)
                    .into_parent()
                    .map(|parent| parent.used_user_select(no_gc, cache));
                // > The used value of `auto` is determined as follows:
                // > * On the `::before` and `::after` pseudo-elements, the used value is `none`
                // > * If the element is an editable element, the used value is `contain`
                // > * Otherwise, if the used value of `user-select` on the parent of this element
                // >   is `all`, the used value is `all`
                // > * Otherwise, if the used value of `user-select` on the parent of this element
                // >   is `none`, the used value is `none`
                // > * Otherwise, the used value is `text`
                match parent_used_value {
                    Some(UsedUserSelect::All) => UsedUserSelect::All,
                    Some(UsedUserSelect::None) => UsedUserSelect::None,
                    _ => UsedUserSelect::Text,
                }
            },
        };
        cache.insert(cache_key, used_value);
        used_value
    }
}
