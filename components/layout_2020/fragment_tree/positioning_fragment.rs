/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx_traits::print_tree::PrintTree;
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;

use super::{BaseFragment, BaseFragmentInfo, Fragment};
use crate::cell::ArcRefCell;
use crate::geom::{LogicalRect, PhysicalRect};

/// Can contain child fragments with relative coordinates, but does not contribute to painting
/// itself. [`PositioningFragments`] may be completely anonymous, or just non-painting Fragments
/// generated by boxes.
#[derive(Serialize)]
pub(crate) struct PositioningFragment {
    pub base: BaseFragment,
    pub rect: LogicalRect<Length>,
    pub children: Vec<ArcRefCell<Fragment>>,
    pub writing_mode: WritingMode,

    /// The scrollable overflow of this anonymous fragment's children.
    pub scrollable_overflow: PhysicalRect<Length>,

    /// If this fragment was created with a style, the style of the fragment.
    #[serde(skip_serializing)]
    pub style: Option<ServoArc<ComputedValues>>,
}

impl PositioningFragment {
    pub fn new_anonymous(
        rect: LogicalRect<Length>,
        children: Vec<Fragment>,
        mode: WritingMode,
    ) -> Self {
        Self::new_with_base_fragment(BaseFragment::anonymous(), None, rect, children, mode)
    }

    pub fn new_empty(
        base_fragment_info: BaseFragmentInfo,
        rect: LogicalRect<Length>,
        style: ServoArc<ComputedValues>,
    ) -> Self {
        let writing_mode = style.writing_mode;
        Self::new_with_base_fragment(
            base_fragment_info.into(),
            Some(style),
            rect,
            Vec::new(),
            writing_mode,
        )
    }

    fn new_with_base_fragment(
        base: BaseFragment,
        style: Option<ServoArc<ComputedValues>>,
        rect: LogicalRect<Length>,
        children: Vec<Fragment>,
        mode: WritingMode,
    ) -> Self {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        let content_origin = rect.start_corner.to_physical(mode);
        let scrollable_overflow = children.iter().fold(PhysicalRect::zero(), |acc, child| {
            acc.union(
                &child
                    .scrollable_overflow(&containing_block)
                    .translate(content_origin.to_vector()),
            )
        });
        PositioningFragment {
            base,
            style,
            rect,
            children: children.into_iter().map(ArcRefCell::new).collect(),
            writing_mode: mode,
            scrollable_overflow,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "PositioningFragment\
                \nbase={:?}\
                \nrect={:?}\
                \nscrollable_overflow={:?}",
            self.base, self.rect, self.scrollable_overflow
        ));

        for child in &self.children {
            child.borrow().print(tree);
        }
        tree.end_level();
    }
}