/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Add;

use app_units::{Au, MAX_AU};
use euclid::num::Zero;
use log::warn;
use style::computed_values::border_collapse::T as BorderCollapse;
use style::logical_geometry::WritingMode;
use style::values::computed::{CSSPixelLength, Length, Percentage};
use style::values::generics::box_::{GenericVerticalAlign as VerticalAlign, VerticalAlignKeyword};
use style::values::generics::length::GenericLengthPercentageOrAuto::{Auto, LengthPercentage};

use super::{Table, TableSlot, TableSlotCell};
use crate::context::LayoutContext;
use crate::formatting_contexts::{Baselines, IndependentLayout};
use crate::fragment_tree::{AnonymousFragment, BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::{AuOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{PositioningContext, PositioningContextLength};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt, PaddingBorderMargin};
use crate::table::TableSlotCoordinates;
use crate::ContainingBlock;

/// A result of a final or speculative layout of a single cell in
/// the table. Note that this is only done for slots that are not
/// covered by spans or empty.
struct CellLayout {
    layout: IndependentLayout,
    padding: LogicalSides<Length>,
    border: LogicalSides<Length>,
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
        self.layout.content_block_size + (self.border.block_sum() + self.padding.block_sum()).into()
    }
}

/// A helper struct that performs the layout of the box tree version
/// of a table into the fragment tree version. This implements
/// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>
struct TableLayout<'a> {
    table: &'a Table,
    pbm: PaddingBorderMargin,
    column_constrainedness: Vec<bool>,
    column_has_originating_cell: Vec<bool>,
    cell_measures: Vec<Vec<CellOrColumnMeasure>>,
    assignable_width: Au,
    column_measures: Vec<CellOrColumnMeasure>,
    distributed_column_widths: Vec<Au>,
    row_sizes: Vec<Au>,
    row_baselines: Vec<Au>,
    cells_laid_out: Vec<Vec<Option<CellLayout>>>,
}

#[derive(Clone, Debug)]
struct CellOrColumnMeasure {
    content_sizes: ContentSizes,
    percentage_width: Percentage,
}

impl CellOrColumnMeasure {
    fn zero() -> Self {
        Self {
            content_sizes: ContentSizes::zero(),
            percentage_width: Percentage(0.),
        }
    }
}

impl<'a> TableLayout<'a> {
    fn new(table: &'a Table) -> TableLayout {
        Self {
            table,
            pbm: PaddingBorderMargin::zero(),
            column_constrainedness: Vec::new(),
            column_has_originating_cell: Vec::new(),
            cell_measures: Vec::new(),
            assignable_width: Au::zero(),
            column_measures: Vec::new(),
            distributed_column_widths: Vec::new(),
            row_sizes: Vec::new(),
            row_baselines: Vec::new(),
            cells_laid_out: Vec::new(),
        }
    }

    /// Do the preparatory steps to table layout, measuring cells and distributing sizes
    /// to all columns and rows.
    fn compute_measures(
        &mut self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) {
        let writing_mode = containing_block.style.writing_mode;
        self.compute_column_constrainedness_and_has_originating_cells(writing_mode);
        self.compute_cell_measures(layout_context, containing_block);
        self.compute_column_measures();
        self.compute_table_width(containing_block);
        self.distributed_column_widths = self.distribute_width_to_columns();
        self.do_row_layout_first_pass(layout_context, containing_block, positioning_context);
        self.distribute_height_to_rows();
    }

