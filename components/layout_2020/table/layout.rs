/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::num::Zero;
use log::warn;
use style::logical_geometry::WritingMode;
use style::values::computed::{CSSPixelLength, Length, LengthOrAuto};
use style::values::generics::length::GenericLengthPercentageOrAuto::{Auto, LengthPercentage};

use super::{Table, TableSlot, TableSlotCell};
use crate::context::LayoutContext;
use crate::formatting_contexts::IndependentLayout;
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::{LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{PositioningContext, PositioningContextLength};
use crate::sizing::ContentSizes;
use crate::style_ext::{ComputedValuesExt, PaddingBorderMargin};
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
    rowspan: usize,
}

/// A helper struct that performs the layout of the box tree version
/// of a table into the fragment tree version. This implements
/// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>
struct TableLayout<'a> {
    table: &'a Table,
    pbm: PaddingBorderMargin,
    assignable_width: Au,
    column_sizes: Vec<Au>,
    row_sizes: Vec<Au>,
    cells_laid_out: Vec<Vec<Option<CellLayout>>>,
}

impl<'a> TableLayout<'a> {
    fn new(table: &'a Table) -> TableLayout {
        Self {
            table,
            pbm: PaddingBorderMargin::zero(),
            assignable_width: Au::zero(),
            column_sizes: Vec::new(),
            row_sizes: Vec::new(),
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
        let (inline_content_sizes, column_content_sizes) = self
            .table
            .compute_inline_content_sizes(layout_context, containing_block.style.writing_mode);

        self.calculate_assignable_table_width(containing_block, inline_content_sizes);
        self.column_sizes =
            self.distribute_width_to_columns(column_content_sizes, containing_block);
        self.do_row_layout_first_pass(layout_context, containing_block, positioning_context);
        self.distribute_height_to_rows();
    }

    fn calculate_assignable_table_width(
        &mut self,
        containing_block: &ContainingBlock,
        inline_content_sizes: ContentSizes,
    ) {
        let grid_min_inline_size = inline_content_sizes.min_content;
        let grid_max_inline_size = inline_content_sizes.max_content;

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
        let used_min_width_of_table = grid_min_inline_size.max(min_content_sizes.inline.into());

        // https://drafts.csswg.org/css-tables/#used-width-of-table
        // > The used width of a table depends on the columns and captions widths as follows:
        // > * If the table-root’s width property has a computed value (resolving to
        // >   resolved-table-width) other than auto, the used width is the greater of
        // >   resolved-table-width, and the used min-width of the table.
        // > * If the table-root has 'width: auto', the used width is the greater of min(GRIDMAX,
        // >   the table’s containing block width), the used min-width of the table.
        let used_width_of_table = match content_box_size.inline {
            LengthPercentage(length_percentage) => {
                length_percentage.max(used_min_width_of_table.into())
            },
            Auto => grid_max_inline_size
                .min(containing_block.inline_size.into())
                .max(used_min_width_of_table)
                .into(),
        };

        self.assignable_width = used_width_of_table.into();
    }

    /// Distribute width to columns, performing step 2.4 of table layout from
    /// <https://drafts.csswg.org/css-tables/#table-layout-algorithm>.
    fn distribute_width_to_columns(
        &self,
        column_content_sizes: Vec<ContentSizes>,
        containing_block: &ContainingBlock,
    ) -> Vec<Au> {
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
            let coords = TableSlotCoordinates::new(column_idx, 0);
            let cell = match self.table.resolve_first_cell(coords) {
                Some(cell) => cell,
                None => {
                    min_content_sizing_guesses.push(Au::zero());
                    min_content_percentage_sizing_guesses.push(Au::zero());
                    min_content_specified_sizing_guesses.push(Au::zero());
                    max_content_sizing_guesses.push(Au::zero());
                    continue;
                },
            };

            let inline_size = cell
                .style
                .box_size(containing_block.style.writing_mode)
                .inline;
            let min_and_max_content = &column_content_sizes[column_idx];
            let min_content_width = min_and_max_content.min_content;
            let max_content_width = min_and_max_content.max_content;

            let (
                min_content_percentage_sizing_guess,
                min_content_specified_sizing_guess,
                max_content_sizing_guess,
            ) = match inline_size {
                LengthPercentage(length_percentage) if length_percentage.has_percentage() => {
                    let percent_guess = min_content_width.max(
                        length_percentage
                            .resolve(self.assignable_width.into())
                            .into(),
                    );
                    (percent_guess, percent_guess, percent_guess)
                },
                LengthPercentage(_) => (min_content_width, max_content_width, max_content_width),
                Auto => (min_content_width, min_content_width, max_content_width),
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

    fn distribute_extra_width_to_columns(
        &self,
        max_content_sizing_guesses: &mut Vec<Au>,
        max_content_sum: Au,
    ) {
        // The simplest distribution algorithm, until we have support for proper extra space
        // distribution is to equally distribute the extra space.
        let ratio_factor = 1.0 / max_content_sizing_guesses.len() as f32;
        let extra_space_for_all_columns =
            (self.assignable_width - max_content_sum).scale_by(ratio_factor);
        for guess in max_content_sizing_guesses.iter_mut() {
            *guess += extra_space_for_all_columns;
        }
    }

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
                    total_width += self.column_sizes[width_index];
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
                    inline_size: total_width,
                    block_size: LengthOrAuto::Auto,
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
                    rowspan: cell.rowspan,
                }))
            }
            self.cells_laid_out.push(cells_laid_out_row);
        }
    }

    fn distribute_height_to_rows(&mut self) {
        for row_index in 0..self.table.size.height {
            let mut max_row_height = Au::zero();
            for column_index in 0..self.table.size.width {
                let coords = TableSlotCoordinates::new(column_index, row_index);
                self.table
                    .resolve_first_cell_coords(coords)
                    .map(|resolved_coords| {
                        let cell = self.cells_laid_out[resolved_coords.y][resolved_coords.x]
                            .as_ref()
                            .unwrap();
                        let total_height = cell.layout.content_block_size +
                            cell.border.block_sum().into() +
                            cell.padding.block_sum().into();
                        // TODO: We are accounting for rowspan=0 here, but perhaps this should be
                        // translated into a real rowspan during table box tree construction.
                        let effective_rowspan = match cell.rowspan {
                            0 => (self.table.size.height - resolved_coords.y) as i32,
                            rowspan => rowspan as i32,
                        };
                        max_row_height = (total_height / effective_rowspan).max(max_row_height)
                    });
            }
            self.row_sizes.push(max_row_height);
        }
    }

    /// Lay out the table of this [`TableLayout`] into fragments. This should only be be called
    /// after calling [`TableLayout.compute_measures`].
    fn layout_into_box_fragments(
        mut self,
        positioning_context: &mut PositioningContext,
    ) -> (Vec<Fragment>, Au) {
        assert_eq!(self.table.size.height, self.row_sizes.len());
        assert_eq!(self.table.size.width, self.column_sizes.len());

        let mut fragments = Vec::new();
        let mut row_offset = Au::zero();
        for row_index in 0..self.table.size.height {
            let mut column_offset = Au::zero();
            let row_size = self.row_sizes[row_index];

            for column_index in 0..self.table.size.width {
                let column_size = self.column_sizes[column_index];
                let layout = match self.cells_laid_out[row_index][column_index].take() {
                    Some(layout) => layout,
                    None => continue,
                };

                let cell = match self.table.slots[row_index][column_index] {
                    TableSlot::Cell(ref cell) => cell,
                    _ => {
                        warn!("Did not find a non-spanned cell at index with layout.");
                        continue;
                    },
                };

                let cell_rect: LogicalRect<Length> = LogicalRect {
                    start_corner: LogicalVec2 {
                        inline: column_offset.into(),
                        block: row_offset.into(),
                    },
                    size: LogicalVec2 {
                        inline: column_size.into(),
                        block: row_size.into(),
                    },
                };

                fragments.push(Fragment::Box(cell.create_fragment(
                    layout,
                    cell_rect,
                    positioning_context,
                )));

                column_offset += column_size;
            }

            row_offset += row_size;
        }

        (fragments, row_offset)
    }
}

