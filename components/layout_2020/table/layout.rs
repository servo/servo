/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::cmp::Ordering;
use std::mem;
use std::ops::Range;

use app_units::Au;
use log::warn;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use servo_arc::Arc;
use style::Zero;
use style::computed_values::border_collapse::T as BorderCollapse;
use style::computed_values::box_sizing::T as BoxSizing;
use style::computed_values::caption_side::T as CaptionSide;
use style::computed_values::empty_cells::T as EmptyCells;
use style::computed_values::position::T as Position;
use style::computed_values::table_layout::T as TableLayoutMode;
use style::computed_values::visibility::T as Visibility;
use style::properties::ComputedValues;
use style::values::computed::{
    BorderStyle, LengthPercentage as ComputedLengthPercentage, Percentage,
};
use style::values::generics::box_::{GenericVerticalAlign as VerticalAlign, VerticalAlignKeyword};

use super::{
    ArcRefCell, CollapsedBorder, CollapsedBorderLine, SpecificTableGridInfo, Table, TableCaption,
    TableLayoutStyle, TableSlot, TableSlotCell, TableSlotCoordinates, TableTrack, TableTrackGroup,
};
use crate::context::LayoutContext;
use crate::formatting_contexts::Baselines;
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, ExtraBackground, Fragment, FragmentFlags,
    PositioningFragment, SpecificLayoutInfo,
};
use crate::geom::{
    LogicalRect, LogicalSides, LogicalSides1D, LogicalVec2, PhysicalPoint, PhysicalRect,
    PhysicalSides, PhysicalVec, Size, SizeConstraint, ToLogical, ToLogicalWithContainingBlock,
};
use crate::layout_box_base::CacheableLayoutResult;
use crate::positioned::{PositioningContext, PositioningContextLength, relative_adjustement};
use crate::sizing::{ComputeInlineContentSizes, ContentSizes, InlineContentSizesResult};
use crate::style_ext::{
    BorderStyleColor, Clamp, ComputedValuesExt, LayoutStyle, PaddingBorderMargin,
};
use crate::{
    ConstraintSpace, ContainingBlock, ContainingBlockSize, IndefiniteContainingBlock, WritingMode,
};

/// A result of a final or speculative layout of a single cell in
/// the table. Note that this is only done for slots that are not
/// covered by spans or empty.
struct CellLayout {
    layout: CacheableLayoutResult,
    padding: LogicalSides<Au>,
    border: LogicalSides<Au>,
    positioning_context: PositioningContext,
}

impl CellLayout {
    fn ascent(&self) -> Au {
        self.layout
            .baselines
            .first
            .unwrap_or(self.layout.content_block_size)
    }

    /// The block size of this laid out cell including its border and padding.
    fn outer_block_size(&self) -> Au {
        self.layout.content_block_size + self.border.block_sum() + self.padding.block_sum()
    }

    /// Whether the cell has no in-flow or out-of-flow contents, other than collapsed whitespace.
    /// Note this logic differs from 'empty-cells', which counts abspos contents as empty.
    fn is_empty(&self) -> bool {
        self.layout.fragments.is_empty()
    }

    /// Whether the cell is considered empty for the purpose of the 'empty-cells' property.
    fn is_empty_for_empty_cells(&self) -> bool {
        self.layout
            .fragments
            .iter()
            .all(|fragment| matches!(fragment, Fragment::AbsoluteOrFixedPositioned(_)))
    }
}

/// Information stored during the layout of rows.
#[derive(Clone, Debug, Default)]
struct RowLayout {
    constrained: bool,
    has_cell_with_span_greater_than_one: bool,
    percent: Percentage,
}

/// Information stored during the layout of columns.
#[derive(Clone, Debug, Default)]
struct ColumnLayout {
    constrained: bool,
    has_originating_cells: bool,
    content_sizes: ContentSizes,
    percentage: Option<Percentage>,
}

fn max_two_optional_percentages(
    a: Option<Percentage>,
    b: Option<Percentage>,
) -> Option<Percentage> {
    match (a, b) {
        (Some(a), Some(b)) => Some(Percentage(a.0.max(b.0))),
        _ => a.or(b),
    }
}

impl ColumnLayout {
    fn incorporate_cell_measure(&mut self, cell_measure: &CellOrTrackMeasure) {
        self.content_sizes.max_assign(cell_measure.content_sizes);
        self.percentage = max_two_optional_percentages(self.percentage, cell_measure.percentage);
    }
}

impl CollapsedBorder {
    fn new(style_color: BorderStyleColor, width: Au) -> Self {
        Self { style_color, width }
    }

    fn from_layout_style(
        layout_style: &LayoutStyle,
        writing_mode: WritingMode,
    ) -> LogicalSides<Self> {
        let border_style_color = layout_style.style().border_style_color(writing_mode);
        let border_width = layout_style.border_width(writing_mode);
        LogicalSides {
            inline_start: Self::new(border_style_color.inline_start, border_width.inline_start),
            inline_end: Self::new(border_style_color.inline_end, border_width.inline_end),
            block_start: Self::new(border_style_color.block_start, border_width.block_start),
            block_end: Self::new(border_style_color.block_end, border_width.block_end),
        }
    }

    fn max_assign(&mut self, other: &Self) {
        if *self < *other {
            *self = other.clone();
        }
    }

    fn max_assign_to_slice(&self, slice: &mut [CollapsedBorder]) {
        for collapsed_border in slice {
            collapsed_border.max_assign(self)
        }
    }

    fn hide(&mut self) {
        self.style_color = BorderStyleColor::hidden();
        self.width = Au::zero();
    }
}

/// <https://drafts.csswg.org/css-tables/#border-specificity>
/// > Given two borders styles, the border style having the most specificity is the border style which…
/// > 1. … has the value "hidden" as border-style, if only one does
/// > 2. … has the biggest border-width, once converted into css pixels
/// > 3. … has the border-style which comes first in the following list:
/// >    double, solid, dashed, dotted, ridge, outset, groove, inset, none
impl PartialOrd for CollapsedBorder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let is_hidden = |border: &Self| border.style_color.style == BorderStyle::Hidden;
        let style_specificity = |border: &Self| match border.style_color.style {
            BorderStyle::None => 0,
            BorderStyle::Inset => 1,
            BorderStyle::Groove => 2,
            BorderStyle::Outset => 3,
            BorderStyle::Ridge => 4,
            BorderStyle::Dotted => 5,
            BorderStyle::Dashed => 6,
            BorderStyle::Solid => 7,
            BorderStyle::Double => 8,
            BorderStyle::Hidden => 9,
        };
        let candidate = (is_hidden(self).cmp(&is_hidden(other)))
            .then_with(|| self.width.cmp(&other.width))
            .then_with(|| style_specificity(self).cmp(&style_specificity(other)));
        if !candidate.is_eq() || self.style_color.color == other.style_color.color {
            Some(candidate)
        } else {
            None
        }
    }
}

impl Eq for CollapsedBorder {}

type CollapsedBorders = LogicalVec2<Vec<CollapsedBorderLine>>;

/// A helper struct that performs the layout of the box tree version
/// of a table into the fragment tree version. This implements
/// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>
pub(crate) struct TableLayout<'a> {
    table: &'a Table,
    pbm: PaddingBorderMargin,
    rows: Vec<RowLayout>,
    columns: Vec<ColumnLayout>,
    cell_measures: Vec<Vec<LogicalVec2<CellOrTrackMeasure>>>,
    /// The calculated width of the table, including space for the grid and also for any
    /// captions.
    table_width: Au,
    /// The table width minus the total horizontal border spacing (if any). This is the
    /// width that we will be able to allocate to the columns.
    assignable_width: Au,
    final_table_height: Au,
    distributed_column_widths: Vec<Au>,
    row_sizes: Vec<Au>,
    /// The accumulated baseline of each row, relative to the top of the row.
    row_baselines: Vec<Au>,
    cells_laid_out: Vec<Vec<Option<CellLayout>>>,
    basis_for_cell_padding_percentage: Au,
    /// Information about collapsed borders.
    collapsed_borders: Option<CollapsedBorders>,
    is_in_fixed_mode: bool,
}

#[derive(Clone, Debug)]
struct CellOrTrackMeasure {
    content_sizes: ContentSizes,
    percentage: Option<Percentage>,
}

impl Zero for CellOrTrackMeasure {
    fn zero() -> Self {
        Self {
            content_sizes: ContentSizes::zero(),
            percentage: None,
        }
    }

    fn is_zero(&self) -> bool {
        self.content_sizes.is_zero() && self.percentage.is_none()
    }
}

impl<'a> TableLayout<'a> {
    fn new(table: &'a Table) -> TableLayout<'a> {
        // The CSSWG resolved that only `inline-size: auto` can prevent fixed table mode.
        // <https://github.com/w3c/csswg-drafts/issues/10937#issuecomment-2669150397>
        let style = &table.style;
        let is_in_fixed_mode = style.get_table().table_layout == TableLayoutMode::Fixed &&
            !style.box_size(style.writing_mode).inline.is_initial();
        Self {
            table,
            pbm: PaddingBorderMargin::zero(),
            rows: Vec::new(),
            columns: Vec::new(),
            cell_measures: Vec::new(),
            table_width: Au::zero(),
            assignable_width: Au::zero(),
            final_table_height: Au::zero(),
            distributed_column_widths: Vec::new(),
            row_sizes: Vec::new(),
            row_baselines: Vec::new(),
            cells_laid_out: Vec::new(),
            basis_for_cell_padding_percentage: Au::zero(),
            collapsed_borders: None,
            is_in_fixed_mode,
        }
    }

    /// This is an implementation of *Computing Cell Measures* from
    /// <https://drafts.csswg.org/css-tables/#computing-cell-measures>.
    pub(crate) fn compute_cell_measures(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) {
        let row_measures = vec![LogicalVec2::zero(); self.table.size.width];
        self.cell_measures = vec![row_measures; self.table.size.height];

        for row_index in 0..self.table.size.height {
            for column_index in 0..self.table.size.width {
                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => continue,
                };

                let layout_style = cell.layout_style();
                let padding = layout_style
                    .padding(writing_mode)
                    .percentages_relative_to(Au::zero());
                let border = self
                    .get_collapsed_border_widths_for_area(LogicalSides {
                        inline_start: column_index,
                        inline_end: column_index + cell.colspan,
                        block_start: row_index,
                        block_end: row_index + cell.rowspan,
                    })
                    .unwrap_or_else(|| layout_style.border_width(writing_mode));

                let padding_border_sums = LogicalVec2 {
                    inline: padding.inline_sum() + border.inline_sum(),
                    block: padding.block_sum() + border.block_sum(),
                };

                let CellOrColumnOuterSizes {
                    preferred: preferred_size,
                    min: min_size,
                    max: max_size,
                    percentage: percentage_size,
                } = CellOrColumnOuterSizes::new(
                    &cell.base.style,
                    writing_mode,
                    &padding_border_sums,
                    self.is_in_fixed_mode,
                );

                // <https://drafts.csswg.org/css-tables/#in-fixed-mode>
                // > When a table-root is laid out in fixed mode, the content of its table-cells is ignored
                // > for the purpose of width computation, the aggregation algorithm for column sizing considers
                // > only table-cells belonging to the first row track
                let inline_measure = if self.is_in_fixed_mode {
                    if row_index > 0 {
                        CellOrTrackMeasure::zero()
                    } else {
                        CellOrTrackMeasure {
                            content_sizes: preferred_size.inline.into(),
                            percentage: percentage_size.inline,
                        }
                    }
                } else {
                    let inline_content_sizes = cell.inline_content_sizes(layout_context) +
                        padding_border_sums.inline.into();
                    assert!(
                        inline_content_sizes.max_content >= inline_content_sizes.min_content,
                        "the max-content size should never be smaller than the min-content size"
                    );

                    // These formulas differ from the spec, but seem to match Gecko and Blink.
                    let outer_min_content_width = inline_content_sizes
                        .min_content
                        .clamp_between_extremums(min_size.inline, max_size.inline);
                    let outer_max_content_width = if self.columns[column_index].constrained {
                        inline_content_sizes
                            .min_content
                            .max(preferred_size.inline)
                            .clamp_between_extremums(min_size.inline, max_size.inline)
                    } else {
                        inline_content_sizes
                            .max_content
                            .max(preferred_size.inline)
                            .clamp_between_extremums(min_size.inline, max_size.inline)
                    };
                    assert!(outer_min_content_width <= outer_max_content_width);

                    CellOrTrackMeasure {
                        content_sizes: ContentSizes {
                            min_content: outer_min_content_width,
                            max_content: outer_max_content_width,
                        },
                        percentage: percentage_size.inline,
                    }
                };

                // This measure doesn't take into account the `min-content` and `max-content` sizes.
                // These sizes are incorporated after the first row layout pass, when the block size
                // of the layout is known.
                let block_measure = CellOrTrackMeasure {
                    content_sizes: preferred_size.block.into(),
                    percentage: percentage_size.block,
                };

                self.cell_measures[row_index][column_index] = LogicalVec2 {
                    inline: inline_measure,
                    block: block_measure,
                };
            }
        }
    }

