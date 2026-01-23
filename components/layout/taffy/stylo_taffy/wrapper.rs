/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use style::properties::ComputedValues;
use style::values::CustomIdent;
use style::values::computed::{BorderSideWidth, GridTemplateAreas, LengthPercentage};
use style::values::generics::grid::{TrackListValue, TrackRepeat, TrackSize};
use style::values::specified::BorderStyle;
use style::values::specified::position::NamedArea;
use style::{Atom, OwnedSlice};
use taffy::prelude::TaffyAuto;

use super::{convert, stylo};

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
    type CustomIdent = Atom;

    #[inline]
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        convert::box_generation_mode(self.style.get_box().display)
    }

    #[inline]
    fn direction(&self) -> taffy::Direction {
        convert::direction(self.style.clone_direction())
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
        // Taffy doesn't support static nor sticky positionings, they are treated
        // as relative. As a workaround, ignore the insets.
        if matches!(
            self.style.get_box().position,
            stylo::Position::Static | stylo::Position::Sticky
        ) {
            return taffy::Rect {
                left: taffy::LengthPercentageAuto::AUTO,
                right: taffy::LengthPercentageAuto::AUTO,
                top: taffy::LengthPercentageAuto::AUTO,
                bottom: taffy::LengthPercentageAuto::AUTO,
            };
        }
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
        let border = self.style.get_border();
        let resolve = |width: &BorderSideWidth, style: BorderStyle| {
            taffy::LengthPercentage::length(if style.none_or_hidden() {
                0.0
            } else {
                width.0.to_f32_px()
            })
        };
        taffy::Rect {
            left: resolve(&border.border_left_width, border.border_left_style),
            right: resolve(&border.border_right_width, border.border_right_style),
            top: resolve(&border.border_top_width, border.border_top_style),
            bottom: resolve(&border.border_bottom_width, border.border_bottom_style),
        }
    }
}

type SliceMapIter<'a, Input, Output> =
    core::iter::Map<core::slice::Iter<'a, Input>, for<'c> fn(&'c Input) -> Output>;
type SliceMapRefIter<'a, Input, Output> =
    core::iter::Map<core::slice::Iter<'a, Input>, for<'c> fn(&'c Input) -> &'c Output>;

// Line name iterator type aliases
type LineNameSetIter<'a> = SliceMapRefIter<'a, CustomIdent, Atom>;
type LineNameIter<'a> = core::iter::Map<
    core::slice::Iter<'a, OwnedSlice<CustomIdent>>,
    fn(&OwnedSlice<CustomIdent>) -> LineNameSetIter<'_>,
>;

#[derive(Clone)]
pub struct StyloLineNameIter<'a>(LineNameIter<'a>);
impl<'a> StyloLineNameIter<'a> {
    fn new(names: &'a OwnedSlice<OwnedSlice<CustomIdent>>) -> Self {
        Self(names.iter().map(|names| names.iter().map(|ident| &ident.0)))
    }
}
impl<'a> Iterator for StyloLineNameIter<'a> {
    type Item = core::iter::Map<core::slice::Iter<'a, CustomIdent>, fn(&CustomIdent) -> &Atom>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for StyloLineNameIter<'_> {}
impl<'a> taffy::TemplateLineNames<'a, Atom> for StyloLineNameIter<'a> {
    type LineNameSet<'b>
        = SliceMapRefIter<'b, CustomIdent, Atom>
    where
        Self: 'b;
}

pub struct RepetitionWrapper<'a>(&'a TrackRepeat<LengthPercentage, i32>);

