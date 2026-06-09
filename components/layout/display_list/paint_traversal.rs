/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;

use app_units::Au;
use servo_base::id::ScrollTreeNodeId;
use style::values::computed::TextDecorationLine;

use crate::display_list::{
    ClipId, FragmentTextDecoration, StackingContext, StackingContextFragments,
};
use crate::fragment_tree::{
    BoxFragment, Fragment, FragmentFlags, IFrameFragment, ImageFragment, PositioningFragment,
    TextFragment,
};
use crate::geom::{PhysicalPoint, PhysicalRect};

pub(crate) struct PaintTraversal<'a, Handler: PaintTraversalHandler> {
    handler: &'a mut Handler,
    outlines: Vec<(TraversalState, Arc<BoxFragment>)>,
    floats: Vec<(TraversalState, Arc<BoxFragment>)>,
}

impl<'a, Handler: PaintTraversalHandler> PaintTraversal<'a, Handler> {
    pub(crate) fn traverse(root_stacking_context: &StackingContext, handler: &'a mut Handler) {
        Self {
            handler,
            outlines: Vec::new(),
            floats: Vec::new(),
        }
        .traverse_stacking_context(&TraversalState::default(), root_stacking_context);
    }

    /// <https://drafts.csswg.org/css-position-4/#paint-a-stacking-context>
    fn traverse_stacking_context(
        &mut self,
        state: &TraversalState,
        stacking_context: &StackingContext,
    ) {
        let old_outlines_length = self.outlines.len();
        let state = state.push_stacking_context(stacking_context);
        let stacking_context_state = self.handler.visit_stacking_context(stacking_context);

        // > Step 1: If root is an element, paint a stacking context given root’s principal
        // > box and canvas, then return.
        // > Step 2: Assert: root is a box, and generates a stacking context.
        // > Ensured by the fact that `self` is a stacking context.

        // > Step 3: If root is a root element’s principal box, paint root’s background over
        // > the entire canvas, with the origin of the background positioning area being the
        // > position on canvas that would be used if root’s background was being painted
        // > normally.
        if let StackingContextFragments::Root = &stacking_context.fragment {
            self.handler.visit_box_for_root_background(&state);
        }

        // > Step 4: If root is a block-level box, paint a block’s decorations given root
        // > and canvas.
        let root_fragment = stacking_context.fragment();
        if let Some(root_fragment) = root_fragment &&
            !root_fragment.with_style().is_inline_box()
        {
            self.handle_box(&state, root_fragment);
        }

        // > Step 5: For each of root’s positioned descendants with negative (non-zero) z-index
        // > values, sort those descendants by z-index order (most negative first) then tree
        // > order, and paint a stacking context given each descendant and canvas.
        let mut children = stacking_context.children.iter().peekable();
        while children.peek().is_some_and(|child| child.z_index < 0) {
            self.traverse_stacking_context(
                &state,
                children.next().expect("Should have a value due to peek."),
            );
        }

        if let Some(root_fragment) = root_fragment {
            self.traverse_stacking_context_inner(&state, root_fragment);
        }

        // > Step 9: For each of root’s positioned descendants with z-index: auto or
        // > z-index: 0, in tree order:
        // >   ↪ descendant has z-index: auto
        // >     Paint a stacking container given the descendant and canvas.
        // >   ↪ descendant has z-index: 0
        // >     Paint a stacking context given the descendant and canvas.
        //
        // > Step 10: For each of root’s positioned descendants with positive (non-zero)
        // > z-index values, sort those descendants by z-index order (smallest first) then
        // > tree order, and paint a stacking context given each descendant and canvas.
        for child in children {
            assert!(child.z_index >= 0);
            self.traverse_stacking_context(&state, child);
        }

        // > Step 11: If the UA uses out-of-band outlines, draw all of root’s outlines
        // > (those that it skipped drawing due to not using in-band outlines during the
        // > current invocation of this algorithm) into canvas.
        if old_outlines_length < self.outlines.len() {
            for (state, outline_fragment) in &self.outlines.split_off(old_outlines_length) {
                self.handler.visit_box_for_outline(state, outline_fragment);
            }
        }

        self.handler
            .leave_stacking_context(&state, stacking_context_state);
    }

