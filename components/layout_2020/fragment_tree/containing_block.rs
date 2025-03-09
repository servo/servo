/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::computed_values::position::T as ComputedPosition;

use crate::fragment_tree::Fragment;
use crate::geom::{PhysicalRect, PhysicalVec};

/// A data structure used to track the containing block when recursing
/// through the Fragment tree. It tracks the three types of containing
/// blocks (for all descendants, for absolute and fixed position
/// descendants, and for fixed position descendants).
pub(crate) struct ContainingBlockManager<'a, T> {
    /// The containing block for all non-absolute descendants. "...if the element's
    /// position is 'relative' or 'static', the containing block is formed by the
    /// content edge of the nearest block container ancestor box." This is also
    /// the case for 'position: sticky' elements.
    /// <https://www.w3.org/TR/CSS2/visudet.html#containing-block-details>
    pub for_non_absolute_descendants: &'a T,

    /// The containing block for absolute descendants. "If the element has
    /// 'position: absolute', the containing block is
    /// established by the nearest ancestor with a 'position' of 'absolute',
    /// 'relative' or 'fixed', in the following way:
    ///   1. In the case that the ancestor is an inline element, the containing
    ///      block is the bounding box around the padding boxes of the first and the
    ///      last inline boxes generated for that element. In CSS 2.1, if the inline
    ///      element is split across multiple lines, the containing block is
    ///      undefined.
    ///   2. Otherwise, the containing block is formed by the padding edge of the
    ///      ancestor.
    ///
    /// <https://www.w3.org/TR/CSS2/visudet.html#containing-block-details>
    /// If the ancestor forms a containing block for all descendants (see below),
    /// this value will be None and absolute descendants will use the containing
    /// block for fixed descendants.
    pub for_absolute_descendants: Option<&'a T>,

    /// The containing block for fixed and absolute descendants.
    /// "For elements whose layout is governed by the CSS box model, any value
    /// other than none for the transform property also causes the element to
    /// establish a containing block for all descendants. Its padding box will be
    /// used to layout for all of its absolute-position descendants,
    /// fixed-position descendants, and descendant fixed background attachments."
    /// <https://w3c.github.io/csswg-drafts/css-transforms-1/#containing-block-for-all-descendants>
    /// See `ComputedValues::establishes_containing_block_for_all_descendants`
    /// for a list of conditions where an element forms a containing block for
    /// all descendants.
    pub for_absolute_and_fixed_descendants: &'a T,
}

impl<'a, T> ContainingBlockManager<'a, T> {
    pub(crate) fn get_containing_block_for_fragment(&self, fragment: &Fragment) -> &T {
        if let Fragment::Box(box_fragment) = fragment {
            match box_fragment.borrow().style.clone_position() {
                ComputedPosition::Fixed => self.for_absolute_and_fixed_descendants,
                ComputedPosition::Absolute => self
                    .for_absolute_descendants
                    .unwrap_or(self.for_absolute_and_fixed_descendants),
                _ => self.for_non_absolute_descendants,
            }
        } else {
            self.for_non_absolute_descendants
        }
    }

    pub(crate) fn new_for_non_absolute_descendants(
        &self,
        for_non_absolute_descendants: &'a T,
    ) -> Self {
        ContainingBlockManager {
            for_non_absolute_descendants,
            for_absolute_descendants: self.for_absolute_descendants,
            for_absolute_and_fixed_descendants: self.for_absolute_and_fixed_descendants,
        }
    }

    pub(crate) fn new_for_absolute_descendants(
        &self,
        for_non_absolute_descendants: &'a T,
        for_absolute_descendants: &'a T,
    ) -> Self {
        ContainingBlockManager {
            for_non_absolute_descendants,
            for_absolute_descendants: Some(for_absolute_descendants),
            for_absolute_and_fixed_descendants: self.for_absolute_and_fixed_descendants,
        }
    }

    pub(crate) fn new_for_absolute_and_fixed_descendants(
        &self,
        for_non_absolute_descendants: &'a T,
        for_absolute_and_fixed_descendants: &'a T,
    ) -> Self {
        ContainingBlockManager {
            for_non_absolute_descendants,
            for_absolute_descendants: None,
            for_absolute_and_fixed_descendants,
        }
    }
}

/// Containing block rect with additional information required for a query.
pub(crate) struct ContainingBlockQueryInfo {
    /// Containing block rect, that bounds the children.
    pub(crate) rect: PhysicalRect<Au>,

    /// The scroll offset of the containing block has.
    pub(crate) scroll_offset: PhysicalVec<Au>,
}

impl ContainingBlockQueryInfo {
    /// Transform child's rectangle according to this containing block transformation.
    /// TODO: this is supposed to handle CSS transform but it is not happening.
    pub(crate) fn transform_rect_relative_to_self(
        &self,
        rect: PhysicalRect<Au>,
    ) -> PhysicalRect<Au> {
        rect.translate(self.rect.origin.to_vector() + self.scroll_offset)
    }

    /// New containing block that is a child of this containing block with
    /// ancestor's transformation applied.
    pub(crate) fn new_relative_transformed_child(
        &self,
        rect: PhysicalRect<Au>,
        scroll_offset: PhysicalVec<Au>,
    ) -> ContainingBlockQueryInfo {
        ContainingBlockQueryInfo {
            rect: self.transform_rect_relative_to_self(rect),
            scroll_offset,
        }
    }
}