impl taffy::GenericRepetition for RepetitionWrapper<'_> {
    type CustomIdent = Atom;

    type RepetitionTrackList<'a>
        = SliceMapIter<'a, stylo::TrackSize<LengthPercentage>, taffy::TrackSizingFunction>
    where
        Self: 'a;

    type TemplateLineNames<'a>
        = StyloLineNameIter<'a>
    where
        Self: 'a;

    fn count(&self) -> taffy::RepetitionCount {
        convert::track_repeat(self.0.count)
    }

    fn tracks(&self) -> Self::RepetitionTrackList<'_> {
        self.0.track_sizes.iter().map(convert::track_size)
    }

    fn lines_names(&self) -> Self::TemplateLineNames<'_> {
        StyloLineNameIter::new(&self.0.line_names)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridContainerStyle for TaffyStyloStyle<T> {
    type Repetition<'a>
        = RepetitionWrapper<'a>
    where
        Self: 'a;

    type TemplateTrackList<'a>
        = core::iter::Map<
        core::slice::Iter<'a, TrackListValue<LengthPercentage, i32>>,
        fn(
            &'a TrackListValue<LengthPercentage, i32>,
        ) -> taffy::GenericGridTemplateComponent<Atom, RepetitionWrapper<'a>>,
    >
    where
        Self: 'a;

    type AutoTrackList<'a>
        = SliceMapIter<'a, TrackSize<LengthPercentage>, taffy::TrackSizingFunction>
    where
        Self: 'a;

    type TemplateLineNames<'a>
        = StyloLineNameIter<'a>
    where
        Self: 'a;
    type GridTemplateAreas<'a>
        = SliceMapIter<'a, NamedArea, taffy::GridTemplateArea<Atom>>
    where
        Self: 'a;

    #[inline]
    fn grid_template_rows(&self) -> Option<Self::TemplateTrackList<'_>> {
        match &self.style.get_position().grid_template_rows {
            stylo::GenericGridTemplateComponent::None => None,
            stylo::GenericGridTemplateComponent::TrackList(list) => {
                Some(list.values.iter().map(|track| match track {
                    stylo::TrackListValue::TrackSize(size) => {
                        taffy::GenericGridTemplateComponent::Single(convert::track_size(size))
                    },
                    stylo::TrackListValue::TrackRepeat(repeat) => {
                        taffy::GenericGridTemplateComponent::Repeat(RepetitionWrapper(repeat))
                    },
                }))
            },

            // TODO: Implement subgrid and masonry
            stylo::GenericGridTemplateComponent::Subgrid(_) => None,
            stylo::GenericGridTemplateComponent::Masonry => None,
        }
    }

    #[inline]
    fn grid_template_columns(&self) -> Option<Self::TemplateTrackList<'_>> {
        match &self.style.get_position().grid_template_columns {
            stylo::GenericGridTemplateComponent::None => None,
            stylo::GenericGridTemplateComponent::TrackList(list) => {
                Some(list.values.iter().map(|track| match track {
                    stylo::TrackListValue::TrackSize(size) => {
                        taffy::GenericGridTemplateComponent::Single(convert::track_size(size))
                    },
                    stylo::TrackListValue::TrackRepeat(repeat) => {
                        taffy::GenericGridTemplateComponent::Repeat(RepetitionWrapper(repeat))
                    },
                }))
            },

            // TODO: Implement subgrid and masonry
            stylo::GenericGridTemplateComponent::Subgrid(_) => None,
            stylo::GenericGridTemplateComponent::Masonry => None,
        }
    }

    #[inline]
    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        self.style
            .get_position()
            .grid_auto_rows
            .0
            .iter()
            .map(convert::track_size)
    }

    #[inline]
    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        self.style
            .get_position()
            .grid_auto_columns
            .0
            .iter()
            .map(convert::track_size)
    }

    fn grid_template_areas(&self) -> Option<Self::GridTemplateAreas<'_>> {
        match &self.style.get_position().grid_template_areas {
            GridTemplateAreas::Areas(areas) => {
                Some(areas.0.areas.iter().map(|area| taffy::GridTemplateArea {
                    name: area.name.clone(),
                    row_start: area.rows.start as u16,
                    row_end: area.rows.end as u16,
                    column_start: area.columns.start as u16,
                    column_end: area.columns.end as u16,
                }))
            },
            GridTemplateAreas::None => None,
        }
    }

    fn grid_template_column_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        match &self.style.get_position().grid_template_columns {
            stylo::GenericGridTemplateComponent::None => None,
            stylo::GenericGridTemplateComponent::TrackList(list) => {
                Some(StyloLineNameIter::new(&list.line_names))
            },
            // TODO: Implement subgrid and masonry
            stylo::GenericGridTemplateComponent::Subgrid(_) => None,
            stylo::GenericGridTemplateComponent::Masonry => None,
        }
    }

    fn grid_template_row_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        match &self.style.get_position().grid_template_rows {
            stylo::GenericGridTemplateComponent::None => None,
            stylo::GenericGridTemplateComponent::TrackList(list) => {
                Some(StyloLineNameIter::new(&list.line_names))
            },
            // TODO: Implement subgrid and masonry
            stylo::GenericGridTemplateComponent::Subgrid(_) => None,
            stylo::GenericGridTemplateComponent::Masonry => None,
        }
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
        convert::content_alignment(self.style.get_position().align_content)
    }

    #[inline]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        convert::content_alignment(self.style.get_position().justify_content)
    }

    #[inline]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.style.get_position().align_items.0)
    }

    #[inline]
    fn justify_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.style.get_position().justify_items.computed.0.0)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridItemStyle for TaffyStyloStyle<T> {
    #[inline]
    fn grid_row(&self) -> taffy::Line<taffy::GridPlacement<Atom>> {
        let position_styles = self.style.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_row_start),
            end: convert::grid_line(&position_styles.grid_row_end),
        }
    }

    #[inline]
    fn grid_column(&self) -> taffy::Line<taffy::GridPlacement<Atom>> {
        let position_styles = self.style.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_column_start),
            end: convert::grid_line(&position_styles.grid_column_end),
        }
    }

    #[inline]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.style.get_position().align_self.0)
    }

    #[inline]
    fn justify_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.style.get_position().justify_self.0)
    }
}
