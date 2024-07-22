/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use app_units::{Au, MAX_AU};
use log::warn;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use servo_arc::Arc;
use style::computed_values::border_collapse::T as BorderCollapse;
use style::computed_values::box_sizing::T as BoxSizing;
use style::computed_values::caption_side::T as CaptionSide;
use style::computed_values::empty_cells::T as EmptyCells;
use style::computed_values::visibility::T as Visibility;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage as ComputedLengthPercentage, Percentage};
use style::values::generics::box_::{GenericVerticalAlign as VerticalAlign, VerticalAlignKeyword};
use style::values::generics::length::GenericLengthPercentageOrAuto::{Auto, LengthPercentage};
use style::Zero;

use super::{Table, TableCaption, TableSlot, TableSlotCell, TableTrack, TableTrackGroup};
use crate::context::LayoutContext;
use crate::formatting_contexts::{Baselines, IndependentLayout};
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, ExtraBackground, Fragment, FragmentFlags,
    PositioningFragment,
};
use crate::geom::{AuOrAuto, LengthPercentageOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{relative_adjustement, PositioningContext, PositioningContextLength};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::table::TableSlotCoordinates;
use crate::ContainingBlock;

/// A result of a final or speculative layout of a single cell in
/// the table. Note that this is only done for slots that are not
/// covered by spans or empty.
struct CellLayout {
    layout: IndependentLayout,
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
        !self
            .layout
            .fragments
            .iter()
            .any(|fragment| !matches!(fragment, Fragment::AbsoluteOrFixedPositioned(_)))
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
}

/// The calculated collapsed borders.
#[derive(Clone, Debug, Default)]
struct CollapsedBorders {
    block: Vec<Au>,
    inline: Vec<Au>,
}

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
    column_measures: Vec<CellOrTrackMeasure>,
    distributed_column_widths: Vec<Au>,
    row_sizes: Vec<Au>,
    /// The accumulated baseline of each row, relative to the top of the row.
    row_baselines: Vec<Au>,
    cells_laid_out: Vec<Vec<Option<CellLayout>>>,
    basis_for_cell_padding_percentage: Au,
    /// Information about collapsed borders.
    collapsed_borders: Option<CollapsedBorders>,
}

#[derive(Clone, Debug)]
struct CellOrTrackMeasure {
    content_sizes: ContentSizes,
    percentage: Percentage,
}

impl Zero for CellOrTrackMeasure {
    fn zero() -> Self {
        Self {
            content_sizes: ContentSizes::zero(),
            percentage: Percentage(0.),
        }
    }

    fn is_zero(&self) -> bool {
        self.content_sizes.is_zero() && self.percentage.is_zero()
    }
}

impl<'a> TableLayout<'a> {
    fn new(table: &'a Table) -> TableLayout {
        Self {
            table,
            pbm: PaddingBorderMargin::zero(),
            rows: Vec::new(),
            columns: Vec::new(),
            cell_measures: Vec::new(),
            table_width: Au::zero(),
            assignable_width: Au::zero(),
            final_table_height: Au::zero(),
            column_measures: Vec::new(),
            distributed_column_widths: Vec::new(),
            row_sizes: Vec::new(),
            row_baselines: Vec::new(),
            cells_laid_out: Vec::new(),
            basis_for_cell_padding_percentage: Au::zero(),
            collapsed_borders: None,
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

                let padding = cell
                    .style
                    .padding(writing_mode)
                    .percentages_relative_to(Length::zero());

                let border = self
                    .get_collapsed_borders_for_cell(
                        cell,
                        TableSlotCoordinates::new(column_index, row_index),
                    )
                    .unwrap_or_else(|| cell.style.border_width(writing_mode));

                let padding_border_sums = LogicalVec2 {
                    inline: (padding.inline_sum() + border.inline_sum()).into(),
                    block: (padding.block_sum() + border.block_sum()).into(),
                };

                let (size, min_size, max_size) =
                    get_outer_sizes_from_style(&cell.style, writing_mode, &padding_border_sums);
                let mut inline_content_sizes = cell
                    .contents
                    .contents
                    .inline_content_sizes(layout_context, writing_mode);
                inline_content_sizes.min_content += padding_border_sums.inline;
                inline_content_sizes.max_content += padding_border_sums.inline;

                // TODO: the max-content size should never be smaller than the min-content size!
                inline_content_sizes.max_content = inline_content_sizes
                    .max_content
                    .max(inline_content_sizes.min_content);

                let percentage_contribution =
                    get_size_percentage_contribution_from_style(&cell.style, writing_mode);

                // These formulas differ from the spec, but seem to match Gecko and Blink.
                let outer_min_content_width = inline_content_sizes
                    .min_content
                    .min(max_size.inline)
                    .max(min_size.inline);
                let outer_max_content_width = if self.columns[column_index].constrained {
                    inline_content_sizes
                        .min_content
                        .max(size.inline)
                        .min(max_size.inline)
                        .max(min_size.inline)
                } else {
                    inline_content_sizes
                        .max_content
                        .max(size.inline)
                        .min(max_size.inline)
                        .max(min_size.inline)
                };
                assert!(outer_min_content_width <= outer_max_content_width);

                let inline_measure = CellOrTrackMeasure {
                    content_sizes: ContentSizes {
                        min_content: outer_min_content_width,
                        max_content: outer_max_content_width,
                    },
                    percentage: percentage_contribution.inline,
                };

                // This measure doesn't take into account the `min-content` and `max-content` sizes.
                // These sizes are incorporated after the first row layout pass, when the block size
                // of the layout is known.
                let block_measure = CellOrTrackMeasure {
                    content_sizes: ContentSizes {
                        min_content: size.block,
                        max_content: size.block,
                    },
                    percentage: percentage_contribution.block,
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

        for column_index in 0..self.table.size.width {
            if let Some(column) = self.table.columns.get(column_index) {
                if !column.style.box_size(writing_mode).inline.is_auto() {
                    self.columns[column_index].constrained = true;
                    continue;
                }
                if let Some(column_group_index) = column.group_index {
                    let column_group = &self.table.column_groups[column_group_index];
                    if !column_group.style.box_size(writing_mode).inline.is_auto() {
                        self.columns[column_index].constrained = true;
                        continue;
                    }
                }
                self.columns[column_index].constrained = false;
            }
        }

        for row_index in 0..self.table.size.height {
            if let Some(row) = self.table.rows.get(row_index) {
                if !row.style.box_size(writing_mode).block.is_auto() {
                    self.rows[row_index].constrained = true;
                    continue;
                }
                if let Some(row_group_index) = row.group_index {
                    let row_group = &self.table.row_groups[row_group_index];
                    if !row_group.style.box_size(writing_mode).block.is_auto() {
                        self.rows[row_index].constrained = true;
                        continue;
                    }
                }
            }
            self.rows[row_index].constrained = false;
        }

        for column_index in 0..self.table.size.width {
            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                let cell_constrained = match self.table.resolve_first_cell(coords) {
                    Some(cell) if cell.colspan == 1 => cell
                        .style
                        .box_size(writing_mode)
                        .inline
                        .non_auto()
                        .and_then(|length_percentage| length_percentage.to_length())
                        .is_some(),
                    _ => false,
                };

                let rowspan_greater_than_1 = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell.rowspan > 1,
                    _ => false,
                };

                self.rows[row_index].has_cell_with_span_greater_than_one |= rowspan_greater_than_1;
                self.rows[row_index].constrained |= cell_constrained;

                let has_originating_cell =
                    matches!(self.table.get_slot(coords), Some(TableSlot::Cell(_)));
                self.columns[column_index].has_originating_cells |= has_originating_cell;
                self.columns[column_index].constrained |= cell_constrained;
            }
        }
    }