    fn traverse_stacking_context_inner(&mut self, state: &TraversalState, root: &Arc<BoxFragment>) {
        let root = &root.with_style();
        let old_float_length = self.floats.len();
        let mut saw_inline_level_or_replaced = root.is_replaced();

        // > Step 6: For each of root’s in-flow, non-positioned, block-level descendants, in
        // > tree order, paint a block’s decorations given the descendant and canvas.
        let inner_state = state.push_box_fragment(root);
        for child in root.children.iter() {
            saw_inline_level_or_replaced |=
                self.traverse_block_level_descendants_decorations(&inner_state, child);
        }

        // Collapsed table borders are painted after block-level descendants. This isn't
        // well specified, but is being discussed in <https://github.com/w3c/csswg-drafts/issues/11570>.
        if root.is_table_grid_with_collapsed_borders() {
            self.handler
                .visit_box_for_collapsed_table_borders(state, root);
        }

        // > Step 7: For each of root’s non-positioned floating descendants, in tree order,
        // > paint a stacking container given the descendant and canvas.
        if old_float_length < self.floats.len() {
            for (state, float_fragment) in &self.floats.split_off(old_float_length) {
                self.handle_box(state, float_fragment);
                self.traverse_stacking_context_inner(state, float_fragment);
            }
        }

        // Step 8:
        if root.is_inline_box() {
            // >  ↪ If root is an inline-level box
            // >     For each line box root is in, paint a box in a line box given root, the
            // >     line box, and canvas.
            self.traverse_box_in_a_line_box(state, root, true /* at_stacking_context_root */);
        } else if saw_inline_level_or_replaced {
            // >  ↪ Otherwise
            // >    First for root, then for all its in-flow, non-positioned, block-level
            // >    descendant boxes, in tree order:
            // >    1. If the box is a replaced element, paint the replaced content into canvas,
            // >       atomically.
            // >    2. Otherwise, for each line box of the box, paint a box in a line box given the
            // >       box, the line box, and canvas.
            // >    3. If the UA uses in-band outlines, paint the outlines of the box into canvas.
            self.traverse_line_boxes_and_replaced_for_box(
                state, root, true, /* at_root_of_stacking_context */
            );
        }
    }

    /// An implementation of
    /// <https://drafts.csswg.org/css-position-4/#paint-a-stacking-context> that only
    /// implements the parts relevant to stacking containers. This is an optimization to
    /// avoid work when descending into positioned container and stacking context
    /// contents.
    fn traverse_stacking_container(
        &mut self,
        state: &TraversalState,
        root: &Arc<BoxFragment>,
        is_block_level: bool,
    ) {
        let old_outlines_length = self.outlines.len();

        // > Step 4: If root is a block-level box, paint a block’s decorations given root
        // > and canvas.
        if is_block_level {
            self.handle_box(state, root);
        }

        // This is steps 6 through 8.
        self.traverse_stacking_context_inner(state, root);

        // > Step 11: If the UA uses out-of-band outlines, draw all of root’s outlines
        // > (those that it skipped drawing due to not using in-band outlines during the
        // > current invocation of this algorithm) into canvas.
        if old_outlines_length < self.outlines.len() {
            for (state, outline_fragment) in &self.outlines.split_off(old_outlines_length) {
                self.handler.visit_box_for_outline(state, outline_fragment);
            }
        }
    }

