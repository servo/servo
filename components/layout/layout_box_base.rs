/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use euclid::Point2D;
use layout_api::LayoutDamage;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use style::computed_values::position::T as Position;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::specified::align::AlignFlags;
use style_traits::CSSPixel;

use crate::context::LayoutContext;
use crate::dom::{LayoutBox, WeakLayoutBox};
use crate::flow::CollapsibleWithParentStartMargin;
use crate::formatting_contexts::Baselines;
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, Fragment, FragmentStatus,
    SpecificLayoutInfo,
};
use crate::geom::LogicalSides1D;
use crate::positioned::{PositioningContext, relative_adjustement};
use crate::sizing::{ComputeInlineContentSizes, InlineContentSizesResult, SizeConstraint};
use crate::{ConstraintSpace, ContainingBlock, ContainingBlockSize};

/// A box tree node that handles containing information about style and the original DOM
/// node or pseudo-element that it is based on. This also handles caching of layout values
/// such as the inline content sizes to avoid recalculating these values during layout
/// passes.
///
/// In the future, this will hold layout results to support incremental layout.
#[derive(MallocSizeOf)]
pub(crate) struct LayoutBoxBase {
    pub base_fragment_info: BaseFragmentInfo,
    pub style: ServoArc<ComputedValues>,
    pub cached_inline_content_size:
        AtomicRefCell<Option<Box<(SizeConstraint, InlineContentSizesResult)>>>,
    pub outer_inline_content_sizes_depend_on_content: AtomicBool,
    pub cached_layout_result: AtomicRefCell<Option<LayoutResultAndInputs>>,
    pub fragments: AtomicRefCell<Vec<Fragment>>,
    pub parent_box: Option<WeakLayoutBox>,
}

impl LayoutBoxBase {
    pub(crate) fn new(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
    ) -> Self {
        Self {
            base_fragment_info,
            style,
            cached_inline_content_size: AtomicRefCell::default(),
            outer_inline_content_sizes_depend_on_content: AtomicBool::new(true),
            cached_layout_result: AtomicRefCell::default(),
            fragments: AtomicRefCell::default(),
            parent_box: None,
        }
    }