    /// Compute the constrainedness of every column in the table.
    ///
    /// > A column is constrained if its corresponding table-column-group (if any), its
    /// > corresponding table-column (if any), or any of the cells spanning only that
    /// > column has a computed width that is not "auto", and is not a percentage.
    fn compute_track_constrainedness_and_has_originating_cells(
        &mut self,
        writing_mode: WritingMode,
    ) {
        self.rows = vec![RowLayout::default(); self.table.size.height];
        self.columns = vec![ColumnLayout::default(); self.table.size.width];

        let is_length = |size: &Size<ComputedLengthPercentage>| {
            size.to_numeric().is_some_and(|size| !size.has_percentage())
        };

        for column_index in 0..self.table.size.width {
            if let Some(column) = self.table.columns.get(column_index) {
                if is_length(&column.style.box_size(writing_mode).inline) {
                    self.columns[column_index].constrained = true;
                    continue;
                }
                if let Some(column_group_index) = column.group_index {
                    let column_group = &self.table.column_groups[column_group_index];
                    if is_length(&column_group.style.box_size(writing_mode).inline) {
                        self.columns[column_index].constrained = true;
                        continue;
                    }
                }
            }
        }

        for row_index in 0..self.table.size.height {
            if let Some(row) = self.table.rows.get(row_index) {
                if is_length(&row.style.box_size(writing_mode).block) {
                    self.rows[row_index].constrained = true;
                    continue;
                }
                if let Some(row_group_index) = row.group_index {
                    let row_group = &self.table.row_groups[row_group_index];
                    if is_length(&row_group.style.box_size(writing_mode).block) {
                        self.rows[row_index].constrained = true;
                        continue;
                    }
                }
            }
        }

        for column_index in 0..self.table.size.width {
            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                let cell_constrained = match self.table.resolve_first_cell(coords) {
                    Some(cell) if cell.colspan == 1 => {
                        cell.base.style.box_size(writing_mode).map(is_length)
                    },
                    _ => LogicalVec2::default(),
                };

                let rowspan_greater_than_1 = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell.rowspan > 1,
                    _ => false,
                };

                self.rows[row_index].has_cell_with_span_greater_than_one |= rowspan_greater_than_1;
                self.rows[row_index].constrained |= cell_constrained.block;

                let has_originating_cell =
                    matches!(self.table.get_slot(coords), Some(TableSlot::Cell(_)));
                self.columns[column_index].has_originating_cells |= has_originating_cell;
                self.columns[column_index].constrained |= cell_constrained.inline;
            }
        }
    }

    /// This is an implementation of *Computing Column Measures* from
    /// <https://drafts.csswg.org/css-tables/#computing-column-measures>.
    fn compute_column_measures(&mut self, writing_mode: WritingMode) {
        // Compute the column measures only taking into account cells with colspan == 1.
        // This is the base case that will be used to iteratively account for cells with
        // larger colspans afterward.
        //
        // > min-content width of a column based on cells of span up to 1
        // >     The largest of:
        // >         - the width specified for the column:
        // >               - the outer min-content width of its corresponding table-column,
        // >                 if any (and not auto)
        // >               - the outer min-content width of its corresponding table-column-group, if any
        // >               - or 0, if there is none
        // >         - the outer min-content width of each cell that spans the column whose colSpan
        // >           is 1 (or just the one in the first row in fixed mode) or 0 if there is none
        // >
        // > max-content width of a column based on cells of span up to 1
        // >     The largest of:
        // >         - the outer max-content width of its corresponding
        // >           table-column-group, if any
        // >         - the outer max-content width of its corresponding table-column, if any
        // >         - the outer max-content width of each cell that spans the column
        // >           whose colSpan is 1 (or just the one in the first row if in fixed mode) or 0
        // >           if there is no such cell
        // >
        // > intrinsic percentage width of a column based on cells of span up to 1
        // >     The largest of the percentage contributions of each cell that spans the column whose colSpan is
        // >     1, of its corresponding table-column (if any), and of its corresponding table-column-group (if
        // >     any)
        //
        // TODO: Take into account `table-column` and `table-column-group` lengths.
        // TODO: Take into account changes to this computation for fixed table layout.
        let mut colspan_cell_constraints = Vec::new();
        for column_index in 0..self.table.size.width {
            let column = &mut self.columns[column_index];

            let column_measure = self.table.get_column_measure_for_column_at_index(
                writing_mode,
                column_index,
                self.is_in_fixed_mode,
            );
            column.content_sizes = column_measure.content_sizes;
            column.percentage = column_measure.percentage;

            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                let cell_measure = &self.cell_measures[row_index][column_index].inline;

                let cell = match self.table.get_slot(coords) {
                    Some(TableSlot::Cell(cell)) => cell,
                    _ => continue,
                };

                if cell.colspan != 1 {
                    colspan_cell_constraints.push(ColspanToDistribute {
                        starting_column: column_index,
                        span: cell.colspan,
                        content_sizes: cell_measure.content_sizes,
                        percentage: cell_measure.percentage,
                    });
                    continue;
                }

                // This takes the max of `min_content`, `max_content`, and
                // intrinsic percentage width as described above.
                column.incorporate_cell_measure(cell_measure);
            }
        }

        // Sort the colspanned cell constraints by their span and starting column.
        colspan_cell_constraints.sort_by(ColspanToDistribute::comparison_for_sort);

        // Distribute constraints from cells with colspan != 1 to their component columns.
        self.distribute_colspanned_cells_to_columns(colspan_cell_constraints);

        // > intrinsic percentage width of a column:
        // > the smaller of:
        // >   * the intrinsic percentage width of the column based on cells of span up to N,
        // >     where N is the number of columns in the table
        // >   * 100% minus the sum of the intrinsic percentage width of all prior columns in
        // >     the table (further left when direction is "ltr" (right for "rtl"))
        let mut total_intrinsic_percentage_width = 0.;
        for column in self.columns.iter_mut() {
            if let Some(ref mut percentage) = column.percentage {
                let final_intrinsic_percentage_width =
                    percentage.0.min(1. - total_intrinsic_percentage_width);
                total_intrinsic_percentage_width += final_intrinsic_percentage_width;
                *percentage = Percentage(final_intrinsic_percentage_width);
            }
        }
    }

    fn distribute_colspanned_cells_to_columns(
        &mut self,
        colspan_cell_constraints: Vec<ColspanToDistribute>,
    ) {
        for colspan_cell_constraints in colspan_cell_constraints {
            self.distribute_colspanned_cell_to_columns(colspan_cell_constraints);
        }
    }

    /// Distribute the inline size from a cell with colspan != 1 to the columns that it spans.
    /// This is heavily inspired by the approach that Chromium takes in redistributing colspan
    /// cells' inline size to columns (`DistributeColspanCellToColumnsAuto` in
    /// `blink/renderer/core/layout/table/table_layout_utils.cc`).
    fn distribute_colspanned_cell_to_columns(
        &mut self,
        colspan_cell_constraints: ColspanToDistribute,
    ) {
        let border_spacing = self.table.border_spacing().inline;
        let column_range = colspan_cell_constraints.range();
        let column_count = column_range.len();
        let total_border_spacing =
            border_spacing.scale_by((colspan_cell_constraints.span - 1) as f32);

        let mut percent_columns_count = 0;
        let mut columns_percent_sum = 0.;
        let mut columns_non_percent_max_inline_size_sum = Au::zero();
        for column in self.columns[column_range.clone()].iter() {
            if let Some(percentage) = column.percentage {
                percent_columns_count += 1;
                columns_percent_sum += percentage.0;
            } else {
                columns_non_percent_max_inline_size_sum += column.content_sizes.max_content;
            }
        }

        let colspan_percentage = colspan_cell_constraints.percentage.unwrap_or_default();
        let surplus_percent = colspan_percentage.0 - columns_percent_sum;
        if surplus_percent > 0. && column_count > percent_columns_count {
            for column in self.columns[column_range.clone()].iter_mut() {
                if column.percentage.is_some() {
                    continue;
                }

                let ratio = if columns_non_percent_max_inline_size_sum.is_zero() {
                    1. / ((column_count - percent_columns_count) as f32)
                } else {
                    column.content_sizes.max_content.to_f32_px() /
                        columns_non_percent_max_inline_size_sum.to_f32_px()
                };
                column.percentage = Some(Percentage(surplus_percent * ratio));
            }
        }

        let colspan_cell_min_size = (colspan_cell_constraints.content_sizes.min_content -
            total_border_spacing)
            .max(Au::zero());
        let distributed_minimum = Self::distribute_width_to_columns(
            colspan_cell_min_size,
            &self.columns[column_range.clone()],
        );
        {
            let column_span = &mut self.columns[colspan_cell_constraints.range()];
            for (column, minimum_size) in column_span.iter_mut().zip(distributed_minimum) {
                column.content_sizes.min_content.max_assign(minimum_size);
            }
        }

        let colspan_cell_max_size = (colspan_cell_constraints.content_sizes.max_content -
            total_border_spacing)
            .max(Au::zero());
        let distributed_maximum = Self::distribute_width_to_columns(
            colspan_cell_max_size,
            &self.columns[colspan_cell_constraints.range()],
        );
        {
            let column_span = &mut self.columns[colspan_cell_constraints.range()];
            for (column, maximum_size) in column_span.iter_mut().zip(distributed_maximum) {
                column
                    .content_sizes
                    .max_content
                    .max_assign(maximum_size.max(column.content_sizes.min_content));
            }
        }
    }

    fn compute_measures(&mut self, layout_context: &LayoutContext, writing_mode: WritingMode) {
        self.compute_track_constrainedness_and_has_originating_cells(writing_mode);
        self.compute_cell_measures(layout_context, writing_mode);
        self.compute_column_measures(writing_mode);
    }

    /// Compute the GRIDMIN and GRIDMAX.
    fn compute_grid_min_max(&self) -> ContentSizes {
        // https://drafts.csswg.org/css-tables/#gridmin:
        // > The row/column-grid width minimum (GRIDMIN) width is the sum of the min-content width of
        // > all the columns plus cell spacing or borders.
        // https://drafts.csswg.org/css-tables/#gridmax:
        // > The row/column-grid width maximum (GRIDMAX) width is the sum of the max-content width of
        // > all the columns plus cell spacing or borders.
        //
        // The specification doesn't say what to do with columns with percentages, so we follow the
        // approach that LayoutNG takes here. We try to figure out the size contribution
        // of the percentage columns, by working backward to find the calculated
        // percentage of non-percent columns and using that to calculate the size of the
        // percent columns.
        let mut largest_percentage_column_max_size = Au::zero();
        let mut percent_sum = 0.;
        let mut non_percent_columns_max_sum = Au::zero();
        let mut grid_min_max = ContentSizes::zero();
        for column in self.columns.iter() {
            match column.percentage {
                Some(percentage) if !percentage.is_zero() => {
                    largest_percentage_column_max_size.max_assign(
                        column
                            .content_sizes
                            .max_content
                            .scale_by(1.0 / percentage.0),
                    );
                    percent_sum += percentage.0;
                },
                _ => {
                    non_percent_columns_max_sum += column.content_sizes.max_content;
                },
            }

            grid_min_max += column.content_sizes;
        }

        grid_min_max
            .max_content
            .max_assign(largest_percentage_column_max_size);

        // Do not take into account percentage of columns when this table is a descendant
        // of a flex, grid, or table container. These modes with percentage columns can
        // cause inline width to become infinitely wide.
        if !percent_sum.is_zero() &&
            self.table
                .percentage_columns_allowed_for_inline_content_sizes
        {
            let total_inline_size =
                non_percent_columns_max_sum.scale_by(1.0 / (1.0 - percent_sum.min(1.0)));
            grid_min_max.max_content.max_assign(total_inline_size);
        }

        assert!(
            grid_min_max.min_content <= grid_min_max.max_content,
            "GRIDMAX should never be smaller than GRIDMIN {:?}",
            grid_min_max
        );

        let inline_border_spacing = self.table.total_border_spacing().inline;
        grid_min_max.min_content += inline_border_spacing;
        grid_min_max.max_content += inline_border_spacing;
        grid_min_max
    }

    /// Compute CAPMIN: <https://drafts.csswg.org/css-tables/#capmin>
    fn compute_caption_minimum_inline_size(&self, layout_context: &LayoutContext) -> Au {
        let containing_block = IndefiniteContainingBlock {
            size: LogicalVec2::default(),
            writing_mode: self.table.style.writing_mode,
        };
        self.table
            .captions
            .iter()
            .map(|caption| {
                let context = caption.context.borrow();
                context
                    .outer_inline_content_sizes(
                        layout_context,
                        &containing_block,
                        &LogicalVec2::zero(),
                        false, /* auto_block_size_stretches_to_containing_block */
                    )
                    .sizes
                    .min_content
            })
            .max()
            .unwrap_or_default()
    }

    fn compute_table_width(&mut self, containing_block_for_children: &ContainingBlock) {
        // This assumes that the parent formatting context computed the correct inline size
        // of the table, by enforcing its min-content size as a minimum.
        // This should be roughly equivalent to what the spec calls "used width of a table".
        // https://drafts.csswg.org/css-tables/#used-width-of-table
        self.table_width = containing_block_for_children.size.inline;

        // > The assignable table width is the used width of the table minus the total horizontal
        // > border spacing (if any). This is the width that we will be able to allocate to the
        // > columns.
        self.assignable_width = self.table_width - self.table.total_border_spacing().inline;

        // This is the amount that we will use to resolve percentages in the padding of cells.
        // It matches what Gecko and Blink do, though they disagree when there is a big caption.
        self.basis_for_cell_padding_percentage =
            self.table_width - self.table.border_spacing().inline * 2;
    }

    /// Distribute width to columns, performing step 2.4 of table layout from
    /// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>.
    fn distribute_width_to_columns(target_inline_size: Au, columns: &[ColumnLayout]) -> Vec<Au> {
        // No need to do anything if there is no column.
        // Note that tables without rows may still have columns.
        if columns.is_empty() {
            return Vec::new();
        }

        // > First, each column of the table is assigned a sizing type:
        // >  * percent-column: a column whose any constraint is defined to use a percentage only
        // >                    (with a value different from 0%)
        // >  * pixel-column: column whose any constraint is defined to use a defined length only
        // >                  (and is not a percent-column)
        // >  * auto-column: any other column
        // >
        // > Then, valid sizing methods are to be assigned to the columns by sizing type, yielding
        // > the following sizing-guesses:
        // >
        // > * The min-content sizing-guess is the set of column width assignments where
        // >   each column is assigned its min-content width.
        // > * The min-content-percentage sizing-guess is the set of column width assignments where:
        // >       * each percent-column is assigned the larger of:
        // >           * its intrinsic percentage width times the assignable width and
        // >           * its min-content width.
        // >       * all other columns are assigned their min-content width.
        // > * The min-content-specified sizing-guess is the set of column width assignments where:
        // >       * each percent-column is assigned the larger of:
        // >           * its intrinsic percentage width times the assignable width and
        // >           * its min-content width
        // >       * any other column that is constrained is assigned its max-content width
        // >       * all other columns are assigned their min-content width.
        // > * The max-content sizing-guess is the set of column width assignments where:
        // >       * each percent-column is assigned the larger of:
        // >           * its intrinsic percentage width times the assignable width and
        // >           * its min-content width
        // >       * all other columns are assigned their max-content width.
        let mut min_content_sizing_guesses = Vec::new();
        let mut min_content_percentage_sizing_guesses = Vec::new();
        let mut min_content_specified_sizing_guesses = Vec::new();
        let mut max_content_sizing_guesses = Vec::new();

        for column in columns {
            let min_content_width = column.content_sizes.min_content;
            let max_content_width = column.content_sizes.max_content;
            let constrained = column.constrained;

            let (
                min_content_percentage_sizing_guess,
                min_content_specified_sizing_guess,
                max_content_sizing_guess,
            ) = if let Some(percentage) = column.percentage {
                let resolved = target_inline_size.scale_by(percentage.0);
                let percent_guess = min_content_width.max(resolved);
                (percent_guess, percent_guess, percent_guess)
            } else if constrained {
                (min_content_width, max_content_width, max_content_width)
            } else {
                (min_content_width, min_content_width, max_content_width)
            };

            min_content_sizing_guesses.push(min_content_width);
            min_content_percentage_sizing_guesses.push(min_content_percentage_sizing_guess);
            min_content_specified_sizing_guesses.push(min_content_specified_sizing_guess);
            max_content_sizing_guesses.push(max_content_sizing_guess);
        }

        // > If the assignable table width is less than or equal to the max-content sizing-guess, the
        // > used widths of the columns must be the linear combination (with weights adding to 1) of
        // > the two consecutive sizing-guesses whose width sums bound the available width.
        //
        // > Otherwise, the used widths of the columns are the result of starting from the max-content
        // > sizing-guess and distributing the excess width to the columns of the table according to
        // > the rules for distributing excess width to columns (for used width).
        fn sum(guesses: &[Au]) -> Au {
            guesses.iter().fold(Au::zero(), |sum, guess| sum + *guess)
        }

        let max_content_sizing_sum = sum(&max_content_sizing_guesses);
        if target_inline_size >= max_content_sizing_sum {
            Self::distribute_extra_width_to_columns(
                columns,
                &mut max_content_sizing_guesses,
                max_content_sizing_sum,
                target_inline_size,
            );
            return max_content_sizing_guesses;
        }
        let min_content_specified_sizing_sum = sum(&min_content_specified_sizing_guesses);
        if target_inline_size == min_content_specified_sizing_sum {
            return min_content_specified_sizing_guesses;
        }
        let min_content_percentage_sizing_sum = sum(&min_content_percentage_sizing_guesses);
        if target_inline_size == min_content_percentage_sizing_sum {
            return min_content_percentage_sizing_guesses;
        }
        let min_content_sizes_sum = sum(&min_content_sizing_guesses);
        if target_inline_size <= min_content_sizes_sum {
            return min_content_sizing_guesses;
        }

        let bounds = |sum_a, sum_b| target_inline_size > sum_a && target_inline_size < sum_b;

        let blend = |a: &[Au], sum_a: Au, b: &[Au], sum_b: Au| {
            // First convert the Au units to f32 in order to do floating point division.
            let weight_a = (target_inline_size - sum_b).to_f32_px() / (sum_a - sum_b).to_f32_px();
            let weight_b = 1.0 - weight_a;

            let mut remaining_assignable_width = target_inline_size;
            let mut widths: Vec<Au> = a
                .iter()
                .zip(b.iter())
                .map(|(guess_a, guess_b)| {
                    let column_width = guess_a.scale_by(weight_a) + guess_b.scale_by(weight_b);
                    // Clamp to avoid exceeding the assignable width. This could otherwise
                    // happen when dealing with huge values whose sum is clamped to MAX_AU.
                    let column_width = column_width.min(remaining_assignable_width);
                    remaining_assignable_width -= column_width;
                    column_width
                })
                .collect();

            if !remaining_assignable_width.is_zero() {
                // The computations above can introduce floating-point imprecisions.
                // Since these errors are very small (1Au), it's fine to simply adjust
                // the first column such that the total width matches the assignable width
                debug_assert!(
                    remaining_assignable_width >= Au::zero(),
                    "Sum of columns shouldn't exceed the assignable table width"
                );
                debug_assert!(
                    remaining_assignable_width <= Au::new(widths.len() as i32),
                    "A deviation of more than one Au per column is unlikely to be caused by float imprecision"
                );

                // We checked if the table had columns at the top of the function, so there
                // always is a first column
                widths[0] += remaining_assignable_width;
            }

            debug_assert!(widths.iter().sum::<Au>() == target_inline_size);

            widths
        };

        if bounds(min_content_sizes_sum, min_content_percentage_sizing_sum) {
            return blend(
                &min_content_sizing_guesses,
                min_content_sizes_sum,
                &min_content_percentage_sizing_guesses,
                min_content_percentage_sizing_sum,
            );
        }

        if bounds(
            min_content_percentage_sizing_sum,
            min_content_specified_sizing_sum,
        ) {
            return blend(
                &min_content_percentage_sizing_guesses,
                min_content_percentage_sizing_sum,
                &min_content_specified_sizing_guesses,
                min_content_specified_sizing_sum,
            );
        }

        assert!(bounds(
            min_content_specified_sizing_sum,
            max_content_sizing_sum
        ));
        blend(
            &min_content_specified_sizing_guesses,
            min_content_specified_sizing_sum,
            &max_content_sizing_guesses,
            max_content_sizing_sum,
        )
    }

    /// This is an implementation of *Distributing excess width to columns* from
    /// <https://drafts.csswg.org/css-tables/#distributing-width-to-columns>.
    fn distribute_extra_width_to_columns(
        columns: &[ColumnLayout],
        column_sizes: &mut [Au],
        column_sizes_sum: Au,
        assignable_width: Au,
    ) {
        let all_columns = 0..columns.len();
        let extra_inline_size = assignable_width - column_sizes_sum;

        let has_originating_cells =
            |column_index: &usize| columns[*column_index].has_originating_cells;
        let is_constrained = |column_index: &usize| columns[*column_index].constrained;
        let is_unconstrained = |column_index: &usize| !is_constrained(column_index);
        let has_percent_greater_than_zero = |column_index: &usize| {
            columns[*column_index]
                .percentage
                .is_some_and(|percentage| percentage.0 > 0.)
        };
        let has_percent_zero = |column_index: &usize| !has_percent_greater_than_zero(column_index);
        let has_max_content =
            |column_index: &usize| !columns[*column_index].content_sizes.max_content.is_zero();

        let max_content_sum = |column_index: usize| columns[column_index].content_sizes.max_content;

        // > If there are non-constrained columns that have originating cells with intrinsic
        // > percentage width of 0% and with nonzero max-content width (aka the columns allowed to
        // > grow by this rule), the distributed widths of the columns allowed to grow by this rule
        // > are increased in proportion to max-content width so the total increase adds to the
        // > excess width.
        let unconstrained_max_content_columns = all_columns
            .clone()
            .filter(is_unconstrained)
            .filter(has_originating_cells)
            .filter(has_percent_zero)
            .filter(has_max_content);
        let total_max_content_width = unconstrained_max_content_columns
            .clone()
            .map(max_content_sum)
            .fold(Au::zero(), |a, b| a + b);
        if total_max_content_width != Au::zero() {
            for column_index in unconstrained_max_content_columns {
                column_sizes[column_index] += extra_inline_size.scale_by(
                    columns[column_index].content_sizes.max_content.to_f32_px() /
                        total_max_content_width.to_f32_px(),
                );
            }
            return;
        }

        // > Otherwise, if there are non-constrained columns that have originating cells with intrinsic
        // > percentage width of 0% (aka the columns allowed to grow by this rule, which thanks to the
        // > previous rule must have zero max-content width), the distributed widths of the columns
        // > allowed to grow by this rule are increased by equal amounts so the total increase adds to
        // > the excess width.V
        let unconstrained_no_percent_columns = all_columns
            .clone()
            .filter(is_unconstrained)
            .filter(has_originating_cells)
            .filter(has_percent_zero);
        let total_unconstrained_no_percent = unconstrained_no_percent_columns.clone().count();
        if total_unconstrained_no_percent > 0 {
            let extra_space_per_column =
                extra_inline_size.scale_by(1.0 / total_unconstrained_no_percent as f32);
            for column_index in unconstrained_no_percent_columns {
                column_sizes[column_index] += extra_space_per_column;
            }
            return;
        }

        // > Otherwise, if there are constrained columns with intrinsic percentage width of 0% and
        // > with nonzero max-content width (aka the columns allowed to grow by this rule, which, due
        // > to other rules, must have originating cells), the distributed widths of the columns
        // > allowed to grow by this rule are increased in proportion to max-content width so the
        // > total increase adds to the excess width.
        let constrained_max_content_columns = all_columns
            .clone()
            .filter(is_constrained)
            .filter(has_originating_cells)
            .filter(has_percent_zero)
            .filter(has_max_content);
        let total_max_content_width = constrained_max_content_columns
            .clone()
            .map(max_content_sum)
            .fold(Au::zero(), |a, b| a + b);
        if total_max_content_width != Au::zero() {
            for column_index in constrained_max_content_columns {
                column_sizes[column_index] += extra_inline_size.scale_by(
                    columns[column_index].content_sizes.max_content.to_f32_px() /
                        total_max_content_width.to_f32_px(),
                );
            }
            return;
        }

        // > Otherwise, if there are columns with intrinsic percentage width greater than 0% (aka the
        // > columns allowed to grow by this rule, which, due to other rules, must have originating
        // > cells), the distributed widths of the columns allowed to grow by this rule are increased
        // > in proportion to intrinsic percentage width so the total increase adds to the excess
        // > width.
        let columns_with_percentage = all_columns.clone().filter(has_percent_greater_than_zero);
        let total_percent = columns_with_percentage
            .clone()
            .map(|column_index| columns[column_index].percentage.unwrap_or_default().0)
            .sum::<f32>();
        if total_percent > 0. {
            for column_index in columns_with_percentage {
                let column_percentage = columns[column_index].percentage.unwrap_or_default();
                column_sizes[column_index] +=
                    extra_inline_size.scale_by(column_percentage.0 / total_percent);
            }
            return;
        }

        // > Otherwise, if there is any such column, the distributed widths of all columns that have
        // > originating cells are increased by equal amounts so the total increase adds to the excess
        // > width.
        let has_originating_cells_columns = all_columns.clone().filter(has_originating_cells);
        let total_has_originating_cells = has_originating_cells_columns.clone().count();
        if total_has_originating_cells > 0 {
            let extra_space_per_column =
                extra_inline_size.scale_by(1.0 / total_has_originating_cells as f32);
            for column_index in has_originating_cells_columns {
                column_sizes[column_index] += extra_space_per_column;
            }
            return;
        }

        // > Otherwise, the distributed widths of all columns are increased by equal amounts so the
        // total increase adds to the excess width.
        let extra_space_for_all_columns = extra_inline_size.scale_by(1.0 / columns.len() as f32);
        for guess in column_sizes.iter_mut() {
            *guess += extra_space_for_all_columns;
        }
    }

    /// This is an implementation of *Row layout (first pass)* from
    /// <https://drafts.csswg.org/css-tables/#row-layout>.
    fn layout_cells_in_row(
        &mut self,
        layout_context: &LayoutContext,
        containing_block_for_table: &ContainingBlock,
        parent_positioning_context: &mut PositioningContext,
    ) {
        self.cells_laid_out = self
            .table
            .slots
            .par_iter()
            .enumerate()
            .map(|(row_index, row_slots)| {
                // When building the PositioningContext for this cell, we want it to have the same
                // configuration for whatever PositioningContext the contents are ultimately added to.
                let collect_for_nearest_positioned_ancestor = parent_positioning_context
                    .collects_for_nearest_positioned_ancestor() ||
                    self.table.rows.get(row_index).is_some_and(|row| {
                        let row_group_collects_for_nearest_positioned_ancestor =
                            row.group_index.is_some_and(|group_index| {
                                self.table.row_groups[group_index]
                                    .style
                                    .establishes_containing_block_for_absolute_descendants(
                                        FragmentFlags::empty(),
                                    )
                            });
                        row_group_collects_for_nearest_positioned_ancestor ||
                            row.style
                                .establishes_containing_block_for_absolute_descendants(
                                    FragmentFlags::empty(),
                                )
                    });

                row_slots
                    .par_iter()
                    .enumerate()
                    .map(|(column_index, slot)| {
                        let TableSlot::Cell(cell) = slot else {
                            return None;
                        };

                        let area = LogicalSides {
                            inline_start: column_index,
                            inline_end: column_index + cell.colspan,
                            block_start: row_index,
                            block_end: row_index + cell.rowspan,
                        };
                        let layout_style = cell.layout_style();
                        let border = self
                            .get_collapsed_border_widths_for_area(area)
                            .unwrap_or_else(|| {
                                layout_style
                                    .border_width(containing_block_for_table.style.writing_mode)
                            });
                        let padding: LogicalSides<Au> = layout_style
                            .padding(containing_block_for_table.style.writing_mode)
                            .percentages_relative_to(self.basis_for_cell_padding_percentage);
                        let inline_border_padding_sum = border.inline_sum() + padding.inline_sum();

                        let mut total_cell_width: Au = (column_index..column_index + cell.colspan)
                            .map(|column_index| self.distributed_column_widths[column_index])
                            .sum::<Au>() -
                            inline_border_padding_sum;
                        total_cell_width = total_cell_width.max(Au::zero());

                        let containing_block_for_children = ContainingBlock {
                            size: ContainingBlockSize {
                                inline: total_cell_width,
                                block: SizeConstraint::default(),
                            },
                            style: &cell.base.style,
                        };

                        let mut positioning_context = PositioningContext::new_for_subtree(
                            collect_for_nearest_positioned_ancestor,
                        );

                        let layout = cell.contents.layout(
                            layout_context,
                            &mut positioning_context,
                            &containing_block_for_children,
                            false, /* depends_on_block_constraints */
                        );

                        Some(CellLayout {
                            layout,
                            padding,
                            border,
                            positioning_context,
                        })
                    })
                    .collect()
            })
            .collect();

        // Now go through all cells laid out and update the cell measure based on the size
        // determined during layout.
        for row_index in 0..self.table.size.height {
            for column_index in 0..self.table.size.width {
                let Some(layout) = &self.cells_laid_out[row_index][column_index] else {
                    continue;
                };

                self.cell_measures[row_index][column_index]
                    .block
                    .content_sizes
                    .max_assign(layout.outer_block_size().into());
            }
        }
    }

    /// Do the first layout of a table row, after laying out the cells themselves. This is
    /// more or less and implementation of <https://drafts.csswg.org/css-tables/#row-layout>.
    fn do_first_row_layout(&mut self, writing_mode: WritingMode) -> Vec<Au> {
        let mut row_sizes = (0..self.table.size.height)
            .map(|row_index| {
                let (mut max_ascent, mut max_descent, mut max_row_height) =
                    (Au::zero(), Au::zero(), Au::zero());

                for column_index in 0..self.table.size.width {
                    let cell = match self.table.slots[row_index][column_index] {
                        TableSlot::Cell(ref cell) => cell,
                        _ => continue,
                    };

                    let layout = match self.cells_laid_out[row_index][column_index] {
                        Some(ref layout) => layout,
                        None => {
                            warn!(
                                "Did not find a layout at a slot index with an originating cell."
                            );
                            continue;
                        },
                    };

                    let outer_block_size = layout.outer_block_size();
                    if cell.rowspan == 1 {
                        max_row_height.max_assign(outer_block_size);
                    }

                    if cell.effective_vertical_align() == VerticalAlignKeyword::Baseline {
                        let ascent = layout.ascent();
                        let border_padding_start =
                            layout.border.block_start + layout.padding.block_start;
                        let border_padding_end = layout.border.block_end + layout.padding.block_end;
                        max_ascent.max_assign(ascent + border_padding_start);

                        // Only take into account the descent of this cell if doesn't span
                        // rows. The descent portion of the cell in cells that do span rows
                        // may extend into other rows.
                        if cell.rowspan == 1 {
                            max_descent.max_assign(
                                layout.layout.content_block_size - ascent + border_padding_end,
                            );
                        }
                    }
                }
                self.row_baselines.push(max_ascent);
                max_row_height.max(max_ascent + max_descent)
            })
            .collect();
        self.calculate_row_sizes_after_first_layout(&mut row_sizes, writing_mode);
        row_sizes
    }

    #[allow(clippy::ptr_arg)] // Needs to be a vec because of the function above
    /// After doing layout of table rows, calculate final row size and distribute space across
    /// rowspanned cells. This follows the implementation of LayoutNG and the priority
    /// agorithm described at <https://github.com/w3c/csswg-drafts/issues/4418>.
    fn calculate_row_sizes_after_first_layout(
        &mut self,
        row_sizes: &mut Vec<Au>,
        writing_mode: WritingMode,
    ) {
        let mut cells_to_distribute = Vec::new();
        let mut total_percentage = 0.;
        #[allow(clippy::needless_range_loop)] // It makes sense to use it here
        for row_index in 0..self.table.size.height {
            let row_measure = self
                .table
                .get_row_measure_for_row_at_index(writing_mode, row_index);
            row_sizes[row_index].max_assign(row_measure.content_sizes.min_content);

            let mut percentage = row_measure.percentage.unwrap_or_default().0;
            for column_index in 0..self.table.size.width {
                let cell_percentage = self.cell_measures[row_index][column_index]
                    .block
                    .percentage
                    .unwrap_or_default()
                    .0;
                percentage = percentage.max(cell_percentage);

                let cell_measure = &self.cell_measures[row_index][column_index].block;
                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) if cell.rowspan > 1 => cell,
                    TableSlot::Cell(_) => {
                        // If this is an originating cell, that isn't spanning, then we make sure the row is
                        // at least big enough to hold the cell.
                        row_sizes[row_index].max_assign(cell_measure.content_sizes.max_content);
                        continue;
                    },
                    _ => continue,
                };

                cells_to_distribute.push(RowspanToDistribute {
                    coordinates: TableSlotCoordinates::new(column_index, row_index),
                    cell,
                    measure: cell_measure,
                });
            }

            self.rows[row_index].percent = Percentage(percentage.min(1. - total_percentage));
            total_percentage += self.rows[row_index].percent.0;
        }

        cells_to_distribute.sort_by(|a, b| {
            if a.range() == b.range() {
                return a
                    .measure
                    .content_sizes
                    .min_content
                    .cmp(&b.measure.content_sizes.min_content);
            }
            if a.fully_encloses(b) {
                return std::cmp::Ordering::Greater;
            }
            if b.fully_encloses(a) {
                return std::cmp::Ordering::Less;
            }
            a.coordinates.y.cmp(&b.coordinates.y)
        });

        for rowspan_to_distribute in cells_to_distribute {
            let rows_spanned = rowspan_to_distribute.range();
            let current_rows_size: Au = rows_spanned.clone().map(|index| row_sizes[index]).sum();
            let border_spacing_spanned =
                self.table.border_spacing().block * (rows_spanned.len() - 1) as i32;
            let excess_size = (rowspan_to_distribute.measure.content_sizes.min_content -
                current_rows_size -
                border_spacing_spanned)
                .max(Au::zero());

            self.distribute_extra_size_to_rows(
                excess_size,
                rows_spanned,
                row_sizes,
                None,
                true, /* rowspan_distribution */
            );
        }
    }

    /// An implementation of the same extra block size distribution algorithm used in
    /// LayoutNG and described at <https://github.com/w3c/csswg-drafts/issues/4418>.
    fn distribute_extra_size_to_rows(
        &self,
        mut excess_size: Au,
        track_range: Range<usize>,
        track_sizes: &mut [Au],
        percentage_resolution_size: Option<Au>,
        rowspan_distribution: bool,
    ) {
        if excess_size.is_zero() {
            return;
        }

        let is_constrained = |track_index: &usize| self.rows[*track_index].constrained;
        let is_unconstrained = |track_index: &usize| !is_constrained(track_index);
        let is_empty: Vec<bool> = track_sizes.iter().map(|size| size.is_zero()).collect();
        let is_not_empty = |track_index: &usize| !is_empty[*track_index];
        let other_row_that_starts_a_rowspan = |track_index: &usize| {
            *track_index != track_range.start &&
                self.rows[*track_index].has_cell_with_span_greater_than_one
        };

        // If we have a table height (not during rowspan distribution), first distribute to rows
        // that have percentage sizes proportionally to the size missing to reach the percentage
        // of table height required.
        if let Some(percentage_resolution_size) = percentage_resolution_size {
            let get_percent_block_size_deficit = |row_index: usize, track_size: Au| {
                let size_needed_for_percent =
                    percentage_resolution_size.scale_by(self.rows[row_index].percent.0);
                (size_needed_for_percent - track_size).max(Au::zero())
            };
            let percent_block_size_deficit: Au = track_range
                .clone()
                .map(|index| get_percent_block_size_deficit(index, track_sizes[index]))
                .sum();
            let percent_distributable_block_size = percent_block_size_deficit.min(excess_size);
            if percent_distributable_block_size > Au::zero() {
                for track_index in track_range.clone() {
                    let row_deficit =
                        get_percent_block_size_deficit(track_index, track_sizes[track_index]);
                    if row_deficit > Au::zero() {
                        let ratio =
                            row_deficit.to_f32_px() / percent_block_size_deficit.to_f32_px();
                        let size = percent_distributable_block_size.scale_by(ratio);
                        track_sizes[track_index] += size;
                        excess_size -= size;
                    }
                }
            }
        }

        // If this is rowspan distribution and there are rows other than the first row that have a
        // cell with rowspan > 1, distribute the extra space equally to those rows.
        if rowspan_distribution {
            let rows_that_start_rowspan: Vec<usize> = track_range
                .clone()
                .filter(other_row_that_starts_a_rowspan)
                .collect();
            if !rows_that_start_rowspan.is_empty() {
                let scale = 1.0 / rows_that_start_rowspan.len() as f32;
                for track_index in rows_that_start_rowspan.iter() {
                    track_sizes[*track_index] += excess_size.scale_by(scale);
                }
                return;
            }
        }

        // If there are unconstrained non-empty rows, grow them all proportionally to their current size.
        let unconstrained_non_empty_rows: Vec<usize> = track_range
            .clone()
            .filter(is_unconstrained)
            .filter(is_not_empty)
            .collect();
        if !unconstrained_non_empty_rows.is_empty() {
            let total_size: Au = unconstrained_non_empty_rows
                .iter()
                .map(|index| track_sizes[*index])
                .sum();
            for track_index in unconstrained_non_empty_rows.iter() {
                let scale = track_sizes[*track_index].to_f32_px() / total_size.to_f32_px();
                track_sizes[*track_index] += excess_size.scale_by(scale);
            }
            return;
        }

        let (non_empty_rows, empty_rows): (Vec<usize>, Vec<usize>) =
            track_range.clone().partition(is_not_empty);
        let only_have_empty_rows = empty_rows.len() == track_range.len();
        if !empty_rows.is_empty() {
            // If this is rowspan distribution and there are only empty rows, just grow the
            // last one.
            if rowspan_distribution && only_have_empty_rows {
                track_sizes[*empty_rows.last().unwrap()] += excess_size;
                return;
            }

            // Otherwise, if we only have empty rows or if all the non-empty rows are constrained,
            // then grow the empty rows.
            let non_empty_rows_all_constrained = !non_empty_rows.iter().any(is_unconstrained);
            if only_have_empty_rows || non_empty_rows_all_constrained {
                // If there are both unconstrained and constrained empty rows, only increase the
                // size of the unconstrained ones, otherwise increase the size of all empty rows.
                let mut rows_to_grow = &empty_rows;
                let unconstrained_empty_rows: Vec<usize> = rows_to_grow
                    .iter()
                    .copied()
                    .filter(is_unconstrained)
                    .collect();
                if !unconstrained_empty_rows.is_empty() {
                    rows_to_grow = &unconstrained_empty_rows;
                }

                // All empty rows that will grow equally.
                let scale = 1.0 / rows_to_grow.len() as f32;
                for track_index in rows_to_grow.iter() {
                    track_sizes[*track_index] += excess_size.scale_by(scale);
                }
                return;
            }
        }

        // If there are non-empty rows, they all grow in proportion to their current size,
        // whether or not they are constrained.
        if !non_empty_rows.is_empty() {
            let total_size: Au = non_empty_rows.iter().map(|index| track_sizes[*index]).sum();
            for track_index in non_empty_rows.iter() {
                let scale = track_sizes[*track_index].to_f32_px() / total_size.to_f32_px();
                track_sizes[*track_index] += excess_size.scale_by(scale);
            }
        }
    }

    /// Given computed row sizes, compute the final block size of the table and distribute extra
    /// block size to table rows.
    fn compute_table_height_and_final_row_heights(
        &mut self,
        mut row_sizes: Vec<Au>,
        containing_block_for_children: &ContainingBlock,
    ) {
        // The table content height is the maximum of the computed table height from style and the
        // sum of computed row heights from row layout plus size from borders and spacing.
        // TODO: for `height: stretch`, the block size of the containing block is the available
        // space for the entire table wrapper, but here we are using that amount for the table grid.
        // Therefore, if there is a caption, this will cause overflow. Gecko and WebKit have the
        // same problem, but not Blink.
        let table_height_from_style = match containing_block_for_children.size.block {
            SizeConstraint::Definite(size) => size,
            SizeConstraint::MinMax(min, _) => min,
        };

        let block_border_spacing = self.table.total_border_spacing().block;
        let table_height_from_rows = row_sizes.iter().sum::<Au>() + block_border_spacing;
        self.final_table_height = table_height_from_rows.max(table_height_from_style);

        // If the table height is defined by the rows sizes, there is no extra space to distribute
        // to rows.
        if self.final_table_height == table_height_from_rows {
            self.row_sizes = row_sizes;
            return;
        }

        // There was extra block size added to the table from the table style, so distribute this
        // extra space to rows using the same distribution algorithm used for distributing rowspan
        // space.
        // TODO: This should first distribute space to row groups and then to rows.
        self.distribute_extra_size_to_rows(
            self.final_table_height - table_height_from_rows,
            0..self.table.size.height,
            &mut row_sizes,
            Some(self.final_table_height),
            false, /* rowspan_distribution */
        );
        self.row_sizes = row_sizes;
    }

    fn layout_caption(
        &self,
        caption: &TableCaption,
        layout_context: &LayoutContext,
        parent_positioning_context: &mut PositioningContext,
    ) -> BoxFragment {
        let context = caption.context.borrow();
        let mut positioning_context = PositioningContext::new_for_style(context.style());
        let containing_block = &ContainingBlock {
            size: ContainingBlockSize {
                inline: self.table_width + self.pbm.padding_border_sums.inline,
                block: SizeConstraint::default(),
            },
            style: &self.table.style,
        };

        // The parent of a caption is the table wrapper, which establishes an independent
        // formatting context. Therefore, we don't ignore block margins when resolving a
        // stretch block size. https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing
        let ignore_block_margins_for_stretch = LogicalSides1D::new(false, false);

        let mut box_fragment = context.layout_in_flow_block_level(
            layout_context,
            positioning_context
                .as_mut()
                .unwrap_or(parent_positioning_context),
            containing_block,
            None, /* sequential_layout_state */
            ignore_block_margins_for_stretch,
        );

        if let Some(mut positioning_context) = positioning_context.take() {
            positioning_context.layout_collected_children(layout_context, &mut box_fragment);
            parent_positioning_context.append(positioning_context);
        }

        box_fragment
    }

    /// Lay out the table (grid and captions) of this [`TableLayout`] into fragments. This should
    /// only be be called after calling [`TableLayout.compute_measures`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "Table::layout",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    fn layout(
        mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
        depends_on_block_constraints: bool,
    ) -> CacheableLayoutResult {
        let table_writing_mode = containing_block_for_children.style.writing_mode;
        self.compute_border_collapse(table_writing_mode);
        let layout_style = self.table.layout_style(Some(&self));
        let depends_on_block_constraints = depends_on_block_constraints ||
            layout_style
                .content_box_sizes_and_padding_border_margin(&containing_block_for_table.into())
                .depends_on_block_constraints;

        self.pbm = layout_style
            .padding_border_margin_with_writing_mode_and_containing_block_inline_size(
                table_writing_mode,
                containing_block_for_table.size.inline,
            );
        self.compute_measures(layout_context, table_writing_mode);
        self.compute_table_width(containing_block_for_children);

        // The table wrapper is the one that has the CSS properties for the grid's border and padding. This
        // weirdness is difficult to express in Servo's layout system. We have the wrapper size itself as if
        // those properties applied to it and then just account for the discrepency in sizing here. In reality,
        // the wrapper does not draw borders / backgrounds and all of its content (grid and captions) are
        // placed with a negative offset in the table wrapper's content box so that they overlap the undrawn
        // border / padding area.
        //
        // TODO: This is a pretty large hack. It would be nicer to actually have the grid sized properly,
        // but it works for now.
        //
        // Get the padding, border, and margin of the table using the inline size of the table's containing
        // block but in the writing of the table itself.
        // TODO: This is broken for orthoganol flows, because the inline size of the parent isn't necessarily
        // the inline size of the table.
        let containing_block_for_logical_conversion = ContainingBlock {
            size: ContainingBlockSize {
                inline: self.table_width,
                block: containing_block_for_table.size.block,
            },
            style: containing_block_for_children.style,
        };
        let offset_from_wrapper = -self.pbm.padding - self.pbm.border;
        let mut current_block_offset = offset_from_wrapper.block_start;

        let mut table_layout = CacheableLayoutResult {
            fragments: Vec::new(),
            content_block_size: Zero::zero(),
            content_inline_size_for_table: None,
            baselines: Baselines::default(),
            depends_on_block_constraints,
            specific_layout_info: Some(SpecificLayoutInfo::TableWrapper),
            collapsible_margins_in_children: CollapsedBlockMargins::zero(),
        };

        table_layout
            .fragments
            .extend(self.table.captions.iter().filter_map(|caption| {
                if caption.context.borrow().style().clone_caption_side() != CaptionSide::Top {
                    return None;
                }

                let original_positioning_context_length = positioning_context.len();
                let mut caption_fragment =
                    self.layout_caption(caption, layout_context, positioning_context);

                // The caption is not placed yet. Construct a rectangle for it in the adjusted containing block
                // for the table children and only then convert the result to physical geometry.
                let caption_pbm = caption_fragment
                    .padding_border_margin()
                    .to_logical(table_writing_mode);

                let caption_relative_offset = match caption_fragment.style.clone_position() {
                    Position::Relative => {
                        relative_adjustement(&caption_fragment.style, containing_block_for_children)
                    },
                    _ => LogicalVec2::zero(),
                };

                caption_fragment.content_rect = LogicalRect {
                    start_corner: LogicalVec2 {
                        inline: offset_from_wrapper.inline_start + caption_pbm.inline_start,
                        block: current_block_offset + caption_pbm.block_start,
                    } + caption_relative_offset,
                    size: caption_fragment
                        .content_rect
                        .size
                        .to_logical(table_writing_mode),
                }
                .as_physical(Some(&containing_block_for_logical_conversion));

                current_block_offset += caption_fragment
                    .margin_rect()
                    .size
                    .to_logical(table_writing_mode)
                    .block;

                let caption_fragment = Fragment::Box(ArcRefCell::new(caption_fragment));
                positioning_context.adjust_static_position_of_hoisted_fragments(
                    &caption_fragment,
                    original_positioning_context_length,
                );
                Some(caption_fragment)
            }));

        let original_positioning_context_length = positioning_context.len();
        let mut grid_fragment = self.layout_grid(
            layout_context,
            positioning_context,
            &containing_block_for_logical_conversion,
            containing_block_for_children,
        );

        // Take the baseline of the grid fragment, after adjusting it to be in the coordinate system
        // of the table wrapper.
        let logical_grid_content_rect = grid_fragment
            .content_rect
            .to_logical(&containing_block_for_logical_conversion);
        let grid_pbm = grid_fragment
            .padding_border_margin()
            .to_logical(table_writing_mode);
        table_layout.baselines = grid_fragment.baselines(table_writing_mode).offset(
            current_block_offset +
                logical_grid_content_rect.start_corner.block +
                grid_pbm.block_start,
        );

        grid_fragment.content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                inline: offset_from_wrapper.inline_start + grid_pbm.inline_start,
                block: current_block_offset + grid_pbm.block_start,
            },
            size: grid_fragment
                .content_rect
                .size
                .to_logical(table_writing_mode),
        }
        .as_physical(Some(&containing_block_for_logical_conversion));

        current_block_offset += grid_fragment
            .border_rect()
            .size
            .to_logical(table_writing_mode)
            .block;
        if logical_grid_content_rect.size.inline < self.table_width {
            // This can happen when collapsing columns
            table_layout.content_inline_size_for_table =
                Some(logical_grid_content_rect.size.inline);
        }

        let grid_fragment = Fragment::Box(ArcRefCell::new(grid_fragment));
        positioning_context.adjust_static_position_of_hoisted_fragments(
            &grid_fragment,
            original_positioning_context_length,
        );
        table_layout.fragments.push(grid_fragment);

        table_layout
            .fragments
            .extend(self.table.captions.iter().filter_map(|caption| {
                if caption.context.borrow().style().clone_caption_side() != CaptionSide::Bottom {
                    return None;
                }

                let original_positioning_context_length = positioning_context.len();
                let mut caption_fragment =
                    self.layout_caption(caption, layout_context, positioning_context);

                // The caption is not placed yet. Construct a rectangle for it in the adjusted containing block
                // for the table children and only then convert the result to physical geometry.
                let caption_pbm = caption_fragment
                    .padding_border_margin()
                    .to_logical(table_writing_mode);
                caption_fragment.content_rect = LogicalRect {
                    start_corner: LogicalVec2 {
                        inline: offset_from_wrapper.inline_start + caption_pbm.inline_start,
                        block: current_block_offset + caption_pbm.block_start,
                    },
                    size: caption_fragment
                        .content_rect
                        .size
                        .to_logical(table_writing_mode),
                }
                .as_physical(Some(&containing_block_for_logical_conversion));

                current_block_offset += caption_fragment
                    .margin_rect()
                    .size
                    .to_logical(table_writing_mode)
                    .block;

                let caption_fragment = Fragment::Box(ArcRefCell::new(caption_fragment));
                positioning_context.adjust_static_position_of_hoisted_fragments(
                    &caption_fragment,
                    original_positioning_context_length,
                );
                Some(caption_fragment)
            }));

        table_layout.content_block_size = current_block_offset + offset_from_wrapper.block_end;
        table_layout
    }

    /// Lay out the grid portion of this [`TableLayout`] into fragments. This should only be be
    /// called after calling [`TableLayout.compute_measures`].
    fn layout_grid(
        &mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_logical_conversion: &ContainingBlock,
        containing_block_for_children: &ContainingBlock,
    ) -> BoxFragment {
        self.distributed_column_widths =
            Self::distribute_width_to_columns(self.assignable_width, &self.columns);
        self.layout_cells_in_row(
            layout_context,
            containing_block_for_children,
            positioning_context,
        );
        let table_writing_mode = containing_block_for_children.style.writing_mode;
        let first_layout_row_heights = self.do_first_row_layout(table_writing_mode);
        self.compute_table_height_and_final_row_heights(
            first_layout_row_heights,
            containing_block_for_children,
        );

        assert_eq!(self.table.size.height, self.row_sizes.len());
        assert_eq!(self.table.size.width, self.distributed_column_widths.len());

        if self.table.size.width == 0 && self.table.size.height == 0 {
            let content_rect = LogicalRect {
                start_corner: LogicalVec2::zero(),
                size: LogicalVec2 {
                    inline: self.table_width,
                    block: self.final_table_height,
                },
            }
            .as_physical(Some(containing_block_for_logical_conversion));
            return BoxFragment::new(
                self.table.grid_base_fragment_info,
                self.table.grid_style.clone(),
                Vec::new(),
                content_rect,
                self.pbm.padding.to_physical(table_writing_mode),
                self.pbm.border.to_physical(table_writing_mode),
                PhysicalSides::zero(),
                None, /* clearance */
                CollapsedBlockMargins::zero(),
            )
            .with_specific_layout_info(self.specific_layout_info_for_grid());
        }

        let mut table_fragments = Vec::new();
        let table_and_track_dimensions = TableAndTrackDimensions::new(self);
        self.make_fragments_for_columns_and_column_groups(
            &table_and_track_dimensions,
            &mut table_fragments,
        );

        let mut baselines = Baselines::default();
        let mut row_group_fragment_layout = None;
        for row_index in 0..self.table.size.height {
            // From <https://drafts.csswg.org/css-align-3/#baseline-export>
            // > If any cells in the row participate in first baseline/last baseline alignment along
            // > the inline axis, the first/last baseline set of the row is generated from their
            // > shared alignment baseline and the row’s first available font, after alignment has
            // > been performed. Otherwise, the first/last baseline set of the row is synthesized from
            // > the lowest and highest content edges of the cells in the row. [CSS2]
            //
            // If any cell below has baseline alignment, these values will be overwritten,
            // but they are initialized to the content edge of the first row.
            if row_index == 0 {
                let row_end = table_and_track_dimensions
                    .get_row_rect(0)
                    .max_block_position();
                baselines.first = Some(row_end);
                baselines.last = Some(row_end);
            }

            if self.is_row_collapsed(row_index) {
                continue;
            }

            let table_row = &self.table.rows[row_index];
            let mut row_fragment_layout = RowFragmentLayout::new(
                table_row,
                row_index,
                &table_and_track_dimensions,
                &self.table.style,
            );

            let old_row_group_index = row_group_fragment_layout
                .as_ref()
                .map(|layout: &RowGroupFragmentLayout| layout.index);
            if table_row.group_index != old_row_group_index {
                // First create the Fragment for any existing RowGroupFragmentLayout.
                if let Some(old_row_group_layout) = row_group_fragment_layout.take() {
                    table_fragments.push(Fragment::Box(old_row_group_layout.finish(
                        layout_context,
                        positioning_context,
                        containing_block_for_logical_conversion,
                        containing_block_for_children,
                    )));
                }

                // Then, create a new RowGroupFragmentLayout for the current and potentially subsequent rows.
                if let Some(new_group_index) = table_row.group_index {
                    row_group_fragment_layout = Some(RowGroupFragmentLayout::new(
                        &self.table.row_groups[new_group_index],
                        new_group_index,
                        &table_and_track_dimensions,
                    ));
                }
            }

            let column_indices = 0..self.table.size.width;
            row_fragment_layout.fragments.reserve(self.table.size.width);
            for column_index in column_indices {
                if self.is_column_collapsed(column_index) {
                    continue;
                }

                self.do_final_cell_layout(
                    row_index,
                    column_index,
                    &table_and_track_dimensions,
                    &mut baselines,
                    &mut row_fragment_layout,
                    row_group_fragment_layout.as_mut(),
                    positioning_context,
                );
            }

            let row_fragment = Fragment::Box(row_fragment_layout.finish(
                layout_context,
                positioning_context,
                containing_block_for_logical_conversion,
                containing_block_for_children,
                &mut row_group_fragment_layout,
            ));

            match row_group_fragment_layout.as_mut() {
                Some(layout) => layout.fragments.push(row_fragment),
                None => table_fragments.push(row_fragment),
            }
        }

        if let Some(row_group_layout) = row_group_fragment_layout.take() {
            table_fragments.push(Fragment::Box(row_group_layout.finish(
                layout_context,
                positioning_context,
                containing_block_for_logical_conversion,
                containing_block_for_children,
            )));
        }

        let content_rect = LogicalRect {
            start_corner: LogicalVec2::zero(),
            size: LogicalVec2 {
                inline: table_and_track_dimensions.table_rect.max_inline_position(),
                block: table_and_track_dimensions.table_rect.max_block_position(),
            },
        }
        .as_physical(Some(containing_block_for_logical_conversion));
        BoxFragment::new(
            self.table.grid_base_fragment_info,
            self.table.grid_style.clone(),
            table_fragments,
            content_rect,
            self.pbm.padding.to_physical(table_writing_mode),
            self.pbm.border.to_physical(table_writing_mode),
            PhysicalSides::zero(),
            None, /* clearance */
            CollapsedBlockMargins::zero(),
        )
        .with_baselines(baselines)
        .with_specific_layout_info(self.specific_layout_info_for_grid())
    }

    fn specific_layout_info_for_grid(&mut self) -> Option<SpecificLayoutInfo> {
        mem::take(&mut self.collapsed_borders).map(|mut collapsed_borders| {
            // TODO: It would probably be better to use `TableAndTrackDimensions`, since that
            // has already taken care of collapsed tracks and knows the final track positions.
            let mut track_sizes = LogicalVec2 {
                inline: mem::take(&mut self.distributed_column_widths),
                block: mem::take(&mut self.row_sizes),
            };
            for (column_index, column_size) in track_sizes.inline.iter_mut().enumerate() {
                if self.is_column_collapsed(column_index) {
                    mem::take(column_size);
                }
            }
            for (row_index, row_size) in track_sizes.block.iter_mut().enumerate() {
                if self.is_row_collapsed(row_index) {
                    mem::take(row_size);
                }
            }
            let writing_mode = self.table.style.writing_mode;
            if !writing_mode.is_bidi_ltr() {
                track_sizes.inline.reverse();
                collapsed_borders.inline.reverse();
                for border_line in &mut collapsed_borders.block {
                    border_line.reverse();
                }
            }
            SpecificLayoutInfo::TableGridWithCollapsedBorders(Box::new(SpecificTableGridInfo {
                collapsed_borders: if writing_mode.is_horizontal() {
                    PhysicalVec::new(collapsed_borders.inline, collapsed_borders.block)
                } else {
                    PhysicalVec::new(collapsed_borders.block, collapsed_borders.inline)
                },
                track_sizes: if writing_mode.is_horizontal() {
                    PhysicalVec::new(track_sizes.inline, track_sizes.block)
                } else {
                    PhysicalVec::new(track_sizes.block, track_sizes.inline)
                },
            }))
        })
    }

    fn is_row_collapsed(&self, row_index: usize) -> bool {
        let Some(row) = &self.table.rows.get(row_index) else {
            return false;
        };
        if row.style.get_inherited_box().visibility == Visibility::Collapse {
            return true;
        }
        let row_group = match row.group_index {
            Some(group_index) => &self.table.row_groups[group_index],
            None => return false,
        };
        row_group.style.get_inherited_box().visibility == Visibility::Collapse
    }

    fn is_column_collapsed(&self, column_index: usize) -> bool {
        let Some(col) = &self.table.columns.get(column_index) else {
            return false;
        };
        if col.style.get_inherited_box().visibility == Visibility::Collapse {
            return true;
        }
        let col_group = match col.group_index {
            Some(group_index) => &self.table.column_groups[group_index],
            None => return false,
        };
        col_group.style.get_inherited_box().visibility == Visibility::Collapse
    }

    #[allow(clippy::too_many_arguments)]
    fn do_final_cell_layout(
        &mut self,
        row_index: usize,
        column_index: usize,
        dimensions: &TableAndTrackDimensions,
        baselines: &mut Baselines,
        row_fragment_layout: &mut RowFragmentLayout,
        row_group_fragment_layout: Option<&mut RowGroupFragmentLayout>,
        positioning_context_for_table: &mut PositioningContext,
    ) {
        // The PositioningContext for cells is, in order or preference, the PositioningContext of the row,
        // the PositioningContext of the row group, or the PositioningContext of the table.
        let row_group_positioning_context =
            row_group_fragment_layout.and_then(|layout| layout.positioning_context.as_mut());
        let positioning_context = row_fragment_layout
            .positioning_context
            .as_mut()
            .or(row_group_positioning_context)
            .unwrap_or(positioning_context_for_table);

        let layout = match self.cells_laid_out[row_index][column_index].take() {
            Some(layout) => layout,
            None => {
                return;
            },
        };
        let cell = match self.table.slots[row_index][column_index] {
            TableSlot::Cell(ref cell) => cell,
            _ => {
                warn!("Did not find a non-spanned cell at index with layout.");
                return;
            },
        };

        // If this cell has baseline alignment, it can adjust the table's overall baseline.
        let row_block_offset = row_fragment_layout.rect.start_corner.block;
        let row_baseline = self.row_baselines[row_index];
        if cell.effective_vertical_align() == VerticalAlignKeyword::Baseline && !layout.is_empty() {
            let baseline = row_block_offset + row_baseline;
            if row_index == 0 {
                baselines.first = Some(baseline);
            }
            baselines.last = Some(baseline);
        }
        let mut row_relative_cell_rect = dimensions.get_cell_rect(
            TableSlotCoordinates::new(column_index, row_index),
            cell.rowspan,
            cell.colspan,
        );
        row_relative_cell_rect.start_corner -= row_fragment_layout.rect.start_corner;
        let mut fragment = cell.create_fragment(
            layout,
            row_relative_cell_rect,
            row_baseline,
            positioning_context,
            &self.table.style,
            &row_fragment_layout.containing_block,
        );

        // Make a table part rectangle relative to the row fragment for the purposes of
        // drawing extra backgrounds.
        //
        // This rectangle is an offset between the row fragment and the other table
        // part rectangle (row group, column, column group). Everything between them
        // is laid out in a left-to-right fashion, but respecting the verticality of
        // the writing mode. This is why below, only the axes are flipped, but the
        // rectangle is not flipped for RTL.
        let make_relative_to_row_start = |mut rect: LogicalRect<Au>| {
            rect.start_corner -= row_fragment_layout.rect.start_corner;
            let writing_mode = self.table.style.writing_mode;
            PhysicalRect::new(
                if writing_mode.is_horizontal() {
                    PhysicalPoint::new(rect.start_corner.inline, rect.start_corner.block)
                } else {
                    PhysicalPoint::new(rect.start_corner.block, rect.start_corner.inline)
                },
                rect.size.to_physical_size(writing_mode),
            )
        };

        let column = self.table.columns.get(column_index);
        let column_group = column
            .and_then(|column| column.group_index)
            .and_then(|index| self.table.column_groups.get(index));
        if let Some(column_group) = column_group {
            let rect = make_relative_to_row_start(dimensions.get_column_group_rect(column_group));
            fragment.add_extra_background(ExtraBackground {
                style: column_group.style.clone(),
                rect,
            })
        }
        if let Some(column) = column {
            if !column.is_anonymous {
                let rect = make_relative_to_row_start(dimensions.get_column_rect(column_index));
                fragment.add_extra_background(ExtraBackground {
                    style: column.style.clone(),
                    rect,
                })
            }
        }
        let row = self.table.rows.get(row_index);
        let row_group = row
            .and_then(|row| row.group_index)
            .and_then(|index| self.table.row_groups.get(index));
        if let Some(row_group) = row_group {
            let rect = make_relative_to_row_start(dimensions.get_row_group_rect(row_group));
            fragment.add_extra_background(ExtraBackground {
                style: row_group.style.clone(),
                rect,
            })
        }
        if let Some(row) = row {
            let rect = make_relative_to_row_start(row_fragment_layout.rect);
            fragment.add_extra_background(ExtraBackground {
                style: row.style.clone(),
                rect,
            })
        }
        row_fragment_layout
            .fragments
            .push(Fragment::Box(ArcRefCell::new(fragment)));
    }

    fn make_fragments_for_columns_and_column_groups(
        &self,
        dimensions: &TableAndTrackDimensions,
        fragments: &mut Vec<Fragment>,
    ) {
        for column_group in self.table.column_groups.iter() {
            if !column_group.is_empty() {
                fragments.push(Fragment::Positioning(PositioningFragment::new_empty(
                    column_group.base_fragment_info,
                    dimensions
                        .get_column_group_rect(column_group)
                        .as_physical(None),
                    column_group.style.clone(),
                )));
            }
        }

        for (column_index, column) in self.table.columns.iter().enumerate() {
            fragments.push(Fragment::Positioning(PositioningFragment::new_empty(
                column.base_fragment_info,
                dimensions.get_column_rect(column_index).as_physical(None),
                column.style.clone(),
            )));
        }
    }

    fn compute_border_collapse(&mut self, writing_mode: WritingMode) {
        if self.table.style.get_inherited_table().border_collapse != BorderCollapse::Collapse {
            self.collapsed_borders = None;
            return;
        }

        let mut collapsed_borders = LogicalVec2 {
            block: vec![
                vec![Default::default(); self.table.size.width];
                self.table.size.height + 1
            ],
            inline: vec![
                vec![Default::default(); self.table.size.height];
                self.table.size.width + 1
            ],
        };

        let apply_border = |collapsed_borders: &mut CollapsedBorders,
                            layout_style: &LayoutStyle,
                            block: &Range<usize>,
                            inline: &Range<usize>| {
            let border = CollapsedBorder::from_layout_style(layout_style, writing_mode);
            border
                .block_start
                .max_assign_to_slice(&mut collapsed_borders.block[block.start][inline.clone()]);
            border
                .block_end
                .max_assign_to_slice(&mut collapsed_borders.block[block.end][inline.clone()]);
            border
                .inline_start
                .max_assign_to_slice(&mut collapsed_borders.inline[inline.start][block.clone()]);
            border
                .inline_end
                .max_assign_to_slice(&mut collapsed_borders.inline[inline.end][block.clone()]);
        };
        let hide_inner_borders = |collapsed_borders: &mut CollapsedBorders,
                                  block: &Range<usize>,
                                  inline: &Range<usize>| {
            for x in inline.clone() {
                for y in block.clone() {
                    if x != inline.start {
                        collapsed_borders.inline[x][y].hide();
                    }
                    if y != block.start {
                        collapsed_borders.block[y][x].hide();
                    }
                }
            }
        };
        let all_rows = 0..self.table.size.height;
        let all_columns = 0..self.table.size.width;
        for row_index in all_rows.clone() {
            for column_index in all_columns.clone() {
                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => continue,
                };
                let block_range = row_index..row_index + cell.rowspan;
                let inline_range = column_index..column_index + cell.colspan;
                hide_inner_borders(&mut collapsed_borders, &block_range, &inline_range);
                apply_border(
                    &mut collapsed_borders,
                    &cell.layout_style(),
                    &block_range,
                    &inline_range,
                );
            }
        }
        for (row_index, row) in self.table.rows.iter().enumerate() {
            apply_border(
                &mut collapsed_borders,
                &row.layout_style(),
                &(row_index..row_index + 1),
                &all_columns,
            );
        }
        for row_group in &self.table.row_groups {
            apply_border(
                &mut collapsed_borders,
                &row_group.layout_style(),
                &row_group.track_range,
                &all_columns,
            );
        }
        for (column_index, column) in self.table.columns.iter().enumerate() {
            apply_border(
                &mut collapsed_borders,
                &column.layout_style(),
                &all_rows,
                &(column_index..column_index + 1),
            );
        }
        for column_group in &self.table.column_groups {
            apply_border(
                &mut collapsed_borders,
                &column_group.layout_style(),
                &all_rows,
                &column_group.track_range,
            );
        }
        apply_border(
            &mut collapsed_borders,
            &self.table.layout_style_for_grid(),
            &all_rows,
            &all_columns,
        );

        self.collapsed_borders = Some(collapsed_borders);
    }

    fn get_collapsed_border_widths_for_area(
        &self,
        area: LogicalSides<usize>,
    ) -> Option<LogicalSides<Au>> {
        let collapsed_borders = self.collapsed_borders.as_ref()?;
        let columns = || area.inline_start..area.inline_end;
        let rows = || area.block_start..area.block_end;
        let max_width = |slice: &[CollapsedBorder]| {
            let slice_widths = slice.iter().map(|collapsed_border| collapsed_border.width);
            slice_widths.max().unwrap_or_default()
        };
        Some(area.map_inline_and_block_axes(
            |column| max_width(&collapsed_borders.inline[*column][rows()]) / 2,
            |row| max_width(&collapsed_borders.block[*row][columns()]) / 2,
        ))
    }
}