    fn traverse_block_level_descendants_decorations(
        &mut self,
        state: &TraversalState,
        fragment: &Fragment,
    ) -> bool {
        let mut saw_inline_level_or_replaced = false;

        match fragment {
            Fragment::Box(box_fragment) => {
                let box_fragment = &box_fragment.with_style();
                // If this box establishes a stacking context or stacking container, do not paint
                // it during this phase. Instead it is painted when the stacking context or container
                // is processed.
                if box_fragment.stacking_context_type().is_some() {
                    return false;
                }

                // We will process inline atomics during the inline level and replaced traversal.
                if box_fragment.is_atomic_inline_level() || box_fragment.is_flex_or_grid_item() {
                    return true;
                }

                // Don't paint any inline boxes, but do descend into them, in case they contain floats.
                if box_fragment.is_inline_box() {
                    saw_inline_level_or_replaced = true;
                } else {
                    self.handle_box(state, box_fragment);
                }

                if box_fragment.is_replaced() {
                    return true;
                }

                let state_for_children = state.push_box_fragment(box_fragment);
                for child in box_fragment.children.iter() {
                    saw_inline_level_or_replaced |= self
                        .traverse_block_level_descendants_decorations(&state_for_children, child);
                }

                // Collapsed table borders are painted after block-level descendants. This isn't
                // well specified, but is being discussed in <https://github.com/w3c/csswg-drafts/issues/11570>.
                if box_fragment.is_table_grid_with_collapsed_borders() {
                    self.handler
                        .visit_box_for_collapsed_table_borders(state, box_fragment);
                }
            },
            Fragment::Float(float_box_fragment) => {
                if float_box_fragment.stacking_context_type().is_none() {
                    self.floats
                        .push((state.without_text_decorations(), float_box_fragment.clone()));
                }
            },
            Fragment::Positioning(positioning_fragment) => {
                self.handler.visit_positioning(state, positioning_fragment);

                if positioning_fragment.is_line_box() {
                    saw_inline_level_or_replaced = true;
                }

                if !positioning_fragment.children.is_empty() {
                    let state = state.push_positioning_fragment(positioning_fragment);
                    for child in positioning_fragment.children.iter() {
                        saw_inline_level_or_replaced |=
                            self.traverse_block_level_descendants_decorations(&state, child);
                    }
                }
            },
            Fragment::AbsoluteOrFixedPositioned(..) |
            Fragment::Text(..) |
            Fragment::Image(..) |
            Fragment::IFrame(..) => {},
        }

        saw_inline_level_or_replaced
    }

    fn traverse_line_boxes_and_replaced_for_box(
        &mut self,
        state: &TraversalState,
        fragment: &Arc<BoxFragment>,
        at_root_of_stacking_context: bool,
    ) {
        let is_flex_or_grid = fragment.is_flex_or_grid_item();
        if fragment.is_replaced() {
            if is_flex_or_grid {
                self.handle_box(state, fragment);
            }

            let inner_state = state.push_box_fragment(fragment);
            self.traverse_replaced_content(&inner_state, fragment);
            return;
        }

        if !at_root_of_stacking_context && is_flex_or_grid {
            self.traverse_stacking_container(state, fragment, true /* is_block_level */);
            return;
        }

        let inner_state = state.push_box_fragment(fragment);
        for child in fragment.children.iter() {
            self.traverse_line_boxes_and_replaced(
                &inner_state,
                child,
                false, /* at_root_of_stacking_context */
            );
        }
    }

    fn traverse_line_boxes_and_replaced(
        &mut self,
        state: &TraversalState,
        fragment: &Fragment,
        at_root_of_stacking_context: bool,
    ) {
        match fragment {
            Fragment::Box(box_fragment) => {
                // If this box establishes a stacking context or stacking container, do not paint
                // it during this phase. Instead it is painted when the stacking context or container
                // is processed.
                if box_fragment.stacking_context_type().is_some() {
                    return;
                }
                self.traverse_line_boxes_and_replaced_for_box(
                    state,
                    box_fragment,
                    at_root_of_stacking_context,
                );
            },
            Fragment::Positioning(positioning_fragment) if positioning_fragment.is_line_box() => {
                let state = state.push_positioning_fragment(positioning_fragment);
                for child in &positioning_fragment.children {
                    self.traverse_fragment_in_a_line_box(&state, child);
                }
            },
            Fragment::Positioning(positioning_fragment) => {
                if !positioning_fragment.children.is_empty() {
                    let state = state.push_positioning_fragment(positioning_fragment);
                    for child in &positioning_fragment.children {
                        self.traverse_line_boxes_and_replaced(
                            &state, child, false, /* at at_root_of_stacking_context */
                        );
                    }
                }
            },
            Fragment::AbsoluteOrFixedPositioned(_) |
            Fragment::Float(..) |
            Fragment::IFrame(_) |
            Fragment::Image(_) |
            Fragment::Text(..) => {},
        }
    }