    /// Get the inline content sizes of a box tree node that extends this [`LayoutBoxBase`], fetch
    /// the result from a cache when possible.
    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
        layout_box: &impl ComputeInlineContentSizes,
    ) -> InlineContentSizesResult {
        let mut cache = self.cached_inline_content_size.borrow_mut();
        if let Some(cached_inline_content_size) = cache.as_ref() {
            let (previous_cb_block_size, result) = **cached_inline_content_size;
            if !result.depends_on_block_constraints ||
                previous_cb_block_size == constraint_space.block_size
            {
                return result;
            }
            // TODO: Should we keep multiple caches for various block sizes?
        }

        let result =
            layout_box.compute_inline_content_sizes_with_fixup(layout_context, constraint_space);
        *cache = Some(Box::new((constraint_space.block_size, result)));
        result
    }

    pub(crate) fn fragments(&self) -> Vec<Fragment> {
        self.fragments.borrow().clone()
    }

    pub(crate) fn add_fragment(&self, fragment: Fragment) {
        self.fragments.borrow_mut().push(fragment);
    }

    pub(crate) fn set_fragment(&self, fragment: Fragment) {
        *self.fragments.borrow_mut() = vec![fragment];
    }

    pub(crate) fn clear_fragments(&self) {
        self.fragments.borrow_mut().clear();
    }

    pub(crate) fn clear_fragments_and_fragment_cache(&self) {
        self.fragments.borrow_mut().clear();
        *self.cached_layout_result.borrow_mut() = None;
    }

    pub(crate) fn repair_style(&mut self, new_style: &ServoArc<ComputedValues>) {
        self.style = new_style.clone();
        for fragment in self.fragments.borrow_mut().iter_mut() {
            if let Some(base) = fragment.base() {
                base.repair_style(new_style);
            }
        }
    }

    #[expect(unused)]
    pub(crate) fn parent_box(&self) -> Option<LayoutBox> {
        self.parent_box.as_ref().and_then(WeakLayoutBox::upgrade)
    }

    pub(crate) fn add_damage(
        &self,
        element_damage: LayoutDamage,
        damage_from_children: LayoutDamage,
    ) -> LayoutDamage {
        self.clear_fragments_and_fragment_cache();

        if !element_damage.is_empty() ||
            damage_from_children.contains(LayoutDamage::RECOMPUTE_INLINE_CONTENT_SIZES)
        {
            *self.cached_inline_content_size.borrow_mut() = None;
        }

        let mut damage_for_parent = element_damage | damage_from_children;

        // When a block container has a mix of inline-level and block-level contents, the
        // inline-level ones are wrapped inside an anonymous block associated with the
        // block container. The anonymous block has an `auto` size, so its intrinsic
        // contribution depends on content, but it can't affect the intrinsic size of
        // ancestors if the block container is sized extrinsically.
        //
        // If the intrinsic contributions of this node depend on content, we will need to
        // clear the cached intrinsic sizes of the parent. But if the contributions are
        // purely extrinsic, then the intrinsic sizes of the ancestors won't be affected,
        // and we can keep the cache.
        damage_for_parent.set(
            LayoutDamage::RECOMPUTE_INLINE_CONTENT_SIZES,
            !element_damage.is_empty() ||
                (!self.base_fragment_info.is_anonymous() &&
                    self.outer_inline_content_sizes_depend_on_content
                        .load(Ordering::Relaxed)),
        );

        damage_for_parent
    }

    pub(crate) fn cached_same_formatting_context_block_if_applicable(
        &self,
        containing_block: &ContainingBlock,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
        has_inline_parent: bool,
    ) -> Option<Arc<BoxFragment>> {
        let mut cached_layout_result = self.cached_layout_result.borrow_mut();
        let Some(LayoutResultAndInputs::SameFormattingContextBlock(result)) =
            &mut *cached_layout_result
        else {
            return None;
        };

        if result.containing_block_size != containing_block.size ||
            result.containing_block_writing_mode != containing_block.style.writing_mode ||
            result.containing_block_justify_items !=
                containing_block.style.clone_justify_items().computed.0.0 ||
            result.collapsible_with_parent_start_margin != collapsible_with_parent_start_margin ||
            result.ignore_block_margins_for_stretch != ignore_block_margins_for_stretch ||
            result.has_inline_parent != has_inline_parent
        {
            return None;
        }

        let fragment = result.result.fragment.clone();
        {
            // Ideally when the final position doesn't change, this wouldn't be set, but we have
            // no way currently to track whether the final position wil differ from the one set in
            // the cached fragment. Final positioning is done in the containing block and depends
            // on things like margins and the size of siblings.
            fragment
                .base
                .set_status(FragmentStatus::PositionMaybeChanged);

            let mut origin = result.result.original_offset;
            if self.style.clone_position() == Position::Relative {
                origin += relative_adjustement(&self.style, containing_block)
                    .to_physical_vector(containing_block.style.writing_mode)
            }
            fragment.base.set_rect_origin(origin);
        }

        Some(fragment)
    }

    pub(crate) fn cache_same_formatting_context_block_layout(
        &self,
        containing_block: &ContainingBlock,
        collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
        ignore_block_margins_for_stretch: LogicalSides1D<bool>,
        has_inline_parent: bool,
        fragment: Arc<BoxFragment>,
    ) {
        let mut original_offset;
        {
            original_offset = fragment.content_rect().origin;
            if self.style.clone_position() == Position::Relative {
                original_offset -= relative_adjustement(&self.style, containing_block)
                    .to_physical_vector(containing_block.style.writing_mode)
            }
        }

        *self.cached_layout_result.borrow_mut() =
            Some(LayoutResultAndInputs::SameFormattingContextBlock(Box::new(
                SameFormattingContextBlockLayoutResultAndInputs {
                    result: SameFormattingContextBlockLayoutResult {
                        fragment,
                        original_offset,
                    },
                    containing_block_size: containing_block.size.clone(),
                    containing_block_writing_mode: containing_block.style.writing_mode,
                    containing_block_justify_items: containing_block
                        .style
                        .clone_justify_items()
                        .computed
                        .0
                        .0,
                    collapsible_with_parent_start_margin,
                    ignore_block_margins_for_stretch,
                    has_inline_parent,
                },
            )));
    }
}