struct RowFragmentLayout<'a> {
    row: &'a TableTrack,
    rect: LogicalRect<Au>,
    containing_block: ContainingBlock<'a>,
    positioning_context: Option<PositioningContext>,
    fragments: Vec<Fragment>,
}

impl<'a> RowFragmentLayout<'a> {
    fn new(
        table_row: &'a TableTrack,
        index: usize,
        dimensions: &TableAndTrackDimensions,
        table_style: &'a ComputedValues,
    ) -> Self {
        let rect = dimensions.get_row_rect(index);
        let containing_block = ContainingBlock {
            size: ContainingBlockSize {
                inline: rect.size.inline,
                block: SizeConstraint::Definite(rect.size.block),
            },
            style: table_style,
        };
        Self {
            row: table_row,
            rect,
            positioning_context: PositioningContext::new_for_style(&table_row.style),
            containing_block,
            fragments: Vec::new(),
        }
    }
    fn finish(
        mut self,
        layout_context: &LayoutContext,
        table_positioning_context: &mut PositioningContext,
        containing_block_for_logical_conversion: &ContainingBlock,
        containing_block_for_children: &ContainingBlock,
        row_group_fragment_layout: &mut Option<RowGroupFragmentLayout>,
    ) -> ArcRefCell<BoxFragment> {
        if self.positioning_context.is_some() {
            self.rect.start_corner +=
                relative_adjustement(&self.row.style, containing_block_for_children);
        }

        let (inline_size, block_size) = if let Some(row_group_layout) = row_group_fragment_layout {
            self.rect.start_corner -= row_group_layout.rect.start_corner;
            (
                row_group_layout.rect.size.inline,
                SizeConstraint::Definite(row_group_layout.rect.size.block),
            )
        } else {
            (
                containing_block_for_logical_conversion.size.inline,
                containing_block_for_logical_conversion.size.block,
            )
        };

        let row_group_containing_block = ContainingBlock {
            size: ContainingBlockSize {
                inline: inline_size,
                block: block_size,
            },
            style: containing_block_for_logical_conversion.style,
        };

        let mut row_fragment = BoxFragment::new(
            self.row.base_fragment_info,
            self.row.style.clone(),
            self.fragments,
            self.rect.as_physical(Some(&row_group_containing_block)),
            PhysicalSides::zero(), /* padding */
            PhysicalSides::zero(), /* border */
            PhysicalSides::zero(), /* margin */
            None,                  /* clearance */
            CollapsedBlockMargins::zero(),
        );
        row_fragment.set_does_not_paint_background();

        if let Some(mut row_positioning_context) = self.positioning_context.take() {
            row_positioning_context.layout_collected_children(layout_context, &mut row_fragment);

            let positioning_context = row_group_fragment_layout
                .as_mut()
                .and_then(|layout| layout.positioning_context.as_mut())
                .unwrap_or(table_positioning_context);
            positioning_context.append(row_positioning_context);
        }

        ArcRefCell::new(row_fragment)
    }
}