    fn traverse_fragment_in_a_line_box(&mut self, state: &TraversalState, fragment: &Fragment) {
        match fragment {
            Fragment::Box(box_fragment) => self.traverse_box_in_a_line_box(
                state,
                box_fragment,
                false, /* at_stacking_context_root */
            ),
            Fragment::Text(text_fragment) => {
                // This containing block is wrong and should use the size from the parent
                // positioning context.
                let containing_block =
                    PhysicalRect::new(state.origin, text_fragment.base.rect().size);
                self.handler
                    .visit_text(state, containing_block, text_fragment);
            },
            Fragment::AbsoluteOrFixedPositioned(..) | Fragment::Float(..) => {},
            Fragment::Positioning(..) => {
                unreachable!("Unexpected direct descendant PositioningContext of inline.")
            },
            Fragment::Image(..) | Fragment::IFrame(..) => {
                unreachable!("Unexpected replaced content direct descendant of inline.")
            },
        }
    }

    /// <https://www.w3.org/TR/css-position-4/#paint-a-box-in-a-line-box>
    fn traverse_box_in_a_line_box(
        &mut self,
        state: &TraversalState,
        box_fragment: &Arc<BoxFragment>,
        at_stacking_context_root: bool,
    ) {
        let box_fragment = &box_fragment.with_style();
        // If this box establishes a stacking context or stacking container, do not paint
        // it during this phase. Instead it is painted when the stacking context or container
        // is processed.
        if !at_stacking_context_root && box_fragment.stacking_context_type().is_some() {
            return;
        }

        // Block-in-inline split, return to block mode.
        let is_atomic_inline_level = box_fragment.is_atomic_inline_level();
        if !is_atomic_inline_level && !box_fragment.is_inline_box() {
            self.traverse_line_boxes_and_replaced_for_box(
                state,
                box_fragment,
                at_stacking_context_root,
            );
            return;
        }

        // > Step 1: Paint the backgrounds of root’s fragments that are in line box into canvas.
        // > Step 2: Paint the borders of root’s fragments that are in line box into canvas.
        self.handle_box(state, box_fragment);

        // > Step 3:
        //
        // Note: The following steps are in a different order than the specification
        // due to the way we classify fragments.
        //
        // > ↪ If root is an inline-level replaced element
        // >   Paint the replaced content into canvas, atomically.
        if box_fragment.is_replaced() {
            let state = state.push_box_fragment(box_fragment);
            self.traverse_replaced_content(&state, box_fragment);

        // > ↪ If root is an inline-level block or table wrapper box
        // >   Paint a stacking container given root and canvas.
        } else if is_atomic_inline_level || box_fragment.is_flex_or_grid_item() {
            self.traverse_stacking_container(state, box_fragment, false /* is_block_level */);

        // > ↪ If root is an inline box
        // Note: This is handled via recursion into `paint_a_fragment_in_a_line_box`.
        } else {
            let state = state.push_box_fragment(box_fragment);
            for child in &box_fragment.children {
                self.traverse_fragment_in_a_line_box(&state, child);
            }
        }
    }

    fn traverse_replaced_content(
        &mut self,
        state: &TraversalState,
        box_fragment: &Arc<BoxFragment>,
    ) {
        for child in &box_fragment.children {
            match child {
                Fragment::Image(image_fragment) => {
                    let containing_block =
                        PhysicalRect::new(state.origin, box_fragment.content_rect().size);
                    self.handler
                        .visit_image(state, containing_block, image_fragment);
                },
                Fragment::IFrame(iframe_fragment) => {
                    self.handler.visit_iframe(state, iframe_fragment);
                },
                Fragment::Box(box_fragment) => {
                    self.traverse_stacking_container(
                        &state.without_text_decorations(),
                        box_fragment,
                        true, /* is_block_level */
                    );
                },
                _ => {},
            }
        }
    }

    fn handle_box(&mut self, state: &TraversalState, fragment: &Arc<BoxFragment>) {
        if fragment.has_outline() {
            self.outlines.push((state.clone(), fragment.clone()));
        }
        self.handler.visit_box(state, fragment);
    }
}

pub(crate) trait PaintTraversalHandler {
    type StackingContextState;

