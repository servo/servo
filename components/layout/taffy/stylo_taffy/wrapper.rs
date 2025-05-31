/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use style::properties::ComputedValues;

use super::convert;

/// A wrapper struct for anything that Deref's to a [`ComputedValues`], which
/// implements Taffy's layout traits and can used with Taffy's layout algorithms.
pub struct TaffyStyloStyle<T: Deref<Target = ComputedValues>> {
    pub style: T,
    pub is_compressible_replaced: bool,
}

impl<T: Deref<Target = ComputedValues>> TaffyStyloStyle<T> {
    pub fn new(style: T, is_compressible_replaced: bool) -> Self {
        Self {
            style,
            is_compressible_replaced,
        }
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::CoreStyle for TaffyStyloStyle<T> {
    #[inline]
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        convert::box_generation_mode(self.style.get_box().display)
    }

    #[inline]
    fn is_block(&self) -> bool {
        convert::is_block(self.style.get_box().display)
    }

    #[inline]
    fn is_compressible_replaced(&self) -> bool {
        self.is_compressible_replaced
    }

    #[inline]
    fn box_sizing(&self) -> taffy::BoxSizing {
        convert::box_sizing(self.style.get_position().box_sizing)
    }

    #[inline]
    fn overflow(&self) -> taffy::Point<taffy::Overflow> {
        let box_styles = self.style.get_box();
        taffy::Point {
            x: convert::overflow(box_styles.overflow_x),
            y: convert::overflow(box_styles.overflow_y),
        }
    }

    #[inline]
    fn scrollbar_width(&self) -> f32 {
        0.0
    }

    #[inline]
    fn position(&self) -> taffy::Position {
        convert::position(self.style.get_box().position)
    }

    #[inline]
    fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        let position_styles = self.style.get_position();
        taffy::Rect {
            left: convert::inset(&position_styles.left),
            right: convert::inset(&position_styles.right),
            top: convert::inset(&position_styles.top),
            bottom: convert::inset(&position_styles.bottom),
        }
    }

    #[inline]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.style.get_position();
        taffy::Size {
            width: convert::dimension(&position_styles.width),
            height: convert::dimension(&position_styles.height),
        }
    }

    #[inline]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.style.get_position();
        taffy::Size {
            width: convert::dimension(&position_styles.min_width),
            height: convert::dimension(&position_styles.min_height),
        }
    }

    #[inline]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.style.get_position();
        taffy::Size {
            width: convert::max_size_dimension(&position_styles.max_width),
            height: convert::max_size_dimension(&position_styles.max_height),
        }
    }

    #[inline]
    fn aspect_ratio(&self) -> Option<f32> {
        convert::aspect_ratio(self.style.get_position().aspect_ratio)
    }

    #[inline]
    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        let margin_styles = self.style.get_margin();
        taffy::Rect {
            left: convert::margin(&margin_styles.margin_left),
            right: convert::margin(&margin_styles.margin_right),
            top: convert::margin(&margin_styles.margin_top),
            bottom: convert::margin(&margin_styles.margin_bottom),
        }
    }

    #[inline]
    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        let padding_styles = self.style.get_padding();
        taffy::Rect {
            left: convert::length_percentage(&padding_styles.padding_left.0),
            right: convert::length_percentage(&padding_styles.padding_right.0),
            top: convert::length_percentage(&padding_styles.padding_top.0),
            bottom: convert::length_percentage(&padding_styles.padding_bottom.0),
        }
    }

    #[inline]
    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        let border_styles = self.style.get_border();
        taffy::Rect {
            left: taffy::LengthPercentage::length(border_styles.border_left_width.to_f32_px()),
            right: taffy::LengthPercentage::length(border_styles.border_right_width.to_f32_px()),
            top: taffy::LengthPercentage::length(border_styles.border_top_width.to_f32_px()),
            bottom: taffy::LengthPercentage::length(border_styles.border_bottom_width.to_f32_px()),
        }
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridContainerStyle for TaffyStyloStyle<T> {
    type TemplateTrackList<'a>
        = Vec<taffy::TrackSizingFunction>
    where
        Self: 'a;
    type AutoTrackList<'a>
        = Vec<taffy::NonRepeatedTrackSizingFunction>
    where
        Self: 'a;

    #[inline]
    fn grid_template_rows(&self) -> Self::TemplateTrackList<'_> {
        convert::grid_template_tracks(&self.style.get_position().grid_template_rows)
    }

    #[inline]
    fn grid_template_columns(&self) -> Self::TemplateTrackList<'_> {
        convert::grid_template_tracks(&self.style.get_position().grid_template_columns)
    }

    #[inline]
    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        convert::grid_auto_tracks(&self.style.get_position().grid_auto_rows)
    }

    #[inline]
    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        convert::grid_auto_tracks(&self.style.get_position().grid_auto_columns)
    }

    #[inline]
    fn grid_auto_flow(&self) -> taffy::GridAutoFlow {
        convert::grid_auto_flow(self.style.get_position().grid_auto_flow)
    }

    #[inline]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        let position_styles = self.style.get_position();
        taffy::Size {
            width: convert::gap(&position_styles.column_gap),
            height: convert::gap(&position_styles.row_gap),
        }
    }

    #[inline]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        convert::content_alignment(self.style.get_position().align_content.0)
    }

    #[inline]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        convert::content_alignment(self.style.get_position().justify_content.0)
    }

    #[inline]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.style.get_position().align_items.0)
    }

    #[inline]
    fn justify_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.style.get_position().justify_items.computed.0)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridItemStyle for TaffyStyloStyle<T> {
    #[inline]
    fn grid_row(&self) -> taffy::Line<taffy::GridPlacement> {
        let position_styles = self.style.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_row_start),
            end: convert::grid_line(&position_styles.grid_row_end),
        }
    }

    #[inline]
    fn grid_column(&self) -> taffy::Line<taffy::GridPlacement> {
        let position_styles = self.style.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_column_start),
            end: convert::grid_line(&position_styles.grid_column_end),
        }
    }

    #[inline]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.style.get_position().align_self.0.0)
    }

    #[inline]
    fn justify_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.style.get_position().justify_self.0.0)
    }
}