struct RowGroupFragmentLayout {
    base_fragment_info: BaseFragmentInfo,
    style: Arc<ComputedValues>,
    rect: LogicalRect<Au>,
    positioning_context: Option<PositioningContext>,
    index: usize,
    fragments: Vec<Fragment>,
}

impl RowGroupFragmentLayout {
    fn new(
        row_group: &TableTrackGroup,
        index: usize,
        dimensions: &TableAndTrackDimensions,
    ) -> Self {
        let rect = dimensions.get_row_group_rect(row_group);
        Self {
            base_fragment_info: row_group.base_fragment_info,
            style: row_group.style.clone(),
            rect,
            positioning_context: PositioningContext::new_for_style(&row_group.style),
            index,
            fragments: Vec::new(),
        }
    }

    fn finish(
        mut self,
        layout_context: &LayoutContext,
        table_positioning_context: &mut PositioningContext,
        containing_block_for_logical_conversion: &ContainingBlock,
        containing_block_for_children: &ContainingBlock,
    ) -> ArcRefCell<BoxFragment> {
        if self.positioning_context.is_some() {
            self.rect.start_corner +=
                relative_adjustement(&self.style, containing_block_for_children);
        }

        let mut row_group_fragment = BoxFragment::new(
            self.base_fragment_info,
            self.style,
            self.fragments,
            self.rect
                .as_physical(Some(containing_block_for_logical_conversion)),
            PhysicalSides::zero(), /* padding */
            PhysicalSides::zero(), /* border */
            PhysicalSides::zero(), /* margin */
            None,                  /* clearance */
            CollapsedBlockMargins::zero(),
        );
        row_group_fragment.set_does_not_paint_background();

        if let Some(mut row_positioning_context) = self.positioning_context.take() {
            row_positioning_context
                .layout_collected_children(layout_context, &mut row_group_fragment);
            table_positioning_context.append(row_positioning_context);
        }

        ArcRefCell::new(row_group_fragment)
    }
}