    fn visit_stacking_context(
        &mut self,
        stacking_context: &StackingContext,
    ) -> Self::StackingContextState;
    fn leave_stacking_context(
        &mut self,
        state: &TraversalState,
        stacking_context_state: Self::StackingContextState,
    );

    fn visit_box(&mut self, state: &TraversalState, fragment: &Arc<BoxFragment>);
    fn visit_iframe(&mut self, _state: &TraversalState, _fragment: &Arc<IFrameFragment>) {}
    fn visit_image(
        &mut self,
        _state: &TraversalState,
        _containing_block: PhysicalRect<Au>,
        _fragment: &Arc<ImageFragment>,
    ) {
    }
    fn visit_text(
        &mut self,
        state: &TraversalState,
        containing_block: PhysicalRect<Au>,
        fragment: &Arc<TextFragment>,
    );
    fn visit_positioning(&mut self, _state: &TraversalState, _fragment: &Arc<PositioningFragment>) {
    }

    fn visit_box_for_root_background(&mut self, _state: &TraversalState) {}
    fn visit_box_for_outline(&mut self, _state: &TraversalState, _fragment: &Arc<BoxFragment>) {}
    fn visit_box_for_collapsed_table_borders(
        &mut self,
        _state: &TraversalState,
        _fragment: &Arc<BoxFragment>,
    ) {
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TraversalState {
    pub spatial_id: ScrollTreeNodeId,
    pub clip_id: ClipId,
    pub origin: PhysicalPoint<Au>,
    pub text_decorations: Rc<Vec<FragmentTextDecoration>>,
}

impl TraversalState {
    #[inline]
    pub(crate) fn push_box_fragment(&self, box_fragment: &Arc<BoxFragment>) -> Self {
        let box_fragment = box_fragment.with_style();
        let style = box_fragment.style();

        // Text decorations are not propagated to atomic inline-level descendants.
        // From https://drafts.csswg.org/css2/#lining-striking-props:
        //
        // > Note that text decorations are not propagated to floating and absolutely
        // > positioned descendants, nor to the contents of atomic inline-level descendants
        // > such as inline blocks and inline tables.
        //
        // Also do not propagate text decorations to floats or replaced content.
        let mut propagated_text_decorations = self.text_decorations.clone();
        if box_fragment.is_atomic_inline_level() ||
            box_fragment.base.flags.contains(
                FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER | FragmentFlags::IS_REPLACED,
            )
        {
            propagated_text_decorations = Default::default();
        }

        let text_decorations = match &style.get_text().text_decoration_line {
            &TextDecorationLine::NONE => propagated_text_decorations,
            line => {
                let mut new_vector = (*propagated_text_decorations).clone();
                let color = &style.get_inherited_text().color;
                new_vector.push(FragmentTextDecoration {
                    line: *line,
                    color: style
                        .clone_text_decoration_color()
                        .resolve_to_absolute(color),
                    style: style.clone_text_decoration_style(),
                });
                Rc::new(new_vector)
            },
        };

        Self {
            origin: self.origin + box_fragment.content_rect().origin.to_vector(),
            spatial_id: box_fragment
                .generated_scroll_tree_node_id()
                .unwrap_or(self.spatial_id),
            clip_id: box_fragment.generated_clip_id().unwrap_or(self.clip_id),
            text_decorations,
        }
    }

    pub(crate) fn without_text_decorations(&self) -> Self {
        Self {
            text_decorations: Default::default(),
            ..*self
        }
    }

    pub(crate) fn push_positioning_fragment(
        &self,
        positioning_fragment: &PositioningFragment,
    ) -> Self {
        Self {
            origin: self.origin + positioning_fragment.base.rect().origin.to_vector(),
            spatial_id: self.spatial_id,
            clip_id: self.clip_id,
            text_decorations: self.text_decorations.clone(),
        }
    }

    pub(crate) fn push_stacking_context(&self, stacking_context: &StackingContext) -> Self {
        Self {
            origin: stacking_context.containing_block_origin,
            spatial_id: stacking_context.scroll_tree_node_id,
            clip_id: stacking_context.clip_id,
            text_decorations: stacking_context.text_decorations.clone(),
        }
    }
}