    /// This is an implementation of *Computing Column Measures* from
    /// <https://drafts.csswg.org/css-tables/#computing-column-measures>.
    fn compute_column_measures(&mut self, writing_mode: WritingMode) {
        let mut column_measures = Vec::new();

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
        let mut next_span_n = usize::MAX;
        for column_index in 0..self.table.size.width {
            let mut column_measure = self
                .table
                .get_column_measure_for_column_at_index(writing_mode, column_index);

            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                match self.table.resolve_first_cell(coords) {
                    Some(cell) if cell.colspan == 1 => cell,
                    Some(cell) => {
                        next_span_n = next_span_n.min(cell.colspan);
                        continue;
                    },
                    _ => continue,
                };

                // This takes the max of `min_content`, `max_content`, and
                // intrinsic percentage width as described above.
                let cell_measure = &self.cell_measures[row_index][column_index].inline;
                column_measure
                    .content_sizes
                    .max_assign(cell_measure.content_sizes);
                column_measure.percentage =
                    Percentage(column_measure.percentage.0.max(cell_measure.percentage.0));
            }

            column_measures.push(column_measure);
        }

        // Now we have the base computation complete, so iteratively take into account cells
        // with higher colspan. Using `next_span_n` we can skip over span counts that don't
        // correspond to any cells.
        while next_span_n < usize::MAX {
            (next_span_n, column_measures) = self
                .compute_content_sizes_for_columns_with_span_up_to_n(next_span_n, &column_measures);
        }

        // > intrinsic percentage width of a column:
        // > the smaller of:
        // >   * the intrinsic percentage width of the column based on cells of span up to N,
        // >     where N is the number of columns in the table
        // >   * 100% minus the sum of the intrinsic percentage width of all prior columns in
        // >     the table (further left when direction is "ltr" (right for "rtl"))
        let mut total_intrinsic_percentage_width = 0.;
        for column_measure in column_measures.iter_mut() {
            let final_intrinsic_percentage_width = column_measure
                .percentage
                .0
                .min(1. - total_intrinsic_percentage_width);
            total_intrinsic_percentage_width += final_intrinsic_percentage_width;
            column_measure.percentage = Percentage(final_intrinsic_percentage_width);
        }