struct TableAndTrackDimensions {
    /// The rect of the full table, not counting for borders, padding, and margin.
    table_rect: LogicalRect<Au>,
    /// The rect of the full table, not counting for borders, padding, and margin
    /// and offset by any border spacing and caption.
    table_cells_rect: LogicalRect<Au>,
    /// The min and max block offsets of each table row.
    row_dimensions: Vec<(Au, Au)>,
    /// The min and max inline offsets of each table column
    column_dimensions: Vec<(Au, Au)>,
}

impl TableAndTrackDimensions {
    fn new(table_layout: &TableLayout) -> Self {
        let border_spacing = table_layout.table.border_spacing();

        // The sizes used for a dimension when that dimension has no table tracks.
        let fallback_inline_size = table_layout.assignable_width;
        let fallback_block_size = table_layout.final_table_height;

        let mut column_dimensions = Vec::new();
        let mut column_offset = Au::zero();
        for column_index in 0..table_layout.table.size.width {
            if table_layout.is_column_collapsed(column_index) {
                column_dimensions.push((column_offset, column_offset));
                continue;
            }
            let start_offset = column_offset + border_spacing.inline;
            let end_offset = start_offset + table_layout.distributed_column_widths[column_index];
            column_dimensions.push((start_offset, end_offset));
            column_offset = end_offset;
        }
        column_offset += if table_layout.table.size.width == 0 {
            fallback_inline_size
        } else {
            border_spacing.inline
        };

        let mut row_dimensions = Vec::new();
        let mut row_offset = Au::zero();
        for row_index in 0..table_layout.table.size.height {
            if table_layout.is_row_collapsed(row_index) {
                row_dimensions.push((row_offset, row_offset));
                continue;
            }
            let start_offset = row_offset + border_spacing.block;
            let end_offset = start_offset + table_layout.row_sizes[row_index];
            row_dimensions.push((start_offset, end_offset));
            row_offset = end_offset;
        }
        row_offset += if table_layout.table.size.height == 0 {
            fallback_block_size
        } else {
            border_spacing.block
        };

        let table_start_corner = LogicalVec2 {
            inline: column_dimensions.first().map_or_else(Au::zero, |v| v.0),
            block: row_dimensions.first().map_or_else(Au::zero, |v| v.0),
        };
        let table_size = LogicalVec2 {
            inline: column_dimensions
                .last()
                .map_or(fallback_inline_size, |v| v.1),
            block: row_dimensions.last().map_or(fallback_block_size, |v| v.1),
        } - table_start_corner;
        let table_cells_rect = LogicalRect {
            start_corner: table_start_corner,
            size: table_size,
        };

        let table_rect = LogicalRect {
            start_corner: LogicalVec2::zero(),
            size: LogicalVec2 {
                inline: column_offset,
                block: row_offset,
            },
        };

        Self {
            table_rect,
            table_cells_rect,
            row_dimensions,
            column_dimensions,
        }
    }

