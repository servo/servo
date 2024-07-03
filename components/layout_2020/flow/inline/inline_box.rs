/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::vec::IntoIter;

use app_units::Au;
use fonts::FontMetrics;
use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;

use super::{inline_container_needs_strut, InlineContainerState, InlineContainerStateFlags};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::NodeAndStyleInfo;
use crate::fragment_tree::BaseFragmentInfo;
use crate::style_ext::{ComputedValuesExt, PaddingBorderMargin};
use crate::ContainingBlock;

#[derive(Debug, Serialize)]
pub(crate) struct InlineBox {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    /// The identifier of this inline box in the containing [`InlineFormattingContext`].
    pub(super) identifier: InlineBoxIdentifier,
    pub is_first_fragment: bool,
    pub is_last_fragment: bool,
    /// The index of the default font in the [`InlineFormattingContext`]'s font metrics store.
    /// This is initialized during IFC shaping.
    pub default_font_index: Option<usize>,
}

impl InlineBox {
    pub(crate) fn new<'dom, Node: NodeExt<'dom>>(info: &NodeAndStyleInfo<Node>) -> Self {
        Self {
            base_fragment_info: info.into(),
            style: info.style.clone(),
            // This will be assigned later, when the box is actually added to the IFC.
            identifier: InlineBoxIdentifier::default(),
            is_first_fragment: true,
            is_last_fragment: false,
            default_font_index: None,
        }
    }

    pub(crate) fn split_around_block(&self) -> Self {
        Self {
            style: self.style.clone(),
            is_first_fragment: false,
            is_last_fragment: false,
            ..*self
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct InlineBoxes {
    /// A collection of all inline boxes in a particular [`InlineFormattingContext`].
    inline_boxes: Vec<ArcRefCell<InlineBox>>,

    /// A list of tokens that represent the actual tree of inline boxes, while allowing
    /// easy traversal forward and backwards through the tree. This structure is also
    /// stored in the [`InlineFormattingContext::inline_items`], but this version is
    /// faster to iterate.
    inline_box_tree: Vec<InlineBoxTreePathToken>,
}

impl InlineBoxes {
    pub(super) fn len(&self) -> usize {
        self.inline_boxes.len()
    }

    pub(super) fn get(&self, identifier: &InlineBoxIdentifier) -> ArcRefCell<InlineBox> {
        self.inline_boxes[identifier.index_in_inline_boxes as usize].clone()
    }

    pub(super) fn end_inline_box(&mut self, identifier: InlineBoxIdentifier) {
        self.inline_box_tree
            .push(InlineBoxTreePathToken::End(identifier));
    }

    pub(super) fn start_inline_box(&mut self, mut inline_box: InlineBox) -> InlineBoxIdentifier {
        assert!(self.inline_boxes.len() <= u32::MAX as usize);
        assert!(self.inline_box_tree.len() <= u32::MAX as usize);

        let index_in_inline_boxes = self.inline_boxes.len() as u32;
        let index_of_start_in_tree = self.inline_box_tree.len() as u32;

        let identifier = InlineBoxIdentifier {
            index_of_start_in_tree,
            index_in_inline_boxes,
        };
        inline_box.identifier = identifier;

        self.inline_boxes.push(ArcRefCell::new(inline_box));
        self.inline_box_tree
            .push(InlineBoxTreePathToken::Start(identifier));
        identifier
    }

    pub(super) fn get_path(
        &self,
        from: Option<InlineBoxIdentifier>,
        to: InlineBoxIdentifier,
    ) -> IntoIter<InlineBoxTreePathToken> {
        if from == Some(to) {
            return Vec::new().into_iter();
        }

        let mut from_index = match from {
            Some(InlineBoxIdentifier {
                index_of_start_in_tree,
                ..
            }) => index_of_start_in_tree as usize,
            None => 0,
        };
        let mut to_index = to.index_of_start_in_tree as usize;
        let is_reversed = to_index < from_index;

        // Do not include the first or final token, depending on direction. These can be equal
        // if we are starting or going to the the root of the inline formatting context, in which
        // case we don't want to adjust.
        if to_index > from_index && from.is_some() {
            from_index += 1;
        } else if to_index < from_index {
            to_index += 1;
        }

        let mut path = Vec::with_capacity(from_index.abs_diff(to_index));
        let min = from_index.min(to_index);
        let max = from_index.max(to_index);

        for token in &self.inline_box_tree[min..=max] {
            // Skip useless recursion into inline boxes; we are looking for a direct path.
            if Some(&token.reverse()) == path.last() {
                path.pop();
            } else {
                path.push(*token);
            }
        }

        if is_reversed {
            path.reverse();
            for token in path.iter_mut() {
                *token = token.reverse();
            }
        }

        path.into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) enum InlineBoxTreePathToken {
    Start(InlineBoxIdentifier),
    End(InlineBoxIdentifier),
}

impl InlineBoxTreePathToken {
    fn reverse(&self) -> Self {
        match self {
            Self::Start(index) => Self::End(*index),
            Self::End(index) => Self::Start(*index),
        }
    }
}

/// An identifier for a particular [`InlineBox`] to be used to fetch it from an [`InlineBoxes`]
/// store of inline boxes.
///
/// [`u32`] is used for the index, in order to save space. The value refers to the token
/// in the start tree data structure which can be fetched to find the actual index of
/// of the [`InlineBox`] in [`InlineBoxes::inline_boxes`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct InlineBoxIdentifier {
    pub index_of_start_in_tree: u32,
    pub index_in_inline_boxes: u32,
}

pub(super) struct InlineBoxContainerState {
    /// The container state common to both [`InlineBox`] and the root of the
    /// [`InlineFormattingContext`].
    pub base: InlineContainerState,

    /// The [`InlineBoxIdentifier`] of this inline container state. If this is the root
    /// the identifier is [`None`].
    pub identifier: InlineBoxIdentifier,

    /// The [`BaseFragmentInfo`] of the [`InlineBox`] that this state tracks.
    pub base_fragment_info: BaseFragmentInfo,

    /// The [`PaddingBorderMargin`] of the [`InlineBox`] that this state tracks.
    pub pbm: PaddingBorderMargin,

    /// Whether this is the last fragment of this InlineBox. This may not be the case if
    /// the InlineBox is split due to an block-in-inline-split and this is not the last of
    /// that split.
    pub is_last_fragment: bool,
}

impl InlineBoxContainerState {
    pub(super) fn new(
        inline_box: &InlineBox,
        containing_block: &ContainingBlock,
        layout_context: &LayoutContext,
        parent_container: &InlineContainerState,
        is_last_fragment: bool,
        font_metrics: Option<&FontMetrics>,
    ) -> Self {
        let style = inline_box.style.clone();
        let pbm = style.padding_border_margin(containing_block);

        let mut flags = InlineContainerStateFlags::empty();
        if inline_container_needs_strut(&style, layout_context, Some(&pbm)) {
            flags.insert(InlineContainerStateFlags::CREATE_STRUT);
        }

        Self {
            base: InlineContainerState::new(
                style,
                flags,
                Some(parent_container),
                parent_container.text_decoration_line,
                font_metrics,
            ),
            identifier: inline_box.identifier,
            base_fragment_info: inline_box.base_fragment_info,
            pbm,
            is_last_fragment,
        }
    }

    pub(super) fn calculate_space_above_baseline(&self) -> Au {
        let (ascent, descent, line_gap) = (
            self.base.font_metrics.ascent,
            self.base.font_metrics.descent,
            self.base.font_metrics.line_gap,
        );
        let leading = line_gap - (ascent + descent);
        leading.scale_by(0.5) + ascent
    }
}