        self.column_measures = column_measures;
    }

    fn compute_content_sizes_for_columns_with_span_up_to_n(
        &self,
        n: usize,
        old_column_measures: &[CellOrTrackMeasure],
    ) -> (usize, Vec<CellOrTrackMeasure>) {
        let mut next_span_n = usize::MAX;
        let mut new_content_sizes_for_columns = Vec::new();
        let border_spacing = self.table.border_spacing();

        for column_index in 0..self.table.size.width {
            let old_column_measure = &old_column_measures[column_index];
            let mut new_column_content_sizes = old_column_measure.content_sizes;
            let mut new_column_intrinsic_percentage_width = old_column_measure.percentage;

            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                let resolved_coords = match self.table.resolve_first_cell_coords(coords) {
                    Some(resolved_coords) => resolved_coords,
                    None => continue,
                };

                let cell = match self.table.resolve_first_cell(resolved_coords) {
                    Some(cell) if cell.colspan <= n => cell,
                    Some(cell) => {
                        next_span_n = next_span_n.min(cell.colspan);
                        continue;
                    },
                    _ => continue,
                };

                let cell_measures =
                    &self.cell_measures[resolved_coords.y][resolved_coords.x].inline;
                let cell_inline_content_sizes = cell_measures.content_sizes;

                let columns_spanned = resolved_coords.x..resolved_coords.x + cell.colspan;
                let baseline_content_sizes: ContentSizes = columns_spanned.clone().fold(
                    ContentSizes::zero(),
                    |total: ContentSizes, spanned_column_index| {
                        total + old_column_measures[spanned_column_index].content_sizes
                    },
                );

                let old_column_content_size = old_column_measure.content_sizes;

                // > **min-content width of a column based on cells of span up to N (N > 1)**
                // >
                // > the largest of the min-content width of the column based on cells of span up to
                // > N-1 and the contributions of the cells in the column whose colSpan is N, where
                // > the contribution of a cell is the result of taking the following steps:
                // >
                // >     1. Define the baseline min-content width as the sum of the max-content
                // >        widths based on cells of span up to N-1 of all columns that the cell spans.
                //
                // Note: This definition is likely a typo, so we use the sum of the min-content
                // widths here instead.
                let baseline_min_content_width = baseline_content_sizes.min_content;
                let baseline_max_content_width = baseline_content_sizes.max_content;

                // >     2. Define the baseline border spacing as the sum of the horizontal
                // >        border-spacing for any columns spanned by the cell, other than the one in
                // >        which the cell originates.
                let baseline_border_spacing = border_spacing.inline * (n as i32 - 1);

                // >     3. The contribution of the cell is the sum of:
                // >         a. the min-content width of the column based on cells of span up to N-1
                let a = old_column_content_size.min_content;

                // >         b. the product of:
                // >             - the ratio of:
                // >                 - the max-content width of the column based on cells of span up
                // >                   to N-1 of the column minus the min-content width of the
                // >                   column based on cells of span up to N-1 of the column, to
                // >                 - the baseline max-content width minus the baseline min-content
                // >                   width
                // >               or zero if this ratio is undefined, and
                // >             - the outer min-content width of the cell minus the baseline
                // >               min-content width and the baseline border spacing, clamped to be
                // >               at least 0 and at most the difference between the baseline
                // >               max-content width and the baseline min-content width
                let old_content_size_difference =
                    old_column_content_size.max_content - old_column_content_size.min_content;
                let baseline_difference = baseline_min_content_width - baseline_max_content_width;

                let mut b =
                    old_content_size_difference.to_f32_px() / baseline_difference.to_f32_px();
                if !b.is_finite() {
                    b = 0.0;
                }
                let b = (cell_inline_content_sizes.min_content -
                    baseline_content_sizes.min_content -
                    baseline_border_spacing)
                    .clamp_between_extremums(Au::zero(), Some(baseline_difference))
                    .scale_by(b);

                // >         c. the product of:
                // >             - the ratio of the max-content width based on cells of span up to
                // >               N-1 of the column to the baseline max-content width
                // >             - the outer min-content width of the cell minus the baseline
                // >               max-content width and baseline border spacing, or 0 if this is
                // >               negative
                let c = (cell_inline_content_sizes.min_content -
                    baseline_content_sizes.max_content -
                    baseline_border_spacing)
                    .min(Au::zero())
                    .scale_by(
                        old_column_content_size.max_content.to_f32_px() /
                            baseline_content_sizes.max_content.to_f32_px(),
                    );

                let new_column_min_content_width = a + b + c;

                // > **max-content width of a column based on cells of span up to N (N > 1)**
                // >
                // > The largest of the max-content width based on cells of span up to N-1 and the
                // > contributions of the cells in the column whose colSpan is N, where the
                // > contribution of a cell is the result of taking the following steps:

                // >     1. Define the baseline max-content width as the sum of the max-content
                // >        widths based on cells of span up to N-1 of all columns that the cell spans.
                //
                // This is calculated above for the min-content width.

                // >     2. Define the baseline border spacing as the sum of the horizontal
                // >        border-spacing for any columns spanned by the cell, other than the one in
                // >        which the cell originates.
                //
                // This is calculated above for min-content width.

                // >     3. The contribution of the cell is the sum of:
                // >          a. the max-content width of the column based on cells of span up to N-1
                let a = old_column_content_size.max_content;

                // >          b. the product of:
                // >              1. the ratio of the max-content width based on cells of span up to
                // >                 N-1 of the column to the baseline max-content width
                let b_1 = old_column_content_size.max_content.to_f32_px() /
                    baseline_content_sizes.max_content.to_f32_px();

                // >              2. the outer max-content width of the cell minus the baseline
                // >                 max-content width and the baseline border spacing, or 0 if this
                // >                 is negative
                let b_2 = (cell_inline_content_sizes.max_content -
                    baseline_content_sizes.max_content -
                    baseline_border_spacing)
                    .min(Au::zero());
                let b = b_2.scale_by(b_1);
                let new_column_max_content_width = a + b + c;

                // The computed values for the column are always the largest of any processed cell
                // in that column.
                new_column_content_sizes.max_assign(ContentSizes {
                    min_content: new_column_min_content_width,
                    max_content: new_column_max_content_width,
                });

                // > If the intrinsic percentage width of a column based on cells of span up to N-1 is
                // > greater than 0%, then the intrinsic percentage width of the column based on cells
                // > of span up to N is the same as the intrinsic percentage width of the column based
                // > on cells of span up to N-1.
                // > Otherwise, it is the largest of the contributions of the cells in the column
                // > whose colSpan is N, where the contribution of a cell is the result of taking
                // > the following steps:
                if old_column_measure.percentage.0 <= 0. && cell_measures.percentage.0 != 0. {
                    // > 1. Start with the percentage contribution of the cell.
                    // > 2. Subtract the intrinsic percentage width of the column based on cells
                    // >    of span up to N-1 of all columns that the cell spans. If this gives a
                    // >    negative result, change it to 0%.
                    let mut spanned_columns_with_zero = 0;
                    let other_column_percentages_sum =
                        (columns_spanned).fold(0., |sum, spanned_column_index| {
                            let spanned_column_percentage =
                                old_column_measures[spanned_column_index].percentage;
                            if spanned_column_percentage.0 == 0. {
                                spanned_columns_with_zero += 1;
                            }
                            sum + spanned_column_percentage.0
                        });
                    let step_2 = (cell_measures.percentage -
                        Percentage(other_column_percentages_sum))
                    .clamp_to_non_negative();

                    // > Multiply by the ratio of:
                    // >  1. the column’s non-spanning max-content width to
                    // >  2. the sum of the non-spanning max-content widths of all columns
                    // >      spanned by the cell that have an intrinsic percentage width of the column
                    // >      based on cells of span up to N-1 equal to 0%.
                    // > However, if this ratio is undefined because the denominator is zero,
                    // > instead use the 1 divided by the number of columns spanned by the cell
                    // > that have an intrinsic percentage width of the column based on cells of
                    // > span up to N-1 equal to zero.
                    let step_3 = step_2.0 * (1.0 / spanned_columns_with_zero as f32);

                    new_column_intrinsic_percentage_width =
                        Percentage(new_column_intrinsic_percentage_width.0.max(step_3));
                }
            }
            new_content_sizes_for_columns.push(CellOrTrackMeasure {
                content_sizes: new_column_content_sizes,
                percentage: new_column_intrinsic_percentage_width,
            });
        }
        (next_span_n, new_content_sizes_for_columns)
    }

    /// Compute the GRIDMIN and GRIDMAX.
    fn compute_grid_min_max(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        self.compute_track_constrainedness_and_has_originating_cells(writing_mode);
        self.compute_border_collapse(writing_mode);
        self.compute_cell_measures(layout_context, writing_mode);
        self.compute_column_measures(writing_mode);

        // https://drafts.csswg.org/css-tables/#gridmin:
        // > The row/column-grid width minimum (GRIDMIN) width is the sum of the min-content width of
        // > all the columns plus cell spacing or borders.
        // https://drafts.csswg.org/css-tables/#gridmax:
        // > The row/column-grid width maximum (GRIDMAX) width is the sum of the max-content width of
        // > all the columns plus cell spacing or borders.
        let mut grid_min_max = self
            .column_measures
            .iter()
            .fold(ContentSizes::zero(), |result, measure| {
                result + measure.content_sizes
            });

        // TODO: GRIDMAX should never be smaller than GRIDMIN!
        grid_min_max
            .max_content
            .max_assign(grid_min_max.min_content);

        let inline_border_spacing = self.table.total_border_spacing().inline;
        grid_min_max.min_content += inline_border_spacing;
        grid_min_max.max_content += inline_border_spacing;
        grid_min_max
    }

    /// Compute CAPMIN: <https://drafts.csswg.org/css-tables/#capmin>
    fn compute_caption_minimum_inline_size(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> Au {
        self.table
            .captions
            .iter()
            .map(|caption| {
                let size;
                let min_size;
                let max_size;
                let padding_border_sums;
                let size_is_auto;
                {
                    let context = caption.context.borrow();
                    let padding = context
                        .style
                        .padding(writing_mode)
                        .percentages_relative_to(Length::zero());
                    let border = context.style.border_width(writing_mode);
                    let margin = context
                        .style
                        .margin(writing_mode)
                        .percentages_relative_to(Length::zero())
                        .auto_is(Length::zero);

                    padding_border_sums = LogicalVec2 {
                        inline: (padding.inline_sum() + border.inline_sum() + margin.inline_sum())
                            .into(),
                        block: (padding.block_sum() + border.block_sum() + margin.block_sum())
                            .into(),
                    };

                    (size, min_size, max_size) = get_outer_sizes_from_style(
                        &context.style,
                        writing_mode,
                        &padding_border_sums,
                    );
                    size_is_auto = context.style.box_size(writing_mode).inline.is_auto();
                }

                // If an inline size is defined it should serve as the upper limit and lower limit
                // of the caption inline size.
                let inline_size = if !size_is_auto {
                    size.inline
                } else {
                    let inline_content_sizes = caption
                        .context
                        .borrow_mut()
                        .inline_content_sizes(layout_context);
                    inline_content_sizes.min_content + padding_border_sums.inline
                };

                inline_size.min(max_size.inline).max(min_size.inline)
            })
            .max()
            .unwrap_or_default()
    }

    fn compute_table_width(
        &mut self,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
        grid_min_max: ContentSizes,
        caption_minimum_inline_size: Au,
    ) {
        let style = &self.table.style;
        self.pbm = style.padding_border_margin(containing_block_for_table);

        // https://drafts.csswg.org/css-tables/#resolved-table-width
        // * If inline-size computes to 'auto', this is the stretch-fit size
        //   (https://drafts.csswg.org/css-sizing-3/#stretch-fit-size).
        // * Otherwise, it's the resulting length (with percentages resolved).
        // In both cases, it's clamped between min-inline-size and max-inline-size.
        // This diverges a little from the specification.
        let resolved_table_width = containing_block_for_children.inline_size;

        // https://drafts.csswg.org/css-tables/#used-width-of-table
        // * If table-root has a computed value for inline-size different than auto:
        //   use the maximum of the resolved table width, GRIDMIN and CAPMIN.
        // * If auto: use the resolved_table_width, clamped between GRIDMIN and GRIDMAX,
        //   but at least as big as min-inline-size and CAPMIN.
        // This diverges a little from the specification, but should be equivalent
        // (other than using the stretch-fit size instead of the containing block width).
        let used_width_of_table = match style
            .content_box_size(containing_block_for_table, &self.pbm)
            .inline
        {
            LengthPercentage(_) => resolved_table_width.max(grid_min_max.min_content),
            Auto => {
                let min_width: Au = style
                    .content_min_box_size(containing_block_for_table, &self.pbm)
                    .inline
                    .auto_is(Length::zero)
                    .into();
                resolved_table_width
                    .clamp(grid_min_max.min_content, grid_min_max.max_content)
                    .max(min_width)
            },
        };

        // Padding and border should apply to the table grid, but they are properties of the
        // parent element (the table wrapper). In order to account for this, we subtract the
        // border and padding inline size from the caption size.
        let caption_minimum_inline_size =
            caption_minimum_inline_size - self.pbm.padding_border_sums.inline;
        self.table_width = used_width_of_table.max(caption_minimum_inline_size);

        // > The assignable table width is the used width of the table minus the total horizontal
        // > border spacing (if any). This is the width that we will be able to allocate to the
        // > columns.
        self.assignable_width = self.table_width - self.table.total_border_spacing().inline;

        // This is the amount that we will use to resolve percentages in the padding of cells.
        // It matches what Gecko and Blink do, though they disagree when there is a big caption.
        self.basis_for_cell_padding_percentage =
            used_width_of_table - self.table.border_spacing().inline * 2;
    }

    /// Distribute width to columns, performing step 2.4 of table layout from
    /// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>.
    fn distribute_width_to_columns(&self) -> Vec<Au> {
        if self.table.slots.is_empty() {
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

        for column_idx in 0..self.table.size.width {
            let column_measure = &self.column_measures[column_idx];
            let min_content_width = column_measure.content_sizes.min_content;
            let max_content_width = column_measure.content_sizes.max_content;
            let constrained = self.columns[column_idx].constrained;

            let (
                min_content_percentage_sizing_guess,
                min_content_specified_sizing_guess,
                max_content_sizing_guess,
            ) = if !column_measure.percentage.is_zero() {
                let resolved = self.assignable_width.scale_by(column_measure.percentage.0);
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
        if self.assignable_width >= max_content_sizing_sum {
            self.distribute_extra_width_to_columns(
                &mut max_content_sizing_guesses,
                max_content_sizing_sum,
            );
            return max_content_sizing_guesses;
        }
        let min_content_specified_sizing_sum = sum(&min_content_specified_sizing_guesses);
        if self.assignable_width == min_content_specified_sizing_sum {
            return min_content_specified_sizing_guesses;
        }
        let min_content_percentage_sizing_sum = sum(&min_content_percentage_sizing_guesses);
        if self.assignable_width == min_content_percentage_sizing_sum {
            return min_content_percentage_sizing_guesses;
        }
        let min_content_sizes_sum = sum(&min_content_sizing_guesses);
        if self.assignable_width <= min_content_sizes_sum {
            return min_content_sizing_guesses;
        }

        let bounds = |sum_a, sum_b| self.assignable_width > sum_a && self.assignable_width < sum_b;
        let blend = |a: &[Au], sum_a: Au, b: &[Au], sum_b: Au| {
            // First convert the Au units to f32 in order to do floating point division.
            let weight_a =
                (self.assignable_width - sum_b).to_f32_px() / (sum_a - sum_b).to_f32_px();
            let weight_b = 1.0 - weight_a;
            a.iter()
                .zip(b.iter())
                .map(|(guess_a, guess_b)| {
                    (guess_a.scale_by(weight_a)) + (guess_b.scale_by(weight_b))
                })
                .collect()
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
    fn distribute_extra_width_to_columns(&self, column_sizes: &mut [Au], column_sizes_sum: Au) {
        let all_columns = 0..self.table.size.width;
        let extra_inline_size = self.assignable_width - column_sizes_sum;

        let has_originating_cells =
            |column_index: &usize| self.columns[*column_index].has_originating_cells;
        let is_constrained = |column_index: &usize| self.columns[*column_index].constrained;
        let is_unconstrained = |column_index: &usize| !is_constrained(column_index);
        let has_percent_greater_than_zero =
            |column_index: &usize| self.column_measures[*column_index].percentage.0 > 0.;
        let has_percent_zero = |column_index: &usize| !has_percent_greater_than_zero(column_index);
        let has_max_content = |column_index: &usize| {
            self.column_measures[*column_index]
                .content_sizes
                .max_content !=
                Au(0)
        };

        let max_content_sum =
            |column_index: usize| self.column_measures[column_index].content_sizes.max_content;

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
                    self.column_measures[column_index]
                        .content_sizes
                        .max_content
                        .to_f32_px() /
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
                    self.column_measures[column_index]
                        .content_sizes
                        .max_content
                        .to_f32_px() /
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
            .map(|column_index| self.column_measures[column_index].percentage.0)
            .sum::<f32>();
        if total_percent > 0. {
            for column_index in columns_with_percentage {
                column_sizes[column_index] += extra_inline_size
                    .scale_by(self.column_measures[column_index].percentage.0 / total_percent);
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
        let extra_space_for_all_columns =
            extra_inline_size.scale_by(1.0 / self.table.size.width as f32);
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
                    self.table.rows.get(row_index).map_or(false, |row| {
                        let row_group_collects_for_nearest_positioned_ancestor =
                            row.group_index.map_or(false, |group_index| {
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
                        let TableSlot::Cell(ref cell) = slot else {
                            return None;
                        };

                        let coordinates = TableSlotCoordinates::new(column_index, row_index);
                        let border: LogicalSides<Au> = self
                            .get_collapsed_borders_for_cell(cell, coordinates)
                            .unwrap_or_else(|| {
                                cell.style
                                    .border_width(containing_block_for_table.style.writing_mode)
                            })
                            .into();

                        let padding: LogicalSides<Au> = cell
                            .style
                            .padding(containing_block_for_table.style.writing_mode)
                            .percentages_relative_to(self.basis_for_cell_padding_percentage.into())
                            .into();
                        let inline_border_padding_sum = border.inline_sum() + padding.inline_sum();

                        let mut total_cell_width: Au = (column_index..column_index + cell.colspan)
                            .map(|column_index| self.distributed_column_widths[column_index])
                            .sum::<Au>() -
                            inline_border_padding_sum;
                        total_cell_width = total_cell_width.max(Au::zero());

                        let containing_block_for_children = ContainingBlock {
                            inline_size: total_cell_width,
                            block_size: AuOrAuto::Auto,
                            style: &cell.style,
                        };

                        let mut positioning_context = PositioningContext::new_for_subtree(
                            collect_for_nearest_positioned_ancestor,
                        );

                        let layout = cell.contents.layout(
                            layout_context,
                            &mut positioning_context,
                            &containing_block_for_children,
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

                let content_size_from_layout = ContentSizes {
                    min_content: layout.layout.content_block_size,
                    max_content: layout.layout.content_block_size,
                };
                self.cell_measures[row_index][column_index]
                    .block
                    .content_sizes
                    .max_assign(content_size_from_layout);
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

            let mut percentage = match self.table.rows.get(row_index) {
                Some(row) => {
                    get_size_percentage_contribution_from_style(&row.style, writing_mode)
                        .block
                        .0
                },
                None => 0.,
            };
            for column_index in 0..self.table.size.width {
                let cell_percentage = self.cell_measures[row_index][column_index]
                    .block
                    .percentage
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
        containing_block_for_table: &ContainingBlock,
    ) {
        // The table content height is the maximum of the computed table height from style and the
        // sum of computed row heights from row layout plus size from borders and spacing.
        // When block-size doesn't compute to auto, `containing_block_for children` will have
        // the resulting length, properly clamped between min-block-size and max-block-size.
        let style = &self.table.style;
        let table_height_from_style = match style
            .content_box_size(containing_block_for_table, &self.pbm)
            .block
        {
            LengthPercentage(_) => containing_block_for_children.block_size,
            Auto => style
                .content_min_box_size(containing_block_for_table, &self.pbm)
                .block
                .map(Au::from),
        }
        .auto_is(Au::zero);

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
        &mut self,
        caption: &TableCaption,
        table_pbm: &PaddingBorderMargin,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        parent_positioning_context: &mut PositioningContext,
    ) -> BoxFragment {
        let context = caption.context.borrow();
        let mut positioning_context = PositioningContext::new_for_style(&context.style);
        let containing_block = &ContainingBlock {
            inline_size: self.table_width + table_pbm.padding_border_sums.inline,
            block_size: AuOrAuto::Auto,
            style: containing_block.style,
        };

        let mut box_fragment = context.layout_in_flow_block_level(
            layout_context,
            positioning_context
                .as_mut()
                .unwrap_or(parent_positioning_context),
            containing_block,
            None, /* sequential_layout_state */
        );

        box_fragment.content_rect.start_corner.block += box_fragment
            .block_margins_collapsed_with_children
            .start
            .solve();

        if let Some(positioning_context) = positioning_context.take() {
            parent_positioning_context.append(positioning_context);
        }

        box_fragment
    }

    /// Lay out the table (grid and captions) of this [`TableLayout`] into fragments. This should
    /// only be be called after calling [`TableLayout.compute_measures`].
    fn layout(
        mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
    ) -> IndependentLayout {
        let writing_mode = containing_block_for_children.style.writing_mode;
        let grid_min_max = self.compute_grid_min_max(layout_context, writing_mode);
        let caption_minimum_inline_size =
            self.compute_caption_minimum_inline_size(layout_context, writing_mode);
        self.compute_table_width(
            containing_block_for_children,
            containing_block_for_table,
            grid_min_max,
            caption_minimum_inline_size,
        );

        // The table wrapper is the one that has the CSS properties for the grid's border and padding. This
        // weirdness is difficult to express in Servo's layout system. We have the wrapper size itself as if
        // those properties applied to it and then just account for the discrepency in sizing here. In reality,
        // the wrapper does not draw borders / backgrounds and all of its content (grid and captions) are
        // placed with a negative offset in the table wrapper's content box so that they overlap the undrawn
        // border / padding area.
        //
        // TODO: This is a pretty large hack. It would be nicer to actually have the grid sized properly,
        // but it works for now.
        let table_pbm = self
            .table
            .style
            .padding_border_margin(containing_block_for_table);
        let offset_from_wrapper = -table_pbm.padding - table_pbm.border;
        let mut current_block_offset = offset_from_wrapper.block_start;

        let mut table_layout = IndependentLayout {
            fragments: Vec::new(),
            content_block_size: Zero::zero(),
            content_inline_size_for_table: None,
            baselines: Baselines::default(),
        };

        table_layout
            .fragments
            .extend(self.table.captions.iter().filter_map(|caption| {
                if caption.context.borrow().style.clone_caption_side() != CaptionSide::Top {
                    return None;
                }

                let original_positioning_context_length = positioning_context.len();
                let mut caption_fragment = self.layout_caption(
                    caption,
                    &table_pbm,
                    layout_context,
                    containing_block_for_children,
                    positioning_context,
                );

                caption_fragment.content_rect.start_corner.inline +=
                    offset_from_wrapper.inline_start;
                caption_fragment.content_rect.start_corner.block += current_block_offset;
                current_block_offset += caption_fragment.margin_rect().size.block;

                let caption_fragment = Fragment::Box(caption_fragment);
                positioning_context.adjust_static_position_of_hoisted_fragments(
                    &caption_fragment,
                    original_positioning_context_length,
                );
                Some(caption_fragment)
            }));

        let original_positioning_context_length = positioning_context.len();
        let mut grid_fragment = self.layout_grid(
            layout_context,
            &table_pbm,
            positioning_context,
            containing_block_for_children,
            containing_block_for_table,
        );

        // Take the baseline of the grid fragment, after adjusting it to be in the coordinate system
        // of the table wrapper.
        table_layout.baselines = grid_fragment
            .baselines
            .offset(current_block_offset + grid_fragment.content_rect.start_corner.block);

        grid_fragment.content_rect.start_corner.inline += offset_from_wrapper.inline_start;
        grid_fragment.content_rect.start_corner.block += current_block_offset;
        current_block_offset += grid_fragment.border_rect().size.block;
        table_layout.content_inline_size_for_table = Some(grid_fragment.content_rect.size.inline);

        let grid_fragment = Fragment::Box(grid_fragment);
        positioning_context.adjust_static_position_of_hoisted_fragments(
            &grid_fragment,
            original_positioning_context_length,
        );
        table_layout.fragments.push(grid_fragment);

        table_layout
            .fragments
            .extend(self.table.captions.iter().filter_map(|caption| {
                if caption.context.borrow().style.clone_caption_side() != CaptionSide::Bottom {
                    return None;
                }

                let original_positioning_context_length = positioning_context.len();
                let mut caption_fragment = self.layout_caption(
                    caption,
                    &table_pbm,
                    layout_context,
                    containing_block_for_children,
                    positioning_context,
                );

                caption_fragment.content_rect.start_corner.inline +=
                    offset_from_wrapper.inline_start;
                caption_fragment.content_rect.start_corner.block += current_block_offset;
                current_block_offset += caption_fragment.margin_rect().size.block;

                let caption_fragment = Fragment::Box(caption_fragment);
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
        table_pbm: &PaddingBorderMargin,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
    ) -> BoxFragment {
        self.distributed_column_widths = self.distribute_width_to_columns();
        self.layout_cells_in_row(
            layout_context,
            containing_block_for_children,
            positioning_context,
        );
        let writing_mode = containing_block_for_children.style.writing_mode;
        let first_layout_row_heights = self.do_first_row_layout(writing_mode);
        self.compute_table_height_and_final_row_heights(
            first_layout_row_heights,
            containing_block_for_children,
            containing_block_for_table,
        );

        assert_eq!(self.table.size.height, self.row_sizes.len());
        assert_eq!(self.table.size.width, self.distributed_column_widths.len());

        if self.table.size.width == 0 && self.table.size.height == 0 {
            let content_rect = LogicalRect {
                start_corner: table_pbm.border_padding_start(),
                size: LogicalVec2 {
                    inline: self.table_width,
                    block: self.final_table_height,
                },
            };
            return BoxFragment::new(
                self.table.grid_base_fragment_info,
                self.table.grid_style.clone(),
                Vec::new(),
                content_rect,
                table_pbm.padding,
                table_pbm.border,
                LogicalSides::zero(),
                None, /* clearance */
                CollapsedBlockMargins::zero(),
            );
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
            let mut row_fragment_layout =
                RowFragmentLayout::new(table_row, row_index, &table_and_track_dimensions);

            let old_row_group_index = row_group_fragment_layout
                .as_ref()
                .map(|layout: &RowGroupFragmentLayout| layout.index);
            if table_row.group_index != old_row_group_index {
                // First create the Fragment for any existing RowGroupFragmentLayout.
                if let Some(old_row_group_layout) = row_group_fragment_layout.take() {
                    table_fragments.push(Fragment::Box(old_row_group_layout.finish(
                        layout_context,
                        positioning_context,
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

                // The PositioningContext for cells is, in order or preference, the PositioningContext of the row,
                // the PositioningContext of the row group, or the PositioningContext of the table.
                let row_group_positioning_context = row_group_fragment_layout
                    .as_mut()
                    .and_then(|layout| layout.positioning_context.as_mut());
                let positioning_context_for_cells = row_fragment_layout
                    .positioning_context
                    .as_mut()
                    .or(row_group_positioning_context)
                    .unwrap_or(positioning_context);

                self.do_final_cell_layout(
                    row_index,
                    column_index,
                    &table_and_track_dimensions,
                    &row_fragment_layout.rect,
                    positioning_context_for_cells,
                    &mut baselines,
                    &mut row_fragment_layout.fragments,
                );
            }

            let row_fragment = Fragment::Box(row_fragment_layout.finish(
                layout_context,
                positioning_context,
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
                containing_block_for_children,
            )));
        }

        let content_rect = LogicalRect {
            start_corner: table_pbm.border_padding_start(),
            size: LogicalVec2 {
                inline: table_and_track_dimensions.table_rect.max_inline_position(),
                block: table_and_track_dimensions.table_rect.max_block_position(),
            },
        };
        BoxFragment::new(
            self.table.grid_base_fragment_info,
            self.table.grid_style.clone(),
            table_fragments,
            content_rect,
            table_pbm.padding,
            table_pbm.border,
            LogicalSides::zero(),
            None, /* clearance */
            CollapsedBlockMargins::zero(),
        )
        .with_baselines(baselines)
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
        row_rect: &LogicalRect<Au>,
        positioning_context: &mut PositioningContext,
        baselines: &mut Baselines,
        cell_fragments: &mut Vec<Fragment>,
    ) {
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

        let row_block_offset = row_rect.start_corner.block;
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
        row_relative_cell_rect.start_corner -= row_rect.start_corner;
        let mut fragment = cell.create_fragment(
            layout,
            row_relative_cell_rect,
            row_baseline,
            positioning_context,
            &self.table.style,
        );
        let column = self.table.columns.get(column_index);
        let column_group = column
            .and_then(|column| column.group_index)
            .and_then(|index| self.table.column_groups.get(index));
        if let Some(column_group) = column_group {
            let mut rect = dimensions.get_column_group_rect(column_group);
            rect.start_corner -= row_rect.start_corner;
            fragment.add_extra_background(ExtraBackground {
                style: column_group.style.clone(),
                rect,
            })
        }
        if let Some(column) = column {
            if !column.is_anonymous {
                let mut rect = dimensions.get_column_rect(column_index);
                rect.start_corner -= row_rect.start_corner;
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
            let mut rect = dimensions.get_row_group_rect(row_group);
            rect.start_corner -= row_rect.start_corner;
            fragment.add_extra_background(ExtraBackground {
                style: row_group.style.clone(),
                rect,
            })
        }
        if let Some(row) = row {
            let mut rect = *row_rect;
            rect.start_corner = LogicalVec2::zero();
            fragment.add_extra_background(ExtraBackground {
                style: row.style.clone(),
                rect,
            })
        }
        cell_fragments.push(Fragment::Box(fragment));

        // If this cell has baseline alignment, it can adjust the table's overall baseline.
    }

    fn make_fragments_for_columns_and_column_groups(
        &mut self,
        dimensions: &TableAndTrackDimensions,
        fragments: &mut Vec<Fragment>,
    ) {
        for column_group in self.table.column_groups.iter() {
            if !column_group.is_empty() {
                fragments.push(Fragment::Positioning(PositioningFragment::new_empty(
                    column_group.base_fragment_info,
                    dimensions.get_column_group_rect(column_group),
                    column_group.style.clone(),
                )));
            }
        }

        for (column_index, column) in self.table.columns.iter().enumerate() {
            fragments.push(Fragment::Positioning(PositioningFragment::new_empty(
                column.base_fragment_info,
                dimensions.get_column_rect(column_index),
                column.style.clone(),
            )));
        }
    }

    fn compute_border_collapse(&mut self, writing_mode: WritingMode) {
        if self.table.style.get_inherited_table().border_collapse != BorderCollapse::Collapse {
            self.collapsed_borders = None;
            return;
        }

        let mut collapsed_borders = CollapsedBorders {
            block: vec![Au::zero(); self.table.size.height + 1],
            inline: vec![Au::zero(); self.table.size.width + 1],
        };

        for row_index in 0..self.table.size.height {
            for column_index in 0..self.table.size.width {
                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => continue,
                };

                let border = cell.style.border_width(writing_mode);
                collapsed_borders.block[row_index].max_assign(border.block_start.into());
                collapsed_borders.block[row_index + cell.rowspan]
                    .max_assign(border.block_end.into());
                collapsed_borders.inline[column_index].max_assign(border.inline_start.into());
                collapsed_borders.inline[column_index + cell.colspan]
                    .max_assign(border.inline_end.into());
            }
        }

        self.collapsed_borders = Some(collapsed_borders);
    }

    fn get_collapsed_borders_for_cell(
        &self,
        cell: &TableSlotCell,
        coordinates: TableSlotCoordinates,
    ) -> Option<LogicalSides<Length>> {
        let collapsed_borders = self.collapsed_borders.as_ref()?;
        let end_x = coordinates.x + cell.colspan;
        let end_y = coordinates.y + cell.rowspan;
        let mut result: LogicalSides<Length> = LogicalSides {
            inline_start: collapsed_borders.inline[coordinates.x],
            inline_end: collapsed_borders.inline[end_x],
            block_start: collapsed_borders.block[coordinates.y],
            block_end: collapsed_borders.block[end_y],
        }
        .into();

        if coordinates.x != 0 {
            result.inline_start = result.inline_start / 2.0;
        }
        if coordinates.y != 0 {
            result.block_start = result.block_start / 2.0;
        }
        if end_x != self.table.size.width {
            result.inline_end = result.inline_end / 2.0;
        }
        if end_y != self.table.size.height {
            result.block_end = result.block_end / 2.0;
        }

        Some(result)
    }
}

struct RowFragmentLayout<'a> {
    row: &'a TableTrack,
    rect: LogicalRect<Au>,
    positioning_context: Option<PositioningContext>,
    fragments: Vec<Fragment>,
}

impl<'a> RowFragmentLayout<'a> {
    fn new(table_row: &'a TableTrack, index: usize, dimensions: &TableAndTrackDimensions) -> Self {
        Self {
            row: table_row,
            rect: dimensions.get_row_rect(index),
            positioning_context: PositioningContext::new_for_style(&table_row.style),
            fragments: Vec::new(),
        }
    }
    fn finish(
        mut self,
        layout_context: &LayoutContext,
        table_positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        row_group_fragment_layout: &mut Option<RowGroupFragmentLayout>,
    ) -> BoxFragment {
        if self.positioning_context.is_some() {
            self.rect.start_corner += relative_adjustement(&self.row.style, containing_block);
        }

        if let Some(ref row_group_layout) = row_group_fragment_layout {
            self.rect.start_corner -= row_group_layout.rect.start_corner;
        }

        let mut row_fragment = BoxFragment::new(
            self.row.base_fragment_info,
            self.row.style.clone(),
            self.fragments,
            self.rect,
            LogicalSides::zero(), /* padding */
            LogicalSides::zero(), /* border */
            LogicalSides::zero(), /* margin */
            None,                 /* clearance */
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

        row_fragment
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
        containing_block: &ContainingBlock,
    ) -> BoxFragment {
        if self.positioning_context.is_some() {
            self.rect.start_corner += relative_adjustement(&self.style, containing_block);
        }

        let mut row_group_fragment = BoxFragment::new(
            self.base_fragment_info,
            self.style,
            self.fragments,
            self.rect,
            LogicalSides::zero(), /* padding */
            LogicalSides::zero(), /* border */
            LogicalSides::zero(), /* margin */
            None,                 /* clearance */
            CollapsedBlockMargins::zero(),
        );
        row_group_fragment.set_does_not_paint_background();

        if let Some(mut row_positioning_context) = self.positioning_context.take() {
            row_positioning_context
                .layout_collected_children(layout_context, &mut row_group_fragment);
            table_positioning_context.append(row_positioning_context);
        }

        row_group_fragment
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

    pub(crate) fn inline_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        let mut layout = TableLayout::new(self);
        let mut table_content_sizes = layout.compute_grid_min_max(layout_context, writing_mode);

        let mut caption_minimum_inline_size =
            layout.compute_caption_minimum_inline_size(layout_context, writing_mode);
        if caption_minimum_inline_size > table_content_sizes.min_content ||
            caption_minimum_inline_size > table_content_sizes.max_content
        {
            // Padding and border should apply to the table grid, but they will be taken into
            // account when computing the inline content sizes of the table wrapper (our parent), so
            // this code removes their contribution from the inline content size of the caption.
            let padding = self
                .style
                .padding(writing_mode)
                .percentages_relative_to(Length::zero());
            let border = self.style.border_width(writing_mode);
            caption_minimum_inline_size -= (padding.inline_sum() + border.inline_sum()).into();
            table_content_sizes
                .min_content
                .max_assign(caption_minimum_inline_size);
            table_content_sizes
                .max_content
                .max_assign(caption_minimum_inline_size);
        }

        table_content_sizes
    }

    fn get_column_measure_for_column_at_index(
        &self,
        writing_mode: WritingMode,
        column_index: usize,
    ) -> CellOrTrackMeasure {
        let column = match self.columns.get(column_index) {
            Some(column) => column,
            None => return CellOrTrackMeasure::zero(),
        };

        let (size, min_size, max_size) =
            get_outer_sizes_from_style(&column.style, writing_mode, &LogicalVec2::zero());
        let percentage_contribution =
            get_size_percentage_contribution_from_style(&column.style, writing_mode);

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
                max_content: min_size.inline.max(max_size.inline.min(size.inline)),
            },
            percentage: percentage_contribution.inline,
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
        let size = row
            .style
            .box_size(writing_mode)
            .block
            .non_auto()
            .and_then(|size| size.to_length())
            .map_or_else(Au::zero, Au::from);
        let percentage_contribution =
            get_size_percentage_contribution_from_style(&row.style, writing_mode);

        CellOrTrackMeasure {
            content_sizes: ContentSizes {
                min_content: size,
                max_content: size,
            },
            percentage: percentage_contribution.block,
        }
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block_for_table: &ContainingBlock,
    ) -> IndependentLayout {
        TableLayout::new(self).layout(
            layout_context,
            positioning_context,
            containing_block_for_children,
            containing_block_for_table,
        )
    }
}

impl TableSlotCell {
    fn effective_vertical_align(&self) -> VerticalAlignKeyword {
        match self.style.clone_vertical_align() {
            VerticalAlign::Keyword(VerticalAlignKeyword::Top) => VerticalAlignKeyword::Top,
            VerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => VerticalAlignKeyword::Bottom,
            VerticalAlign::Keyword(VerticalAlignKeyword::Middle) => VerticalAlignKeyword::Middle,
            _ => VerticalAlignKeyword::Baseline,
        }
    }

    fn create_fragment(
        &self,
        mut layout: CellLayout,
        cell_rect: LogicalRect<Au>,
        cell_baseline: Au,
        positioning_context: &mut PositioningContext,
        table_style: &ComputedValues,
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

        let mut base_fragment_info = self.base_fragment_info;
        if self.style.get_inherited_table().empty_cells == EmptyCells::Hide &&
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
            vertical_align_fragment_rect,
            layout.layout.fragments,
            self.style.writing_mode,
        );

        // Adjust the static position of all absolute children based on the
        // final content rect of this fragment. Note that we are not shifting by the position of the
        // Anonymous fragment we use to shift content to the baseline.
        //
        // TODO(mrobinson): This is correct for absolutes that are direct children of the table
        // cell, but wrong for absolute fragments that are more deeply nested in the hierarchy of
        // fragments.
        layout
            .positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(
                &cell_content_rect.start_corner,
                PositioningContextLength::zero(),
            );
        positioning_context.append(layout.positioning_context);

        BoxFragment::new(
            base_fragment_info,
            self.style.clone(),
            vec![Fragment::Positioning(vertical_align_fragment)],
            cell_content_rect,
            layout.padding,
            layout.border,
            LogicalSides::zero(), /* margin */
            None,                 /* clearance */
            CollapsedBlockMargins::zero(),
        )
        .with_baselines(layout.layout.baselines)
    }
}

fn get_size_percentage_contribution_from_style(
    style: &Arc<ComputedValues>,
    writing_mode: WritingMode,
) -> LogicalVec2<Percentage> {
    // From <https://drafts.csswg.org/css-tables/#percentage-contribution>
    // > The percentage contribution of a table cell, column, or column group is defined
    // > in terms of the computed values of width and max-width that have computed values
    // > that are percentages:
    // >    min(percentage width, percentage max-width).
    // > If the computed values are not percentages, then 0% is used for width, and an
    // > infinite percentage is used for max-width.
    let size = style.box_size(writing_mode);
    let max_size = style.max_box_size(writing_mode);

    let get_contribution_for_axis =
        |size: LengthPercentageOrAuto<'_>, max_size: Option<&ComputedLengthPercentage>| {
            let size_percentage = size
                .non_auto()
                .and_then(|length_percentage| length_percentage.to_percentage())
                .unwrap_or(Percentage(0.));
            let max_size_percentage = max_size
                .and_then(|length_percentage| length_percentage.to_percentage())
                .unwrap_or(Percentage(f32::INFINITY));
            Percentage(size_percentage.0.min(max_size_percentage.0))
        };

    LogicalVec2 {
        inline: get_contribution_for_axis(size.inline, max_size.inline),
        block: get_contribution_for_axis(size.block, max_size.block),
    }
}

fn get_outer_sizes_from_style(
    style: &Arc<ComputedValues>,
    writing_mode: WritingMode,
    padding_border_sums: &LogicalVec2<Au>,
) -> (LogicalVec2<Au>, LogicalVec2<Au>, LogicalVec2<Au>) {
    let box_sizing = style.get_position().box_sizing;
    let outer_size = |size: LogicalVec2<Au>| match box_sizing {
        BoxSizing::ContentBox => size + *padding_border_sums,
        BoxSizing::BorderBox => LogicalVec2 {
            inline: size.inline.max(padding_border_sums.inline),
            block: size.block.max(padding_border_sums.block),
        },
    };
    let get_size_for_axis = |size: &LengthPercentageOrAuto<'_>| {
        size.non_auto()
            .and_then(|size| size.to_length())
            .map_or_else(Au::zero, Au::from)
    };
    let get_max_size_for_axis = |size: &Option<&ComputedLengthPercentage>| {
        size.and_then(|length_percentage| length_percentage.to_length())
            .map_or(MAX_AU, Au::from)
    };

    (
        outer_size(style.box_size(writing_mode).map(get_size_for_axis)),
        outer_size(style.min_box_size(writing_mode).map(get_size_for_axis)),
        outer_size(style.max_box_size(writing_mode).map(get_max_size_for_axis)),
    )
}

struct RowspanToDistribute<'a> {
    coordinates: TableSlotCoordinates,
    cell: &'a TableSlotCell,
    measure: &'a CellOrTrackMeasure,
}

impl<'a> RowspanToDistribute<'a> {
    fn range(&self) -> Range<usize> {
        self.coordinates.y..self.coordinates.y + self.cell.rowspan
    }

    fn fully_encloses(&self, other: &RowspanToDistribute) -> bool {
        other.coordinates.y > self.coordinates.y && other.range().end < self.range().end
    }
}