    fn get_row_rect(&self, row_index: usize) -> LogicalRect<Au> {
        let mut row_rect = self.table_cells_rect;
        let row_dimensions = self.row_dimensions[row_index];
        row_rect.start_corner.block = row_dimensions.0;
        row_rect.size.block = row_dimensions.1 - row_dimensions.0;
        row_rect
    }

    fn get_column_rect(&self, column_index: usize) -> LogicalRect<Au> {
        let mut row_rect = self.table_cells_rect;
        let column_dimensions = self.column_dimensions[column_index];
        row_rect.start_corner.inline = column_dimensions.0;
        row_rect.size.inline = column_dimensions.1 - column_dimensions.0;
        row_rect
    }

    fn get_row_group_rect(&self, row_group: &TableTrackGroup) -> LogicalRect<Au> {
        if row_group.is_empty() {
            return LogicalRect::zero();
        }

        let mut row_group_rect = self.table_cells_rect;
        let block_start = self.row_dimensions[row_group.track_range.start].0;
        let block_end = self.row_dimensions[row_group.track_range.end - 1].1;
        row_group_rect.start_corner.block = block_start;
        row_group_rect.size.block = block_end - block_start;
        row_group_rect
    }

    fn get_column_group_rect(&self, column_group: &TableTrackGroup) -> LogicalRect<Au> {
        if column_group.is_empty() {
            return LogicalRect::zero();
        }

        let mut column_group_rect = self.table_cells_rect;
        let inline_start = self.column_dimensions[column_group.track_range.start].0;
        let inline_end = self.column_dimensions[column_group.track_range.end - 1].1;
        column_group_rect.start_corner.inline = inline_start;
        column_group_rect.size.inline = inline_end - inline_start;
        column_group_rect
    }