impl Table {
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
    ) -> (ContentSizes, Vec<ContentSizes>) {
        let mut final_size = ContentSizes::zero();
        let column_content_sizes = (0..self.size.width)
            .map(|column_idx| {
                let coords = TableSlotCoordinates::new(column_idx, 0);
                let content_sizes =
                    self.inline_content_sizes_for_cell_at(coords, layout_context, writing_mode);
                final_size.min_content += content_sizes.min_content;
                final_size.max_content += content_sizes.max_content;
                content_sizes
            })
            .collect();
        (final_size, column_content_sizes)
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
        let (fragments, content_block_size) =
            table_layout.layout_into_box_fragments(positioning_context);
        IndependentLayout {
            fragments,
            content_block_size,
            last_inflow_baseline_offset: None,
        }
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

    fn create_fragment(
        &self,
        mut layout: CellLayout,
        cell_rect: LogicalRect<Length>,
        positioning_context: &mut PositioningContext,
    ) -> BoxFragment {
        // This must be scoped to this function because it conflicts with euclid's Zero.
        use style::Zero as StyleZero;

        let fragments = layout.layout.fragments;
        let content_rect = cell_rect.deflate(&(&layout.padding + &layout.border));

        // Adjust the static position of all absolute children based on the
        // final content rect of this fragment.
        layout
            .positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(
                &content_rect.start_corner,
                PositioningContextLength::zero(),
            );
        positioning_context.append(layout.positioning_context);

        BoxFragment::new(
            self.base_fragment_info,
            self.style.clone(),
            fragments,
            content_rect,
            layout.padding,
            layout.border,
            LogicalSides::zero(), /* margin */
            None,                 /* clearance */
            layout
                .layout
                .last_inflow_baseline_offset
                .map(|baseline| baseline.into()),
            CollapsedBlockMargins::zero(),
        )
    }
}