    /// This is an implementation of *Computing Cell Measures* from
    /// <https://drafts.csswg.org/css-tables/#computing-cell-measures>.
    pub(crate) fn compute_cell_measures(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
    ) {
        let writing_mode = containing_block.style.writing_mode;
        for row_index in 0..self.table.size.height {
            let mut row_measures = vec![CellOrColumnMeasure::zero(); self.table.size.width];

            for column_index in 0..self.table.size.width {
                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => continue,
                };

                // TODO: Should `box_size` percentages be treated as zero here or resolved against
                // the containing block?
                let pbm = cell.style.padding_border_margin(containing_block);
                let min_inline_size: Au = cell
                    .style
                    .min_box_size(writing_mode)
                    .inline
                    .percentage_relative_to(Length::zero())
                    .map(|value| value.into())
                    .auto_is(Au::zero);
                let max_inline_size: Au = cell.style.max_box_size(writing_mode).inline.map_or_else(
                    || MAX_AU,
                    |length_percentage| length_percentage.resolve(Length::zero()).into(),
                );
                let inline_size: Au = cell
                    .style
                    .box_size(writing_mode)
                    .inline
                    .percentage_relative_to(Length::zero())
                    .map(|value| value.into())
                    .auto_is(Au::zero);

                let content_sizes = cell
                    .contents
                    .contents
                    .inline_content_sizes(layout_context, writing_mode);

                // > The outer min-content width of a table-cell is max(min-width, min-content width)
                // > adjusted by the cell intrinsic offsets.
                let mut outer_min_content_width = content_sizes.min_content.max(min_inline_size);
                let mut outer_max_content_width = if !self.column_constrainedness[column_index] {
                    // > The outer max-content width of a table-cell in a non-constrained column is
                    // > max(min-width, width, min-content width, min(max-width, max-content width))
                    // > adjusted by the cell intrinsic offsets.
                    min_inline_size
                        .max(inline_size)
                        .max(content_sizes.min_content)
                        .max(max_inline_size.min(content_sizes.max_content))
                } else {
                    // > The outer max-content width of a table-cell in a constrained column is
                    // > max(min-width, width, min-content width, min(max-width, width)) adjusted by the
                    // > cell intrinsic offsets.
                    min_inline_size
                        .max(inline_size)
                        .max(content_sizes.min_content)
                        .max(max_inline_size.min(inline_size))
                };

                // > The percentage contribution of a table cell, column, or column group is defined
                // > in terms of the computed values of width and max-width that have computed values
                // > that are percentages:
                // >    min(percentage width, percentage max-width).
                // > If the computed values are not percentages, then 0% is used for width, and an
                // > infinite percentage is used for max-width.
                let inline_size_percent = cell
                    .style
                    .box_size(writing_mode)
                    .inline
                    .non_auto()
                    .and_then(|length_percentage| length_percentage.to_percentage())
                    .unwrap_or(Percentage(0.));
                let max_inline_size_percent = cell
                    .style
                    .max_box_size(writing_mode)
                    .inline
                    .and_then(|length_percentage| length_percentage.to_percentage())
                    .unwrap_or(Percentage(f32::INFINITY));
                let percentage_contribution =
                    Percentage(inline_size_percent.0.min(max_inline_size_percent.0));

                outer_min_content_width += pbm.padding_border_sums.inline;
                outer_max_content_width += pbm.padding_border_sums.inline;
                row_measures[column_index] = CellOrColumnMeasure {
                    content_sizes: ContentSizes {
                        min_content: outer_min_content_width,
                        max_content: outer_max_content_width,
                    },
                    percentage_width: percentage_contribution,
                };
            }

            self.cell_measures.push(row_measures);
        }
    }

    /// Compute the constrainedness of every column in the table.
    ///
    /// > A column is constrained if its corresponding table-column-group (if any), its
    /// > corresponding table-column (if any), or any of the cells spanning only that
    /// > column has a computed width that is not "auto", and is not a percentage.
    fn compute_column_constrainedness_and_has_originating_cells(
        &mut self,
        writing_mode: WritingMode,
    ) {
        for column_index in 0..self.table.size.width {
            let mut column_constrained = false;
            let mut column_has_originating_cell = false;

            for row_index in 0..self.table.size.height {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                let cell_constrained = match self.table.resolve_first_cell(coords) {
                    Some(cell) if cell.colspan == 1 => cell
                        .style
                        .box_size(writing_mode)
                        .inline
                        .non_auto()
                        .map(|length_percentage| length_percentage.to_length().is_some())
                        .unwrap_or(false),
                    _ => false,
                };
                column_has_originating_cell = column_has_originating_cell ||
                    matches!(self.table.get_slot(coords), Some(TableSlot::Cell(_)));
                column_constrained = column_constrained || cell_constrained;
            }
            self.column_constrainedness.push(column_constrained);
            self.column_has_originating_cell
                .push(column_has_originating_cell);
        }
    }

    /// This is an implementation of *Computing Column Measures* from
    /// <https://drafts.csswg.org/css-tables/#computing-column-measures>.
    fn compute_column_measures(&mut self) {
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
            let mut column_measure = CellOrColumnMeasure::zero();

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
                let cell_measure = &self.cell_measures[row_index][column_index];
                column_measure
                    .content_sizes
                    .max_assign(cell_measure.content_sizes);
                column_measure.percentage_width = Percentage(
                    column_measure
                        .percentage_width
                        .0
                        .max(cell_measure.percentage_width.0),
                );
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
        for column_index in 0..self.table.size.width {
            let column_measure = &mut column_measures[column_index];
            let final_intrinsic_percentage_width = column_measure
                .percentage_width
                .0
                .min(100. - total_intrinsic_percentage_width);
            total_intrinsic_percentage_width += final_intrinsic_percentage_width;
            column_measure.percentage_width = Percentage(final_intrinsic_percentage_width);
        }

        self.column_measures = column_measures;
    }

    fn compute_content_sizes_for_columns_with_span_up_to_n(
        &self,
        n: usize,
        old_column_measures: &[CellOrColumnMeasure],
    ) -> (usize, Vec<CellOrColumnMeasure>) {
        let mut next_span_n = usize::MAX;
        let mut new_content_sizes_for_columns = Vec::new();
        let border_spacing = self.table.border_spacing();

        for column_index in 0..self.table.size.width {
            let old_column_measure = &old_column_measures[column_index];
            let mut new_column_content_sizes = ContentSizes::zero();
            let mut new_column_intrinsic_percentage_width = Percentage(0.);

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

                let cell_measures = &self.cell_measures[resolved_coords.y][resolved_coords.x];
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
                if old_column_measure.percentage_width.0 <= 0. &&
                    cell_measures.percentage_width.0 != 0.
                {
                    // > 1. Start with the percentage contribution of the cell.
                    // > 2. Subtract the intrinsic percentage width of the column based on cells
                    // >    of span up to N-1 of all columns that the cell spans. If this gives a
                    // >    negative result, change it to 0%.
                    let mut spanned_columns_with_zero = 0;
                    let other_column_percentages_sum =
                        (columns_spanned).fold(0., |sum, spanned_column_index| {
                            let spanned_column_percentage =
                                old_column_measures[spanned_column_index].percentage_width;
                            if spanned_column_percentage.0 == 0. {
                                spanned_columns_with_zero += 1;
                            }
                            sum + spanned_column_percentage.0
                        });
                    let step_2 = (cell_measures.percentage_width -
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
            new_content_sizes_for_columns.push(CellOrColumnMeasure {
                content_sizes: new_column_content_sizes,
                percentage_width: new_column_intrinsic_percentage_width,
            });
        }
        (next_span_n, new_content_sizes_for_columns)
    }

    fn compute_table_width(&mut self, containing_block: &ContainingBlock) {
        // https://drafts.csswg.org/css-tables/#gridmin:
        // > The row/column-grid width minimum (GRIDMIN) width is the sum of the min-content width of
        // > all the columns plus cell spacing or borders.
        // https://drafts.csswg.org/css-tables/#gridmax:
        // > The row/column-grid width maximum (GRIDMAX) width is the sum of the max-content width of
        // > all the columns plus cell spacing or borders.

        let mut grid_min_and_max = self
            .column_measures
            .iter()
            .fold(ContentSizes::zero(), |result, measure| {
                result + measure.content_sizes
            });
        let border_spacing = self.table.border_spacing();
        let inline_border_spacing = border_spacing.inline * (self.table.size.width as i32 + 1);
        grid_min_and_max.min_content += inline_border_spacing;
        grid_min_and_max.max_content += inline_border_spacing;

        self.pbm = self.table.style.padding_border_margin(containing_block);
        let content_box_size = self
            .table
            .style
            .content_box_size(containing_block, &self.pbm);
        let min_content_sizes = self
            .table
            .style
            .content_min_box_size(containing_block, &self.pbm)
            .auto_is(Length::zero);

        // https://drafts.csswg.org/css-tables/#used-min-width-of-table
        // > The used min-width of a table is the greater of the resolved min-width, CAPMIN, and GRIDMIN.
        let used_min_width_of_table = grid_min_and_max
            .min_content
            .max(min_content_sizes.inline.into());

        // https://drafts.csswg.org/css-tables/#used-width-of-table
        // > The used width of a table depends on the columns and captions widths as follows:
        // > * If the table-root’s width property has a computed value (resolving to
        // >   resolved-table-width) other than auto, the used width is the greater of
        // >   resolved-table-width, and the used min-width of the table.
        // > * If the table-root has 'width: auto', the used width is the greater of min(GRIDMAX,
        // >   the table’s containing block width), the used min-width of the table.
        let used_width_of_table = match content_box_size.inline {
            LengthPercentage(length_percentage) => {
                Au::from(length_percentage).max(used_min_width_of_table)
            },
            Auto => grid_min_and_max
                .max_content
                .min(containing_block.inline_size)
                .max(used_min_width_of_table),
        };

        // > The assignable table width is the used width of the table minus the total horizontal
        // > border spacing (if any). This is the width that we will be able to allocate to the
        // > columns.
        self.assignable_width = used_width_of_table - inline_border_spacing;
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
            use style::Zero;

            let column_measure = &self.column_measures[column_idx];
            let min_content_width = column_measure.content_sizes.min_content;
            let max_content_width = column_measure.content_sizes.max_content;
            let constrained = self.column_constrainedness[column_idx];

            let (
                min_content_percentage_sizing_guess,
                min_content_specified_sizing_guess,
                max_content_sizing_guess,
            ) = if !column_measure.percentage_width.is_zero() {
                let resolved = self
                    .assignable_width
                    .scale_by(column_measure.percentage_width.0);
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
    fn distribute_extra_width_to_columns(&self, column_sizes: &mut Vec<Au>, column_sizes_sum: Au) {
        let all_columns = 0..self.table.size.width;
        let extra_inline_size = self.assignable_width - column_sizes_sum;

        let has_originating_cells =
            |column_index: &usize| self.column_has_originating_cell[*column_index];
        let is_constrained = |column_index: &usize| self.column_constrainedness[*column_index];
        let is_unconstrained = |column_index: &usize| !is_constrained(column_index);
        let has_percent_greater_than_zero =
            |column_index: &usize| self.column_measures[*column_index].percentage_width.0 > 0.;
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
            .map(|column_index| self.column_measures[column_index].percentage_width.0)
            .sum::<f32>();
        if total_percent > 0. {
            for column_index in columns_with_percentage {
                column_sizes[column_index] += extra_inline_size.scale_by(
                    self.column_measures[column_index].percentage_width.0 / total_percent,
                );
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
    fn do_row_layout_first_pass(
        &mut self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        parent_positioning_context: &mut PositioningContext,
    ) {
        for row_index in 0..self.table.slots.len() {
            let row = &self.table.slots[row_index];
            let mut cells_laid_out_row = Vec::new();
            for column_index in 0..row.len() {
                let cell = match &row[column_index] {
                    TableSlot::Cell(cell) => cell,
                    _ => {
                        cells_laid_out_row.push(None);
                        continue;
                    },
                };

                let mut total_width = Au::zero();
                for width_index in column_index..column_index + cell.colspan {
                    total_width += self.distributed_column_widths[width_index];
                }

                let border = cell.style.border_width(containing_block.style.writing_mode);
                let padding = cell
                    .style
                    .padding(containing_block.style.writing_mode)
                    .percentages_relative_to(Length::zero());
                let inline_border_padding_sum = border.inline_sum() + padding.inline_sum();
                let mut total_width: CSSPixelLength =
                    Length::from(total_width) - inline_border_padding_sum;
                total_width = total_width.max(Length::zero());

                let containing_block_for_children = ContainingBlock {
                    inline_size: total_width.into(),
                    block_size: AuOrAuto::Auto,
                    style: &cell.style,
                };
                let collect_for_nearest_positioned_ancestor =
                    parent_positioning_context.collects_for_nearest_positioned_ancestor();
                let mut positioning_context =
                    PositioningContext::new_for_subtree(collect_for_nearest_positioned_ancestor);

                let layout = cell.contents.layout(
                    layout_context,
                    &mut positioning_context,
                    &containing_block_for_children,
                );
                cells_laid_out_row.push(Some(CellLayout {
                    layout,
                    padding,
                    border,
                    positioning_context,
                }))
            }
            self.cells_laid_out.push(cells_laid_out_row);
        }
    }

    fn distribute_height_to_rows(&mut self) {
        for row_index in 0..self.table.size.height {
            let (mut max_ascent, mut max_descent, mut max_row_height) =
                (Au::zero(), Au::zero(), Au::zero());

            for column_index in 0..self.table.size.width {
                let coords = TableSlotCoordinates::new(column_index, row_index);

                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    TableSlot::Spanned(ref spanned_cells) if spanned_cells[0].y != 0 => {
                        let offset = spanned_cells[0];
                        let origin = coords - offset;

                        // We only allocate the remaining space for the last row of the rowspanned cell.
                        if let Some(TableSlot::Cell(origin_cell)) = self.table.get_slot(origin) {
                            if origin_cell.rowspan != offset.y + 1 {
                                continue;
                            }
                        }

                        // This is all of the rows that are spanned except this one.
                        let used_block_size = (origin.y..coords.y)
                            .map(|row_index| self.row_sizes[row_index])
                            .fold(Au::zero(), |sum, size| sum + size);
                        if let Some(layout) = &self.cells_laid_out[origin.y][origin.x] {
                            max_row_height =
                                max_row_height.max(layout.outer_block_size() - used_block_size);
                        }
                        continue;
                    },
                    _ => continue,
                };

                let layout = match self.cells_laid_out[row_index][column_index] {
                    Some(ref layout) => layout,
                    None => {
                        warn!("Did not find a layout at a slot index with an originating cell.");
                        continue;
                    },
                };

                let outer_block_size = layout.outer_block_size();
                if cell.rowspan == 1 {
                    max_row_height = max_row_height.max(outer_block_size);
                }

                if cell.effective_vertical_align() == VerticalAlignKeyword::Baseline {
                    let ascent = layout.ascent();
                    let border_padding_start =
                        layout.border.block_start + layout.padding.block_start;
                    let border_padding_end = layout.border.block_end + layout.padding.block_end;
                    max_ascent = max_ascent.max(ascent + border_padding_start.into());

                    // Only take into account the descent of this cell if doesn't span
                    // rows. The descent portion of the cell in cells that do span rows
                    // will be allocated to the other rows that it spans.
                    if cell.rowspan == 1 {
                        max_descent = max_descent.max(
                            layout.layout.content_block_size - ascent + border_padding_end.into(),
                        );
                    }
                }
            }

            self.row_baselines.push(max_ascent);
            self.row_sizes
                .push(max_row_height.max(max_ascent + max_descent));
        }
    }

    /// Lay out the table of this [`TableLayout`] into fragments. This should only be be called
    /// after calling [`TableLayout.compute_measures`].
    fn layout(mut self, positioning_context: &mut PositioningContext) -> IndependentLayout {
        assert_eq!(self.table.size.height, self.row_sizes.len());
        assert_eq!(self.table.size.width, self.distributed_column_widths.len());

        let mut baselines = Baselines::default();
        let border_spacing = self.table.border_spacing();
        let mut fragments = Vec::new();
        let mut row_offset = border_spacing.block;
        for row_index in 0..self.table.size.height {
            let mut column_offset = border_spacing.inline;
            let row_size = self.row_sizes[row_index];
            let row_baseline = self.row_baselines[row_index];

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
                baselines.first = Some(row_offset + row_size);
                baselines.last = Some(row_offset + row_size);
            }

            for column_index in 0..self.table.size.width {
                let layout = match self.cells_laid_out[row_index][column_index].take() {
                    Some(layout) => layout,
                    None => {
                        continue;
                    },
                };

                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => {
                        warn!("Did not find a non-spanned cell at index with layout.");
                        continue;
                    },
                };

                // If this cell has baseline alignment, it can adjust the table's overall baseline.
                if cell.effective_vertical_align() == VerticalAlignKeyword::Baseline {
                    if row_index == 0 {
                        baselines.first = Some(row_offset + row_baseline);
                    }
                    baselines.last = Some(row_offset + row_baseline);
                }

                // Calculate the inline and block size of all rows and columns that this cell spans.
                let inline_size: Au = (column_index..column_index + cell.colspan)
                    .map(|index| self.distributed_column_widths[index])
                    .fold(Au::zero(), Au::add) +
                    ((cell.colspan - 1) as i32 * border_spacing.inline);
                let block_size: Au = (row_index..row_index + cell.rowspan)
                    .map(|index| self.row_sizes[index])
                    .fold(Au::zero(), Au::add) +
                    ((cell.rowspan - 1) as i32 * border_spacing.block);

                let cell_rect: LogicalRect<Length> = LogicalRect {
                    start_corner: LogicalVec2 {
                        inline: column_offset.into(),
                        block: row_offset.into(),
                    },
                    size: LogicalVec2 {
                        inline: inline_size.into(),
                        block: block_size.into(),
                    },
                };

                fragments.push(Fragment::Box(cell.create_fragment(
                    layout,
                    cell_rect,
                    row_baseline,
                    positioning_context,
                )));

                column_offset += inline_size + border_spacing.inline;
            }

            row_offset += row_size + border_spacing.block;
        }

        if self.table.anonymous {
            baselines.first = None;
            baselines.last = None;
        }

        IndependentLayout {
            fragments,
            content_block_size: row_offset,
            baselines,
        }
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

    fn inline_content_sizes_for_cell_at(
        &self,
        coords: TableSlotCoordinates,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        let cell = match self.resolve_first_cell(coords) {
            Some(cell) => cell,
            None => return ContentSizes::zero(),
        };

        let sizes = cell.inline_content_sizes(layout_context, writing_mode);
        sizes.map(|size| size.scale_by(1.0 / cell.colspan as f32))
    }

    pub(crate) fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> (ContentSizes, Vec<Vec<ContentSizes>>) {
        let mut total_size = ContentSizes::zero();
        let mut inline_content_sizes = Vec::new();
        for column_index in 0..self.size.width {
            let mut row_inline_content_sizes = Vec::new();
            let mut max_content_sizes_in_column = ContentSizes::zero();

            for row_index in 0..self.size.width {
                // TODO: Take into account padding and border here.
                let coords = TableSlotCoordinates::new(column_index, row_index);

                let content_sizes =
                    self.inline_content_sizes_for_cell_at(coords, layout_context, writing_mode);
                max_content_sizes_in_column.max_assign(content_sizes);
                row_inline_content_sizes.push(content_sizes);
            }

            inline_content_sizes.push(row_inline_content_sizes);
            total_size += max_content_sizes_in_column;
        }
        let gutters = self.border_spacing().inline * (self.size.width as i32 + 1);
        total_size.min_content += gutters;
        total_size.max_content += gutters;
        (total_size, inline_content_sizes)
    }

    pub(crate) fn inline_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        self.compute_inline_content_sizes(layout_context, writing_mode)
            .0
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        let mut table_layout = TableLayout::new(self);
        table_layout.compute_measures(layout_context, positioning_context, containing_block);
        table_layout.layout(positioning_context)
    }
}

impl TableSlotCell {
    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        let border = self.style.border_width(writing_mode);
        let padding = self.style.padding(writing_mode);

        // For padding, a cyclic percentage is resolved against zero for determining intrinsic size
        // contributions.
        // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
        let zero = Length::zero();
        let border_padding_sum = border.inline_sum() +
            padding.inline_start.resolve(zero) +
            padding.inline_end.resolve(zero);

        let mut sizes = self
            .contents
            .contents
            .inline_content_sizes(layout_context, writing_mode);
        sizes.min_content += border_padding_sum.into();
        sizes.max_content += border_padding_sum.into();
        sizes
    }

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
        cell_rect: LogicalRect<Length>,
        cell_baseline: Au,
        positioning_context: &mut PositioningContext,
    ) -> BoxFragment {
        // This must be scoped to this function because it conflicts with euclid's Zero.
        use style::Zero as StyleZero;

        let cell_content_rect = cell_rect.deflate(&(&layout.padding + &layout.border));
        let content_block_size = layout.layout.content_block_size.into();
        let vertical_align_offset = match self.effective_vertical_align() {
            VerticalAlignKeyword::Top => Length::new(0.),
            VerticalAlignKeyword::Bottom => cell_content_rect.size.block - content_block_size,
            VerticalAlignKeyword::Middle => {
                (cell_content_rect.size.block - content_block_size).scale_by(0.5)
            },
            _ => {
                Length::from(cell_baseline) -
                    (layout.padding.block_start + layout.border.block_start) -
                    Length::from(layout.ascent())
            },
        };

        // Create an `AnonymousFragment` to move the cell contents to the cell baseline.
        let mut vertical_align_fragment_rect = cell_content_rect.clone();
        vertical_align_fragment_rect.start_corner = LogicalVec2 {
            inline: Length::new(0.),
            block: vertical_align_offset,
        };
        let vertical_align_fragment = AnonymousFragment::new(
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
            self.base_fragment_info,
            self.style.clone(),
            vec![Fragment::Anonymous(vertical_align_fragment)],
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