    fn get_cell_rect(
        &self,
        coordinates: TableSlotCoordinates,
        rowspan: usize,
        colspan: usize,
    ) -> LogicalRect<Au> {
        let start_corner = LogicalVec2 {
            inline: self.column_dimensions[coordinates.x].0,
            block: self.row_dimensions[coordinates.y].0,
        };
        let size = LogicalVec2 {
            inline: self.column_dimensions[coordinates.x + colspan - 1].1,
            block: self.row_dimensions[coordinates.y + rowspan - 1].1,
        } - start_corner;
        LogicalRect { start_corner, size }
    }
}

impl Table {
    fn border_spacing(&self) -> LogicalVec2<Au> {
        if self.style.clone_border_collapse() == BorderCollapse::Collapse {
            LogicalVec2::zero()
        } else {
            let border_spacing = self.style.clone_border_spacing();
            LogicalVec2 {
                inline: border_spacing.horizontal(),
                block: border_spacing.vertical(),
            }
        }
    }

    fn total_border_spacing(&self) -> LogicalVec2<Au> {
        let border_spacing = self.border_spacing();
        LogicalVec2 {
            inline: if self.size.width > 0 {
                border_spacing.inline * (self.size.width as i32 + 1)
            } else {
                Au::zero()
            },
            block: if self.size.height > 0 {
                border_spacing.block * (self.size.height as i32 + 1)
            } else {
                Au::zero()
            },
        }
    }

    fn get_column_measure_for_column_at_index(
        &self,
        writing_mode: WritingMode,
        column_index: usize,
        is_in_fixed_mode: bool,
    ) -> CellOrTrackMeasure {
        let column = match self.columns.get(column_index) {
            Some(column) => column,
            None => return CellOrTrackMeasure::zero(),
        };

        let CellOrColumnOuterSizes {
            preferred: preferred_size,
            min: min_size,
            max: max_size,
            percentage: percentage_size,
        } = CellOrColumnOuterSizes::new(
            &column.style,
            writing_mode,
            &Default::default(),
            is_in_fixed_mode,
        );

        CellOrTrackMeasure {
            content_sizes: ContentSizes {
                // > The outer min-content width of a table-column or table-column-group is
                // > max(min-width, width).
                // But that's clearly wrong, since it would be equal to or greater than
                // the outer max-content width. So we match other browsers instead.
                min_content: min_size.inline,
                // > The outer max-content width of a table-column or table-column-group is
                // > max(min-width, min(max-width, width)).
                // This matches Gecko, but Blink and WebKit ignore max_size.
                max_content: preferred_size
                    .inline
                    .clamp_between_extremums(min_size.inline, max_size.inline),
            },
            percentage: percentage_size.inline,
        }
    }