impl Debug for LayoutBoxBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("LayoutBoxBase").finish()
    }
}

#[derive(MallocSizeOf)]
pub(crate) enum LayoutResultAndInputs {
    IndependentFormattingContext(Box<IndependentFormattingContextLayoutResultAndInputs>),
    SameFormattingContextBlock(Box<SameFormattingContextBlockLayoutResultAndInputs>),
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct IndependentFormattingContextLayoutResult {
    pub fragments: Vec<Fragment>,

    /// <https://drafts.csswg.org/css2/visudet.html#root-height>
    pub content_block_size: Au,

    /// If this layout is for a block container, this tracks the collapsable size
    /// of start and end margins and whether or not the block container collapsed through.
    pub collapsible_margins_in_children: CollapsedBlockMargins,

    /// The contents of a table may force it to become wider than what we would expect
    /// from 'width' and 'min-width'. This is the resulting inline content size,
    /// or None for non-table layouts.
    pub content_inline_size_for_table: Option<Au>,

    /// The offset of the last inflow baseline of this layout in the content area, if
    /// there was one. This is used to propagate baselines to the ancestors of `display:
    /// inline-block`.
    pub baselines: Baselines,

    /// Whether or not this layout depends on the containing block size.
    pub depends_on_block_constraints: bool,

    /// Additional information of this layout that could be used by Javascripts and devtools.
    pub specific_layout_info: Option<SpecificLayoutInfo>,
}

/// A collection of layout inputs and a cached layout result for an IndependentFormattingContext for
/// use in [`LayoutBoxBase`].
#[derive(MallocSizeOf)]
pub(crate) struct IndependentFormattingContextLayoutResultAndInputs {
    /// The [`IndependentFormattingContextLayoutResult`] for this layout.
    pub result: IndependentFormattingContextLayoutResult,

    /// The [`ContainingBlockSize`] to use for this box's contents, but not
    /// for the box itself.
    pub containing_block_for_children_size: ContainingBlockSize,

    /// A [`PositioningContext`] holding absolutely-positioned descendants
    /// collected during the layout of this box.
    pub positioning_context: PositioningContext,
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct SameFormattingContextBlockLayoutResult {
    #[conditional_malloc_size_of]
    pub fragment: Arc<BoxFragment>,
    original_offset: Point2D<Au, CSSPixel>,
}

/// A collection of layout inputs and a cached layout result for a SameFormattingContextBlock for
/// use in [`LayoutBoxBase`].
#[derive(MallocSizeOf)]
pub(crate) struct SameFormattingContextBlockLayoutResultAndInputs {
    pub result: SameFormattingContextBlockLayoutResult,
    /// The [`ContainingBlockSize`] used when this block was laid out.
    pub containing_block_size: ContainingBlockSize,
    /// The containing block's [`WritingMode`]  used when this block was laid out.
    pub containing_block_writing_mode: WritingMode,
    /// The containing block's `justify-items` [`AlignFlags`] used when this block was laid out.
    pub containing_block_justify_items: AlignFlags,
    /// Whether or not the margin in this block was collapsible with the parent's start margin
    /// when this block was laid out.
    collapsible_with_parent_start_margin: Option<CollapsibleWithParentStartMargin>,
    /// Whether or not block margins were ignored for stretch when this block was laid out.
    ignore_block_margins_for_stretch: LogicalSides1D<bool>,
    /// Whether or not this block had an inline parent.
    has_inline_parent: bool,
}