    fn get_row_measure_for_row_at_index(
        &self,
        writing_mode: WritingMode,
        row_index: usize,
    ) -> CellOrTrackMeasure {
        let row = match self.rows.get(row_index) {
            Some(row) => row,
            None => return CellOrTrackMeasure::zero(),
        };

        // In the block axis, the min-content and max-content sizes are the same
        // (except for new layout boxes like grid and flex containers). Note that
        // other browsers don't seem to use the min and max sizing properties here.
        let size = row.style.box_size(writing_mode);
        let max_size = row.style.max_box_size(writing_mode);
        let percentage_contribution = get_size_percentage_contribution(&size, &max_size);

        CellOrTrackMeasure {
            content_sizes: size
                .block
                .to_numeric()
                .and_then(|size| size.to_length())
                .map_or_else(Au::zero, Au::from)
                .into(),
            percentage: percentage_contribution.block,
        }
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
        depends_on_block_constraints: bool,
    ) -> CacheableLayoutResult {
        TableLayout::new(self).layout(
            layout_context,
            positioning_context,
            containing_block_for_children,
            containing_block_for_table,
            depends_on_block_constraints,
        )
    }
}

impl ComputeInlineContentSizes for Table {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "Table::compute_inline_content_sizes",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        let writing_mode = constraint_space.writing_mode;
        let mut layout = TableLayout::new(self);
        layout.compute_border_collapse(writing_mode);
        layout.pbm = self
            .layout_style(Some(&layout))
            .padding_border_margin_with_writing_mode_and_containing_block_inline_size(
                writing_mode,
                Au::zero(),
            );
        layout.compute_measures(layout_context, writing_mode);

        let grid_content_sizes = layout.compute_grid_min_max();

        // Padding and border should apply to the table grid, but they will be taken into
        // account when computing the inline content sizes of the table wrapper (our parent), so
        // this code removes their contribution from the inline content size of the caption.
        let caption_content_sizes = ContentSizes::from(
            layout.compute_caption_minimum_inline_size(layout_context) -
                layout.pbm.padding_border_sums.inline,
        );

        InlineContentSizesResult {
            sizes: grid_content_sizes.max(caption_content_sizes),
            depends_on_block_constraints: false,
        }
    }
}

impl Table {
    #[inline]
    pub(crate) fn layout_style<'a>(
        &'a self,
        layout: Option<&'a TableLayout<'a>>,
    ) -> LayoutStyle<'a> {
        LayoutStyle::Table(TableLayoutStyle {
            table: self,
            layout,
        })
    }

    #[inline]
    pub(crate) fn layout_style_for_grid(&self) -> LayoutStyle {
        LayoutStyle::Default(&self.grid_style)
    }
}

impl TableTrack {
    #[inline]
    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle::Default(&self.style)
    }
}

impl TableTrackGroup {
    #[inline]
    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle::Default(&self.style)
    }
}

impl TableLayoutStyle<'_> {
    #[inline]
    pub(crate) fn style(&self) -> &ComputedValues {
        &self.table.style
    }

    #[inline]
    pub(crate) fn collapses_borders(&self) -> bool {
        self.style().get_inherited_table().border_collapse == BorderCollapse::Collapse
    }

    pub(crate) fn halved_collapsed_border_widths(&self) -> LogicalSides<Au> {
        debug_assert!(self.collapses_borders());
        let area = LogicalSides {
            inline_start: 0,
            inline_end: self.table.size.width,
            block_start: 0,
            block_end: self.table.size.height,
        };
        if let Some(layout) = self.layout {
            layout.get_collapsed_border_widths_for_area(area)
        } else {
            // TODO: this should be cached.
            let mut layout = TableLayout::new(self.table);
            layout.compute_border_collapse(self.style().writing_mode);
            layout.get_collapsed_border_widths_for_area(area)
        }
        .expect("Collapsed borders should be computed")
    }
}

impl TableSlotCell {
    #[inline]
    fn layout_style(&self) -> LayoutStyle {
        self.contents.layout_style(&self.base)
    }

    fn effective_vertical_align(&self) -> VerticalAlignKeyword {
        match self.base.style.clone_vertical_align() {
            VerticalAlign::Keyword(VerticalAlignKeyword::Top) => VerticalAlignKeyword::Top,
            VerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => VerticalAlignKeyword::Bottom,
            VerticalAlign::Keyword(VerticalAlignKeyword::Middle) => VerticalAlignKeyword::Middle,
            _ => VerticalAlignKeyword::Baseline,
        }
    }

    fn inline_content_sizes(&self, layout_context: &LayoutContext) -> ContentSizes {
        let constraint_space = ConstraintSpace::new_for_style_and_ratio(
            &self.base.style,
            None, /* TODO: support preferred aspect ratios on non-replaced boxes */
        );
        self.base
            .inline_content_sizes(layout_context, &constraint_space, &self.contents.contents)
            .sizes
    }

    fn create_fragment(
        &self,
        mut layout: CellLayout,
        cell_rect: LogicalRect<Au>,
        cell_baseline: Au,
        positioning_context: &mut PositioningContext,
        table_style: &ComputedValues,
        containing_block: &ContainingBlock,
    ) -> BoxFragment {
        // This must be scoped to this function because it conflicts with euclid's Zero.
        use style::Zero as StyleZero;

        let cell_content_rect = cell_rect.deflate(&(layout.padding + layout.border));
        let content_block_size = layout.layout.content_block_size;
        let vertical_align_offset = match self.effective_vertical_align() {
            VerticalAlignKeyword::Top => Au::zero(),
            VerticalAlignKeyword::Bottom => cell_content_rect.size.block - content_block_size,
            VerticalAlignKeyword::Middle => {
                (cell_content_rect.size.block - content_block_size).scale_by(0.5)
            },
            _ => {
                cell_baseline -
                    (layout.padding.block_start + layout.border.block_start) -
                    layout.ascent()
            },
        };

        let mut base_fragment_info = self.base.base_fragment_info;
        if self.base.style.get_inherited_table().empty_cells == EmptyCells::Hide &&
            table_style.get_inherited_table().border_collapse != BorderCollapse::Collapse &&
            layout.is_empty_for_empty_cells()
        {
            base_fragment_info.flags.insert(FragmentFlags::DO_NOT_PAINT);
        }

        // Create an `AnonymousFragment` to move the cell contents to the cell baseline.
        let mut vertical_align_fragment_rect = cell_content_rect;
        vertical_align_fragment_rect.start_corner = LogicalVec2 {
            inline: Au::zero(),
            block: vertical_align_offset,
        };
        let vertical_align_fragment = PositioningFragment::new_anonymous(
            vertical_align_fragment_rect.as_physical(None),
            layout.layout.fragments,
        );

        // Adjust the static position of all absolute children based on the
        // final content rect of this fragment. Note that we are not shifting by the position of the
        // Anonymous fragment we use to shift content to the baseline.
        //
        // TODO(mrobinson): This is correct for absolutes that are direct children of the table
        // cell, but wrong for absolute fragments that are more deeply nested in the hierarchy of
        // fragments.
        let physical_cell_rect = cell_content_rect.as_physical(Some(containing_block));
        layout
            .positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(
                &physical_cell_rect.origin.to_vector(),
                PositioningContextLength::zero(),
            );
        positioning_context.append(layout.positioning_context);

        let specific_layout_info = (table_style.get_inherited_table().border_collapse ==
            BorderCollapse::Collapse)
            .then_some(SpecificLayoutInfo::TableCellWithCollapsedBorders);

        BoxFragment::new(
            base_fragment_info,
            self.base.style.clone(),
            vec![Fragment::Positioning(vertical_align_fragment)],
            physical_cell_rect,
            layout.padding.to_physical(table_style.writing_mode),
            layout.border.to_physical(table_style.writing_mode),
            PhysicalSides::zero(), /* margin */
            None,                  /* clearance */
            CollapsedBlockMargins::zero(),
        )
        .with_baselines(layout.layout.baselines)
        .with_specific_layout_info(specific_layout_info)
    }
}

fn get_size_percentage_contribution(
    size: &LogicalVec2<Size<ComputedLengthPercentage>>,
    max_size: &LogicalVec2<Size<ComputedLengthPercentage>>,
) -> LogicalVec2<Option<Percentage>> {
    // From <https://drafts.csswg.org/css-tables/#percentage-contribution>
    // > The percentage contribution of a table cell, column, or column group is defined
    // > in terms of the computed values of width and max-width that have computed values
    // > that are percentages:
    // >    min(percentage width, percentage max-width).
    // > If the computed values are not percentages, then 0% is used for width, and an
    // > infinite percentage is used for max-width.
    LogicalVec2 {
        inline: max_two_optional_percentages(
            size.inline.to_percentage(),
            max_size.inline.to_percentage(),
        ),
        block: max_two_optional_percentages(
            size.block.to_percentage(),
            max_size.block.to_percentage(),
        ),
    }
}

struct CellOrColumnOuterSizes {
    min: LogicalVec2<Au>,
    preferred: LogicalVec2<Au>,
    max: LogicalVec2<Option<Au>>,
    percentage: LogicalVec2<Option<Percentage>>,
}

impl CellOrColumnOuterSizes {
    fn new(
        style: &Arc<ComputedValues>,
        writing_mode: WritingMode,
        padding_border_sums: &LogicalVec2<Au>,
        is_in_fixed_mode: bool,
    ) -> Self {
        let box_sizing = style.get_position().box_sizing;
        let outer_size = |size: LogicalVec2<Au>| match box_sizing {
            BoxSizing::ContentBox => size + *padding_border_sums,
            BoxSizing::BorderBox => LogicalVec2 {
                inline: size.inline.max(padding_border_sums.inline),
                block: size.block.max(padding_border_sums.block),
            },
        };

        let outer_option_size = |size: LogicalVec2<Option<Au>>| match box_sizing {
            BoxSizing::ContentBox => size.map_inline_and_block_axes(
                |inline| inline.map(|inline| inline + padding_border_sums.inline),
                |block| block.map(|block| block + padding_border_sums.block),
            ),
            BoxSizing::BorderBox => size.map_inline_and_block_axes(
                |inline| inline.map(|inline| inline.max(padding_border_sums.inline)),
                |block| block.map(|block| block.max(padding_border_sums.block)),
            ),
        };

        let get_size_for_axis = |size: &Size<ComputedLengthPercentage>| {
            // Note that measures treat all size values other than <length>
            // as the initial value of the property.
            size.to_numeric()
                .and_then(|length_percentage| length_percentage.to_length())
                .map(Au::from)
        };

        let size = style.box_size(writing_mode);
        if is_in_fixed_mode {
            return Self {
                percentage: size.map(|v| v.to_percentage()),
                preferred: outer_option_size(size.map(get_size_for_axis))
                    .map(|v| v.unwrap_or_default()),
                min: LogicalVec2::default(),
                max: LogicalVec2::default(),
            };
        }

        let min_size = style.min_box_size(writing_mode);
        let max_size = style.max_box_size(writing_mode);

        Self {
            min: outer_size(min_size.map(|v| get_size_for_axis(v).unwrap_or_default())),
            preferred: outer_size(size.map(|v| get_size_for_axis(v).unwrap_or_default())),
            max: outer_option_size(max_size.map(get_size_for_axis)),
            percentage: get_size_percentage_contribution(&size, &max_size),
        }
    }
}

struct RowspanToDistribute<'a> {
    coordinates: TableSlotCoordinates,
    cell: &'a TableSlotCell,
    measure: &'a CellOrTrackMeasure,
}

impl RowspanToDistribute<'_> {
    fn range(&self) -> Range<usize> {
        self.coordinates.y..self.coordinates.y + self.cell.rowspan
    }

    fn fully_encloses(&self, other: &RowspanToDistribute) -> bool {
        other.coordinates.y > self.coordinates.y && other.range().end < self.range().end
    }
}

/// The inline size constraints provided by a cell that span multiple columns (`colspan` > 1).
/// These constraints are distributed to the individual columns that make up this cell's span.
#[derive(Debug)]
struct ColspanToDistribute {
    starting_column: usize,
    span: usize,
    content_sizes: ContentSizes,
    percentage: Option<Percentage>,
}

impl ColspanToDistribute {
    /// A comparison function to sort the colspan cell constraints primarily by their span
    /// width and secondarily by their starting column. This is not an implementation of
    /// `PartialOrd` because we want to return [`Ordering::Equal`] even if `self != other`.
    fn comparison_for_sort(a: &Self, b: &Self) -> Ordering {
        a.span
            .cmp(&b.span)
            .then_with(|| b.starting_column.cmp(&b.starting_column))
    }

    fn range(&self) -> Range<usize> {
        self.starting_column..self.starting_column + self.span
    }
}
