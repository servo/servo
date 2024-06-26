/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use std::{cmp, fmt};

use app_units::Au;
use base::print_tree::PrintTree;
use euclid::default::Point2D;
use log::{debug, trace};
use serde::Serialize;
use style::computed_values::{border_collapse, border_spacing, table_layout};
use style::context::SharedStyleContext;
use style::logical_geometry::LogicalSize;
use style::properties::ComputedValues;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::values::computed::Size;
use style::values::CSSFloat;

use crate::block::{
    BlockFlow, CandidateBSizeIterator, ISizeAndMarginsComputer, ISizeConstraintInput,
    ISizeConstraintSolution,
};
use crate::context::LayoutContext;
use crate::display_list::{
    BorderPaintingMode, DisplayListBuildState, StackingContextCollectionFlags,
    StackingContextCollectionState,
};
use crate::flow::{
    BaseFlow, EarlyAbsolutePositionInfo, Flow, FlowClass, GetBaseFlow, ImmutableFlowUtils,
    OpaqueFlow,
};
use crate::flow_list::{FlowListIterator, MutFlowListIterator};
use crate::fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use crate::model::{IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto};
use crate::table_cell::TableCellFlow;
use crate::table_row::{
    self, CellIntrinsicInlineSize, CollapsedBorder, CollapsedBorderFrom, TableRowFlow,
    TableRowSizeData,
};
use crate::table_wrapper::TableLayout;
use crate::{layout_debug, layout_debug_scope};

#[allow(unsafe_code)]
unsafe impl crate::flow::HasBaseFlow for TableFlow {}

/// A table flow corresponded to the table's internal table fragment under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper fragment,
/// not table fragment per CSS 2.1 § 10.5.
#[derive(Serialize)]
#[repr(C)]
pub struct TableFlow {
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each column, computed bottom-up during
    /// intrinsic inline-size bubbling.
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// Information about the actual inline sizes of each column, computed top-down during actual
    /// inline-size bubbling.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,

    /// The final width of the borders in the inline direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_inline_direction_border_widths_for_table: Vec<Au>,

    /// The final width of the borders in the block direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_block_direction_border_widths_for_table: Vec<Au>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableFlow {
    pub fn from_fragment(fragment: Fragment) -> TableFlow {
        let mut block_flow = BlockFlow::from_fragment(fragment);
        let table_layout =
            if block_flow.fragment().style().get_table().table_layout == table_layout::T::Fixed {
                TableLayout::Fixed
            } else {
                TableLayout::Auto
            };
        TableFlow {
            block_flow,
            column_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
            collapsed_inline_direction_border_widths_for_table: Vec::new(),
            collapsed_block_direction_border_widths_for_table: Vec::new(),
            table_layout,
        }
    }

    /// Update the corresponding value of `self_inline_sizes` if a value of `kid_inline_sizes` has
    /// a larger value than one of `self_inline_sizes`. Returns the minimum and preferred inline
    /// sizes.
    fn update_automatic_column_inline_sizes(
        parent_inline_sizes: &mut Vec<ColumnIntrinsicInlineSize>,
        child_cell_inline_sizes: &[CellIntrinsicInlineSize],
        surrounding_size: Au,
    ) -> IntrinsicISizes {
        let mut total_inline_sizes = IntrinsicISizes {
            minimum_inline_size: surrounding_size,
            preferred_inline_size: surrounding_size,
        };
        let mut column_index = 0;
        let mut incoming_rowspan = vec![];

        for child_cell_inline_size in child_cell_inline_sizes {
            // Skip any column occupied by a cell from a previous row.
            while column_index < incoming_rowspan.len() && incoming_rowspan[column_index] != 1 {
                if incoming_rowspan[column_index] > 1 {
                    incoming_rowspan[column_index] -= 1;
                }
                column_index += 1;
            }
            for _ in 0..child_cell_inline_size.column_span {
                if column_index < parent_inline_sizes.len() {
                    // We already have some intrinsic size information for this column. Merge it in
                    // according to the rules specified in INTRINSIC § 4.
                    let parent_sizes = &mut parent_inline_sizes[column_index];
                    if child_cell_inline_size.column_span > 1 {
                        // TODO(pcwalton): Perform the recursive algorithm specified in INTRINSIC §
                        // 4. For now we make this column contribute no width.
                    } else {
                        let column_size = &child_cell_inline_size.column_size;
                        *parent_sizes = ColumnIntrinsicInlineSize {
                            minimum_length: cmp::max(
                                parent_sizes.minimum_length,
                                column_size.minimum_length,
                            ),
                            percentage: parent_sizes.greatest_percentage(column_size),
                            preferred: cmp::max(parent_sizes.preferred, column_size.preferred),
                            constrained: parent_sizes.constrained || column_size.constrained,
                        }
                    }
                } else {
                    // We discovered a new column. Initialize its data.
                    debug_assert_eq!(column_index, parent_inline_sizes.len());
                    if child_cell_inline_size.column_span > 1 {
                        // TODO(pcwalton): Perform the recursive algorithm specified in INTRINSIC §
                        // 4. For now we make this column contribute no width.
                        parent_inline_sizes.push(ColumnIntrinsicInlineSize::new())
                    } else {
                        parent_inline_sizes.push(child_cell_inline_size.column_size)
                    }
                }

                total_inline_sizes.minimum_inline_size +=
                    parent_inline_sizes[column_index].minimum_length;
                total_inline_sizes.preferred_inline_size +=
                    parent_inline_sizes[column_index].preferred;

                // If this cell spans later rows, record its rowspan.
                if child_cell_inline_size.row_span > 1 {
                    if incoming_rowspan.len() < column_index + 1 {
                        incoming_rowspan.resize(column_index + 1, 0);
                    }
                    incoming_rowspan[column_index] = child_cell_inline_size.row_span;
                }

                column_index += 1
            }
        }

        total_inline_sizes
    }

    /// Updates the minimum and preferred inline-size calculation for a single row. This is
    /// factored out into a separate function because we process children of rowgroups too.
    fn update_column_inline_sizes_for_row(
        row: &TableRowFlow,
        column_inline_sizes: &mut Vec<ColumnIntrinsicInlineSize>,
        computation: &mut IntrinsicISizesContribution,
        first_row: bool,
        table_layout: TableLayout,
        surrounding_inline_size: Au,
    ) {
        // Read column inline-sizes from the table-row, and assign inline-size=0 for the columns
        // not defined in the column group.
        //
        // FIXME: Need to read inline-sizes from either table-header-group OR the first table-row.
        match table_layout {
            TableLayout::Fixed => {
                // Fixed table layout only looks at the first row.
                //
                // FIXME(pcwalton): This is really inefficient. We should stop after the first row!
                if first_row {
                    for cell_inline_size in &row.cell_intrinsic_inline_sizes {
                        column_inline_sizes.push(cell_inline_size.column_size);
                    }
                }
            },
            TableLayout::Auto => {
                computation.union_block(&TableFlow::update_automatic_column_inline_sizes(
                    column_inline_sizes,
                    &row.cell_intrinsic_inline_sizes,
                    surrounding_inline_size,
                ))
            },
        }
    }

    /// Returns the effective spacing per cell, taking the value of `border-collapse` into account.
    pub fn spacing(&self) -> border_spacing::T {
        let style = self.block_flow.fragment.style();
        match style.get_inherited_table().border_collapse {
            border_collapse::T::Separate => style.get_inherited_table().border_spacing,
            border_collapse::T::Collapse => border_spacing::T::zero(),
        }
    }

    pub fn total_horizontal_spacing(&self) -> Au {
        let num_columns = self.column_intrinsic_inline_sizes.len();
        if num_columns == 0 {
            return Au(0);
        }
        self.spacing().horizontal() * (num_columns as i32 + 1)
    }

    fn column_styles(&self) -> Vec<ColumnStyle> {
        let mut styles = vec![];
        for group in self
            .block_flow
            .base
            .child_iter()
            .filter(|kid| kid.is_table_colgroup())
        {
            // XXXManishearth these as_foo methods should return options
            // so that we can filter_map
            let group = group.as_table_colgroup();
            let colgroup_style = group.fragment.as_ref().map(|f| f.style());

            // The colgroup's span attribute is only relevant when
            // it has no children
            // https://html.spec.whatwg.org/multipage/#forming-a-table
            if group.cols.is_empty() {
                let span = group
                    .fragment
                    .as_ref()
                    .map(|f| f.column_span())
                    .unwrap_or(1);
                styles.push(ColumnStyle {
                    span,
                    colgroup_style,
                    col_style: None,
                });
            } else {
                for col in &group.cols {
                    // XXXManishearth Arc-cloning colgroup_style is suboptimal
                    styles.push(ColumnStyle {
                        span: col.column_span(),
                        colgroup_style,
                        col_style: Some(col.style()),
                    })
                }
            }
        }
        styles
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Table
    }

    fn as_mut_table(&mut self) -> &mut TableFlow {
        self
    }

    fn as_table(&self) -> &TableFlow {
        self
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    fn mark_as_root(&mut self) {
        self.block_flow.mark_as_root();
    }

    /// The specified column inline-sizes are set from column group and the first row for the fixed
    /// table layout calculation.
    /// The maximum min/pref inline-sizes of each column are set from the rows for the automatic
    /// table layout calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!(
            "table::bubble_inline_sizes {:x}",
            self.block_flow.base.debug_id()
        );

        // Get column inline sizes from colgroups
        for kid in self
            .block_flow
            .base
            .child_iter_mut()
            .filter(|kid| kid.is_table_colgroup())
        {
            for specified_inline_size in &kid.as_mut_table_colgroup().inline_sizes {
                self.column_intrinsic_inline_sizes
                    .push(ColumnIntrinsicInlineSize {
                        minimum_length: match *specified_inline_size {
                            Size::Auto => Au(0),
                            Size::LengthPercentage(ref lp) => {
                                lp.maybe_to_used_value(None).unwrap_or(Au(0))
                            },
                        },
                        percentage: match *specified_inline_size {
                            Size::Auto => 0.0,
                            Size::LengthPercentage(ref lp) => {
                                lp.0.to_percentage().map_or(0.0, |p| p.0)
                            },
                        },
                        preferred: Au(0),
                        constrained: false,
                    })
            }
        }

        self.collapsed_inline_direction_border_widths_for_table = Vec::new();
        self.collapsed_block_direction_border_widths_for_table = vec![Au(0)];

        let collapsing_borders = self
            .block_flow
            .fragment
            .style
            .get_inherited_table()
            .border_collapse ==
            border_collapse::T::Collapse;
        let table_inline_collapsed_borders = if collapsing_borders {
            Some(TableInlineCollapsedBorders {
                start: CollapsedBorder::inline_start(
                    &self.block_flow.fragment.style,
                    CollapsedBorderFrom::Table,
                ),
                end: CollapsedBorder::inline_end(
                    &self.block_flow.fragment.style,
                    CollapsedBorderFrom::Table,
                ),
            })
        } else {
            None
        };

        let mut computation = IntrinsicISizesContribution::new();
        let mut previous_collapsed_block_end_borders =
            PreviousBlockCollapsedBorders::FromTable(CollapsedBorder::block_start(
                &self.block_flow.fragment.style,
                CollapsedBorderFrom::Table,
            ));
        let mut first_row = true;
        let (border_padding, _) = self.block_flow.fragment.surrounding_intrinsic_inline_size();

        {
            let mut iterator = TableRowIterator::new(&mut self.block_flow.base).peekable();
            while let Some(row) = iterator.next() {
                TableFlow::update_column_inline_sizes_for_row(
                    row,
                    &mut self.column_intrinsic_inline_sizes,
                    &mut computation,
                    first_row,
                    self.table_layout,
                    border_padding,
                );
                if collapsing_borders {
                    let next_index_and_sibling = iterator.peek();
                    let next_collapsed_borders_in_block_direction = match next_index_and_sibling {
                        Some(next_sibling) => NextBlockCollapsedBorders::FromNextRow(
                            &next_sibling
                                .as_table_row()
                                .preliminary_collapsed_borders
                                .block_start,
                        ),
                        None => NextBlockCollapsedBorders::FromTable(CollapsedBorder::block_end(
                            &self.block_flow.fragment.style,
                            CollapsedBorderFrom::Table,
                        )),
                    };
                    perform_border_collapse_for_row(
                        row,
                        table_inline_collapsed_borders.as_ref().unwrap(),
                        previous_collapsed_block_end_borders,
                        next_collapsed_borders_in_block_direction,
                        &mut self.collapsed_inline_direction_border_widths_for_table,
                        &mut self.collapsed_block_direction_border_widths_for_table,
                    );
                    previous_collapsed_block_end_borders =
                        PreviousBlockCollapsedBorders::FromPreviousRow(
                            row.final_collapsed_borders.block_end.clone(),
                        );
                }
                first_row = false
            }
        }

        let total_horizontal_spacing = self.total_horizontal_spacing();
        let mut style_specified_intrinsic_inline_size = self
            .block_flow
            .fragment
            .style_specified_intrinsic_inline_size()
            .finish();
        style_specified_intrinsic_inline_size.minimum_inline_size -= total_horizontal_spacing;
        style_specified_intrinsic_inline_size.preferred_inline_size -= total_horizontal_spacing;
        computation.union_block(&style_specified_intrinsic_inline_size);
        computation.surrounding_size += total_horizontal_spacing;

        self.block_flow.base.intrinsic_inline_sizes = computation.finish()
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!(
            "table::assign_inline_sizes {:x}",
            self.block_flow.base.debug_id()
        );
        debug!(
            "assign_inline_sizes({}): assigning inline_size for flow",
            "table"
        );
        trace!("TableFlow before assigning: {:?}", &self);

        let shared_context = layout_context.shared_context();
        // The position was set to the containing block by the flow's parent.
        // FIXME: The code for distributing column widths should really be placed under table_wrapper.rs.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;

        let mut constrained_column_inline_sizes_indices = vec![];
        let mut unspecified_inline_sizes_indices = vec![];
        for (idx, column_inline_size) in self.column_intrinsic_inline_sizes.iter().enumerate() {
            if column_inline_size.constrained {
                constrained_column_inline_sizes_indices.push(idx);
            } else if column_inline_size.percentage == 0.0 {
                unspecified_inline_sizes_indices.push(idx);
            }
        }

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(
            &mut self.block_flow,
            shared_context,
            containing_block_inline_size,
        );

        let inline_start_content_edge = self.block_flow.fragment.border_padding.inline_start;
        let inline_end_content_edge = self.block_flow.fragment.border_padding.inline_end;
        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();
        let spacing_per_cell = self.spacing();
        let total_horizontal_spacing = self.total_horizontal_spacing();
        let content_inline_size = self.block_flow.fragment.border_box.size.inline -
            padding_and_borders -
            total_horizontal_spacing;
        let mut remaining_inline_size = content_inline_size;

        match self.table_layout {
            TableLayout::Fixed => {
                self.column_computed_inline_sizes.clear();

                // https://drafts.csswg.org/css2/tables.html#fixed-table-layout
                for column_inline_size in &self.column_intrinsic_inline_sizes {
                    if column_inline_size.constrained {
                        self.column_computed_inline_sizes
                            .push(ColumnComputedInlineSize {
                                size: column_inline_size.minimum_length,
                            });
                        remaining_inline_size -= column_inline_size.minimum_length;
                    } else if column_inline_size.percentage != 0.0 {
                        let size = remaining_inline_size.scale_by(column_inline_size.percentage);
                        self.column_computed_inline_sizes
                            .push(ColumnComputedInlineSize { size });
                        remaining_inline_size -= size;
                    } else {
                        // Set the size to 0 now, distribute the remaining widths later
                        self.column_computed_inline_sizes
                            .push(ColumnComputedInlineSize { size: Au(0) });
                    }
                }

                // Distribute remaining content inline size
                if !unspecified_inline_sizes_indices.is_empty() {
                    for &index in &unspecified_inline_sizes_indices {
                        self.column_computed_inline_sizes[index].size = remaining_inline_size
                            .scale_by(1.0 / unspecified_inline_sizes_indices.len() as f32);
                    }
                } else {
                    let total_minimum_size = self
                        .column_intrinsic_inline_sizes
                        .iter()
                        .filter(|size| size.constrained)
                        .map(|size| size.minimum_length.0 as f32)
                        .sum::<f32>();

                    for &index in &constrained_column_inline_sizes_indices {
                        let inline_size = self.column_computed_inline_sizes[index].size.0;
                        self.column_computed_inline_sizes[index].size +=
                            remaining_inline_size.scale_by(inline_size as f32 / total_minimum_size);
                    }
                }
            },
            _ => {
                // The table wrapper already computed the inline-sizes and propagated them down
                // to us.
            },
        }

        let column_computed_inline_sizes = &self.column_computed_inline_sizes;
        let collapsed_inline_direction_border_widths_for_table =
            &self.collapsed_inline_direction_border_widths_for_table;
        let mut collapsed_block_direction_border_widths_for_table = self
            .collapsed_block_direction_border_widths_for_table
            .iter()
            .peekable();
        let mut incoming_rowspan = vec![];
        self.block_flow.propagate_assigned_inline_size_to_children(
            shared_context,
            inline_start_content_edge,
            inline_end_content_edge,
            content_inline_size,
            |child_flow,
             _child_index,
             _content_inline_size,
             writing_mode,
             _inline_start_margin_edge,
             _inline_end_margin_edge| {
                table_row::propagate_column_inline_sizes_to_child(
                    child_flow,
                    writing_mode,
                    column_computed_inline_sizes,
                    &spacing_per_cell,
                    &mut incoming_rowspan,
                );
                if child_flow.is_table_row() {
                    let child_table_row = child_flow.as_mut_table_row();
                    child_table_row.populate_collapsed_border_spacing(
                        collapsed_inline_direction_border_widths_for_table,
                        &mut collapsed_block_direction_border_widths_for_table,
                    );
                } else if child_flow.is_table_rowgroup() {
                    let child_table_rowgroup = child_flow.as_mut_table_rowgroup();
                    child_table_rowgroup.populate_collapsed_border_spacing(
                        collapsed_inline_direction_border_widths_for_table,
                        &mut collapsed_block_direction_border_widths_for_table,
                    );
                }
            },
        );

        trace!("TableFlow after assigning: {:?}", &self);
    }

    fn assign_block_size(&mut self, lc: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table");
        trace!("TableFlow before assigning: {:?}", &self);

        let vertical_spacing = self.spacing().vertical();
        self.block_flow
            .assign_block_size_for_table_like_flow(vertical_spacing, lc);

        trace!("TableFlow after assigning: {:?}", &self);
    }

    fn compute_stacking_relative_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow
            .compute_stacking_relative_position(layout_context)
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow
            .update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow
            .update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        let border_painting_mode = match self
            .block_flow
            .fragment
            .style
            .get_inherited_table()
            .border_collapse
        {
            border_collapse::T::Separate => BorderPaintingMode::Separate,
            border_collapse::T::Collapse => BorderPaintingMode::Hidden,
        };

        self.block_flow
            .build_display_list_for_block(state, border_painting_mode);

        let iter = TableCellStyleIterator::new(self);
        for style in iter {
            style.build_display_list(state)
        }
    }

    fn collect_stacking_contexts(&mut self, state: &mut StackingContextCollectionState) {
        // Stacking contexts are collected by the table wrapper.
        self.block_flow.collect_stacking_contexts_for_block(
            state,
            StackingContextCollectionFlags::NEVER_CREATES_STACKING_CONTEXT,
        );
    }

    fn repair_style(&mut self, new_style: &crate::ServoArc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
    }

    fn iterate_through_fragment_border_boxes(
        &self,
        iterator: &mut dyn FragmentBorderBoxIterator,
        level: i32,
        stacking_context_position: &Point2D<Au>,
    ) {
        self.block_flow.iterate_through_fragment_border_boxes(
            iterator,
            level,
            stacking_context_position,
        )
    }

    fn mutate_fragments(&mut self, mutator: &mut dyn FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        self.block_flow.print_extra_flow_children(print_tree);
    }
}

#[derive(Debug)]
struct ColumnStyle<'table> {
    span: u32,
    colgroup_style: Option<&'table ComputedValues>,
    col_style: Option<&'table ComputedValues>,
}

impl fmt::Debug for TableFlow {
    /// Outputs a debugging string describing this table flow.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableFlow: {:?}", self.block_flow)
    }
}

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their inline-sizes are calculated in the same way and do not have margins.
pub struct InternalTable;

impl ISizeAndMarginsComputer for InternalTable {
    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(
        &self,
        block: &mut BlockFlow,
        shared_context: &SharedStyleContext,
        parent_flow_inline_size: Au,
    ) {
        let mut input = self.compute_inline_size_constraint_inputs(
            block,
            parent_flow_inline_size,
            shared_context,
        );

        // Tables are always at least as wide as their minimum inline size.
        let minimum_inline_size = block.base.intrinsic_inline_sizes.minimum_inline_size -
            block.fragment.border_padding.inline_start_end();
        input.available_inline_size = cmp::max(input.available_inline_size, minimum_inline_size);

        let solution = self.solve_inline_size_constraints(block, &input);
        self.set_inline_size_constraint_solutions(block, solution);
    }

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(
        &self,
        _: &mut BlockFlow,
        input: &ISizeConstraintInput,
    ) -> ISizeConstraintSolution {
        ISizeConstraintSolution::new(input.available_inline_size, Au(0), Au(0))
    }
}

/// Information about the intrinsic inline sizes of columns within a table.
///
/// During table inline-size bubbling, we might need to store both a percentage constraint and a
/// specific width constraint. For instance, one cell might say that it wants to be 100 pixels wide
/// in the inline direction and another cell might say that it wants to take up 20% of the inline-
/// size of the table. Now because we bubble up these constraints during the bubble-inline-sizes
/// phase of layout, we don't know yet how wide the table is ultimately going to be in the inline
/// direction. As we need to pick the maximum width of all cells for a column (in this case, the
/// maximum of 100 pixels and 20% of the table), the preceding constraint means that we must
/// potentially store both a specified width *and* a specified percentage, so that the inline-size
/// assignment phase of layout will know which one to pick.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ColumnIntrinsicInlineSize {
    /// The preferred intrinsic inline size.
    pub preferred: Au,
    /// The largest specified size of this column as a length.
    pub minimum_length: Au,
    /// The largest specified size of this column as a percentage (`width` property).
    pub percentage: CSSFloat,
    /// Whether the column inline size is *constrained* per INTRINSIC § 4.1.
    pub constrained: bool,
}

impl ColumnIntrinsicInlineSize {
    /// Returns a newly-initialized `ColumnIntrinsicInlineSize` with all fields blank.
    pub fn new() -> ColumnIntrinsicInlineSize {
        ColumnIntrinsicInlineSize {
            preferred: Au(0),
            minimum_length: Au(0),
            percentage: 0.0,
            constrained: false,
        }
    }

    /// Returns the higher of the two percentages specified in `self` and `other`.
    pub fn greatest_percentage(&self, other: &ColumnIntrinsicInlineSize) -> CSSFloat {
        if self.percentage > other.percentage {
            self.percentage
        } else {
            other.percentage
        }
    }
}

impl Default for ColumnIntrinsicInlineSize {
    fn default() -> Self {
        Self::new()
    }
}

/// The actual inline size for each column.
///
/// TODO(pcwalton): There will probably be some `border-collapse`-related info in here too
/// eventually.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ColumnComputedInlineSize {
    /// The computed size of this inline column.
    pub size: Au,
}

pub trait VecExt<T> {
    fn push_or_set(&mut self, index: usize, value: T) -> &mut T;
    fn get_mut_or_push(&mut self, index: usize, zero: T) -> &mut T;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_or_set(&mut self, index: usize, value: T) -> &mut T {
        if index < self.len() {
            self[index] = value
        } else {
            debug_assert_eq!(index, self.len());
            self.push(value)
        }
        &mut self[index]
    }

    fn get_mut_or_push(&mut self, index: usize, zero: T) -> &mut T {
        if index >= self.len() {
            debug_assert_eq!(index, self.len());
            self.push(zero)
        }
        &mut self[index]
    }
}

/// Updates the border styles in the block direction for a single row. This function should
/// only be called if border collapsing is on. It is factored out into a separate function
/// because we process children of rowgroups too.
fn perform_border_collapse_for_row(
    child_table_row: &mut TableRowFlow,
    table_inline_borders: &TableInlineCollapsedBorders,
    previous_block_borders: PreviousBlockCollapsedBorders,
    next_block_borders: NextBlockCollapsedBorders,
    inline_spacing: &mut Vec<Au>,
    block_spacing: &mut Vec<Au>,
) {
    // TODO mbrubeck: Take rowspan and colspan into account.
    let number_of_borders_inline_direction =
        child_table_row.preliminary_collapsed_borders.inline.len();
    // Compute interior inline borders.
    for (i, this_inline_border) in child_table_row
        .preliminary_collapsed_borders
        .inline
        .iter_mut()
        .enumerate()
    {
        child_table_row
            .final_collapsed_borders
            .inline
            .push_or_set(i, this_inline_border.clone());
        if i == 0 {
            child_table_row.final_collapsed_borders.inline[i].combine(&table_inline_borders.start);
        } else if i + 1 == number_of_borders_inline_direction {
            child_table_row.final_collapsed_borders.inline[i].combine(&table_inline_borders.end);
        }

        let inline_spacing = inline_spacing.get_mut_or_push(i, Au(0));
        *inline_spacing = cmp::max(
            *inline_spacing,
            child_table_row.final_collapsed_borders.inline[i].width,
        )
    }

    // Compute block-start borders.
    let block_start_borders = &mut child_table_row.final_collapsed_borders.block_start;

    block_start_borders.clone_from(&child_table_row.preliminary_collapsed_borders.block_start);

    for (i, this_border) in block_start_borders.iter_mut().enumerate() {
        match previous_block_borders {
            PreviousBlockCollapsedBorders::FromPreviousRow(ref previous_block_borders) => {
                if previous_block_borders.len() > i {
                    this_border.combine(&previous_block_borders[i]);
                }
            },
            PreviousBlockCollapsedBorders::FromTable(ref table_border) => {
                this_border.combine(table_border);
            },
        }
    }

    // Compute block-end borders.
    let next_block = &mut child_table_row.final_collapsed_borders.block_end;
    block_spacing.push(Au(0));
    let block_spacing = block_spacing.last_mut().unwrap();
    for (i, this_block_border) in child_table_row
        .preliminary_collapsed_borders
        .block_end
        .iter()
        .enumerate()
    {
        let next_block = next_block.push_or_set(i, this_block_border.clone());
        match next_block_borders {
            NextBlockCollapsedBorders::FromNextRow(next_block_borders) => {
                if next_block_borders.len() > i {
                    next_block.combine(&next_block_borders[i])
                }
            },
            NextBlockCollapsedBorders::FromTable(ref next_block_borders) => {
                next_block.combine(next_block_borders);
            },
        }
        *block_spacing = cmp::max(*block_spacing, next_block.width)
    }
}

/// Encapsulates functionality shared among all table-like flows: for now, tables and table
/// rowgroups.
pub trait TableLikeFlow {
    /// Lays out the rows of a table.
    fn assign_block_size_for_table_like_flow(
        &mut self,
        block_direction_spacing: Au,
        layout_context: &LayoutContext,
    );
}

impl TableLikeFlow for BlockFlow {
    fn assign_block_size_for_table_like_flow(
        &mut self,
        block_direction_spacing: Au,
        layout_context: &LayoutContext,
    ) {
        debug_assert!(
            self.fragment.style.get_inherited_table().border_collapse ==
                border_collapse::T::Separate ||
                block_direction_spacing == Au(0)
        );

        fn border_spacing_for_row(
            fragment: &Fragment,
            row: &TableRowFlow,
            block_direction_spacing: Au,
        ) -> Au {
            match fragment.style.get_inherited_table().border_collapse {
                border_collapse::T::Separate => block_direction_spacing,
                border_collapse::T::Collapse => row.collapsed_border_spacing.block_start,
            }
        }

        if self
            .base
            .restyle_damage
            .contains(ServoRestyleDamage::REFLOW)
        {
            let mut sizes = vec![Default::default()];
            // The amount of border spacing up to and including this row,
            // but not including the spacing beneath it
            let mut cumulative_border_spacing = Au(0);
            let mut incoming_rowspan_data = vec![];
            let mut rowgroup_id = 0;
            let mut first = true;

            // First pass: Compute block-direction border spacings
            // XXXManishearth this can be done in tandem with the second pass,
            // provided we never hit any rowspan cases
            for kid in self.base.child_iter_mut() {
                if kid.is_table_row() {
                    // skip the first row, it is accounted for
                    if first {
                        first = false;
                        continue;
                    }
                    cumulative_border_spacing += border_spacing_for_row(
                        &self.fragment,
                        kid.as_table_row(),
                        block_direction_spacing,
                    );
                    sizes.push(TableRowSizeData {
                        // we haven't calculated sizes yet
                        size: Au(0),
                        cumulative_border_spacing,
                        rowgroup_id,
                    });
                } else if kid.is_table_rowgroup() && !first {
                    rowgroup_id += 1;
                }
            }

            // Second pass: Compute row block sizes
            // [expensive: iterates over cells]
            let mut i = 0;
            for kid in self.base.child_iter_mut() {
                if kid.is_table_row() {
                    let size = kid.as_mut_table_row().compute_block_size_table_row_base(
                        layout_context,
                        &mut incoming_rowspan_data,
                        &sizes,
                        i,
                    );
                    sizes[i].size = size;
                    i += 1;
                }
            }

            // Our current border-box position.
            let block_start_border_padding = self.fragment.border_padding.block_start;
            let mut current_block_offset = block_start_border_padding;
            let mut has_rows = false;

            // Third pass: Assign block sizes and positions to rows, cells, and other children
            // [expensive: iterates over cells]
            // At this point, `current_block_offset` is at the content edge of our box. Now iterate
            // over children.
            let mut i = 0;
            for kid in self.base.child_iter_mut() {
                if kid.is_table_row() {
                    has_rows = true;
                    let row = kid.as_mut_table_row();
                    row.assign_block_size_to_self_and_children(&sizes, i);
                    row.mut_base().restyle_damage.remove(
                        ServoRestyleDamage::REFLOW_OUT_OF_FLOW | ServoRestyleDamage::REFLOW,
                    );
                    current_block_offset +=
                        border_spacing_for_row(&self.fragment, row, block_direction_spacing);
                    i += 1;
                }

                // At this point, `current_block_offset` is at the border edge of the child.
                kid.mut_base().position.start.b = current_block_offset;

                // Move past the child's border box. Do not use the `translate_including_floats`
                // function here because the child has already translated floats past its border
                // box.
                let kid_base = kid.mut_base();
                current_block_offset += kid_base.position.size.block;
            }

            // Compute any explicitly-specified block size.
            // Can't use `for` because we assign to
            // `candidate_block_size_iterator.candidate_value`.
            let mut block_size = current_block_offset - block_start_border_padding;
            let mut candidate_block_size_iterator = CandidateBSizeIterator::new(
                &self.fragment,
                self.base.block_container_explicit_block_size,
            );
            while let Some(candidate_block_size) = candidate_block_size_iterator.next() {
                candidate_block_size_iterator.candidate_value = match candidate_block_size {
                    MaybeAuto::Auto => block_size,
                    MaybeAuto::Specified(value) => value,
                };
            }

            // Adjust `current_block_offset` as necessary to account for the explicitly-specified
            // block-size.
            block_size = candidate_block_size_iterator.candidate_value;
            let delta = block_size - (current_block_offset - block_start_border_padding);
            current_block_offset += delta;

            // Take border, padding, and spacing into account.
            let block_end_offset = self.fragment.border_padding.block_end +
                if has_rows {
                    block_direction_spacing
                } else {
                    Au(0)
                };
            current_block_offset += block_end_offset;

            // Now that `current_block_offset` is at the block-end of the border box, compute the
            // final border box position.
            self.fragment.border_box.size.block = current_block_offset;
            self.fragment.border_box.start.b = Au(0);
            self.base.position.size.block = current_block_offset;

            // Fourth pass: Assign absolute position info
            // Write in the size of the relative containing block for children. (This information
            // is also needed to handle RTL.)
            for kid in self.base.child_iter_mut() {
                kid.mut_base().early_absolute_position_info = EarlyAbsolutePositionInfo {
                    relative_containing_block_size: self.fragment.content_box().size,
                    relative_containing_block_mode: self.fragment.style().writing_mode,
                };
            }
        }

        self.base
            .restyle_damage
            .remove(ServoRestyleDamage::REFLOW_OUT_OF_FLOW | ServoRestyleDamage::REFLOW);
    }
}

/// Inline collapsed borders for the table itself.
#[derive(Debug)]
struct TableInlineCollapsedBorders {
    /// The table border at the start of the inline direction.
    start: CollapsedBorder,
    /// The table border at the end of the inline direction.
    end: CollapsedBorder,
}

enum PreviousBlockCollapsedBorders {
    FromPreviousRow(Vec<CollapsedBorder>),
    FromTable(CollapsedBorder),
}

enum NextBlockCollapsedBorders<'a> {
    FromNextRow(&'a [CollapsedBorder]),
    FromTable(CollapsedBorder),
}

/// Iterator over all the rows of a table, which also
/// provides the Fragment for rowgroups if any
struct TableRowAndGroupIterator<'a> {
    kids: FlowListIterator<'a>,
    group: Option<(&'a Fragment, FlowListIterator<'a>)>,
}

impl<'a> TableRowAndGroupIterator<'a> {
    fn new(base: &'a BaseFlow) -> Self {
        TableRowAndGroupIterator {
            kids: base.child_iter(),
            group: None,
        }
    }
}

impl<'a> Iterator for TableRowAndGroupIterator<'a> {
    type Item = (Option<&'a Fragment>, &'a TableRowFlow);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // If we're inside a rowgroup, iterate through the rowgroup's children.
        if let Some(ref mut group) = self.group {
            if let Some(grandkid) = group.1.next() {
                return Some((Some(group.0), grandkid.as_table_row()));
            }
        }
        // Otherwise, iterate through the table's children.
        self.group = None;
        match self.kids.next() {
            Some(kid) => {
                if kid.is_table_rowgroup() {
                    let rowgroup = kid.as_table_rowgroup();
                    let iter = rowgroup.block_flow.base.child_iter();
                    self.group = Some((&rowgroup.block_flow.fragment, iter));
                    self.next()
                } else if kid.is_table_row() {
                    Some((None, kid.as_table_row()))
                } else {
                    self.next() // Skip children that are not rows or rowgroups
                }
            },
            None => None,
        }
    }
}

/// Iterator over all the rows of a table, which also
/// provides the Fragment for rowgroups if any
struct MutTableRowAndGroupIterator<'a> {
    kids: MutFlowListIterator<'a>,
    group: Option<(&'a Fragment, MutFlowListIterator<'a>)>,
}

impl<'a> MutTableRowAndGroupIterator<'a> {
    fn new(base: &'a mut BaseFlow) -> Self {
        MutTableRowAndGroupIterator {
            kids: base.child_iter_mut(),
            group: None,
        }
    }
}

impl<'a> Iterator for MutTableRowAndGroupIterator<'a> {
    type Item = (Option<&'a Fragment>, &'a mut TableRowFlow);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // If we're inside a rowgroup, iterate through the rowgroup's children.
        if let Some(ref mut group) = self.group {
            if let Some(grandkid) = group.1.next() {
                return Some((Some(group.0), grandkid.as_mut_table_row()));
            }
        }
        // Otherwise, iterate through the table's children.
        self.group = None;
        match self.kids.next() {
            Some(kid) => {
                if kid.is_table_rowgroup() {
                    let rowgroup = kid.as_mut_table_rowgroup();
                    let iter = rowgroup.block_flow.base.child_iter_mut();
                    self.group = Some((&rowgroup.block_flow.fragment, iter));
                    self.next()
                } else if kid.is_table_row() {
                    Some((None, kid.as_mut_table_row()))
                } else {
                    self.next() // Skip children that are not rows or rowgroups
                }
            },
            None => None,
        }
    }
}

/// Iterator over all the rows of a table
struct TableRowIterator<'a>(MutTableRowAndGroupIterator<'a>);

impl<'a> TableRowIterator<'a> {
    fn new(base: &'a mut BaseFlow) -> Self {
        TableRowIterator(MutTableRowAndGroupIterator::new(base))
    }
}

impl<'a> Iterator for TableRowIterator<'a> {
    type Item = &'a mut TableRowFlow;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|n| n.1)
    }
}

/// An iterator over table cells, yielding all relevant style objects
/// for each cell
///
/// Used for correctly handling table layers from
/// <https://drafts.csswg.org/css2/tables.html#table-layers>
struct TableCellStyleIterator<'table> {
    column_styles: Vec<ColumnStyle<'table>>,
    row_iterator: TableRowAndGroupIterator<'table>,
    row_info: Option<TableCellStyleIteratorRowInfo<'table>>,
    column_index: TableCellColumnIndexData,
}

struct TableCellStyleIteratorRowInfo<'table> {
    row: &'table TableRowFlow,
    rowgroup: Option<&'table Fragment>,
    cell_iterator: FlowListIterator<'table>,
}

impl<'table> TableCellStyleIterator<'table> {
    fn new(table: &'table TableFlow) -> Self {
        let column_styles = table.column_styles();
        let mut row_iterator = TableRowAndGroupIterator::new(&table.block_flow.base);
        let row_info = if let Some((group, row)) = row_iterator.next() {
            Some(TableCellStyleIteratorRowInfo {
                row,
                rowgroup: group,
                cell_iterator: row.block_flow.base.child_iter(),
            })
        } else {
            None
        };
        TableCellStyleIterator {
            column_styles,
            row_iterator,
            row_info,
            column_index: Default::default(),
        }
    }
}

struct TableCellStyleInfo<'table> {
    cell: &'table TableCellFlow,
    colgroup_style: Option<&'table ComputedValues>,
    col_style: Option<&'table ComputedValues>,
    rowgroup_style: Option<&'table ComputedValues>,
    row_style: &'table ComputedValues,
}

#[derive(Default)]
struct TableCellColumnIndexData {
    /// Which column this is in the table
    pub absolute: u32,
    /// The index of the current column in column_styles
    /// (i.e. which <col> element it is)
    pub relative: u32,
    /// In case of multispan <col>s, where we are in the
    /// span of the current <col> element
    pub relative_offset: u32,
}

impl TableCellColumnIndexData {
    /// Moves forward by `amount` columns, updating the various indices used
    ///
    /// This totally ignores rowspan -- if colspan and rowspan clash,
    /// they just overlap, so we ignore it.
    fn advance(&mut self, amount: u32, column_styles: &[ColumnStyle]) {
        self.absolute += amount;
        self.relative_offset += amount;
        if let Some(mut current_col) = column_styles.get(self.relative as usize) {
            while self.relative_offset >= current_col.span {
                // move to the next column
                self.relative += 1;
                self.relative_offset -= current_col.span;
                if let Some(column_style) = column_styles.get(self.relative as usize) {
                    current_col = column_style;
                } else {
                    // we ran out of column_styles,
                    // so we don't need to update the indices
                    break;
                }
            }
        }
    }
}

impl<'table> Iterator for TableCellStyleIterator<'table> {
    type Item = TableCellStyleInfo<'table>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // FIXME We do this awkward .take() followed by shoving it back in
        // because without NLL the row_info borrow lasts too long
        if let Some(mut row_info) = self.row_info.take() {
            if let Some(rowspan) = row_info
                .row
                .incoming_rowspan
                .get(self.column_index.absolute as usize)
            {
                // we are not allowed to use this column as a starting point. Try the next one.
                if *rowspan > 1 {
                    self.column_index.advance(1, &self.column_styles);
                    // put row_info back in
                    self.row_info = Some(row_info);
                    // try again
                    return self.next();
                }
            }
            if let Some(cell) = row_info.cell_iterator.next() {
                let rowgroup_style = row_info.rowgroup.map(|r| r.style());
                let row_style = row_info.row.block_flow.fragment.style();
                let cell = cell.as_table_cell();
                let (col_style, colgroup_style) = if let Some(column_style) =
                    self.column_styles.get(self.column_index.relative as usize)
                {
                    let styles = (column_style.col_style, column_style.colgroup_style);
                    self.column_index
                        .advance(cell.column_span, &self.column_styles);

                    styles
                } else {
                    (None, None)
                };
                // put row_info back in
                self.row_info = Some(row_info);
                Some(TableCellStyleInfo {
                    cell,
                    colgroup_style,
                    col_style,
                    rowgroup_style,
                    row_style,
                })
            } else {
                // next row
                if let Some((group, row)) = self.row_iterator.next() {
                    self.row_info = Some(TableCellStyleIteratorRowInfo {
                        row,
                        rowgroup: group,
                        cell_iterator: row.block_flow.base.child_iter(),
                    });
                    self.column_index = Default::default();
                    self.next()
                } else {
                    // out of rows
                    // row_info stays None
                    None
                }
            }
        } else {
            // empty table
            None
        }
    }
}

impl<'table> TableCellStyleInfo<'table> {
    fn build_display_list(&self, mut state: &mut DisplayListBuildState) {
        use style::computed_values::visibility::T as Visibility;

        if !self.cell.visible ||
            self.cell
                .block_flow
                .fragment
                .style()
                .get_inherited_box()
                .visibility !=
                Visibility::Visible
        {
            return;
        }
        let border_painting_mode = match self
            .cell
            .block_flow
            .fragment
            .style
            .get_inherited_table()
            .border_collapse
        {
            border_collapse::T::Separate => BorderPaintingMode::Separate,
            border_collapse::T::Collapse => {
                BorderPaintingMode::Collapse(&self.cell.collapsed_borders)
            },
        };
        {
            let cell_flow = &self.cell.block_flow;

            let build_dl = |sty: &ComputedValues, state: &mut &mut DisplayListBuildState| {
                let background = sty.get_background();
                let background_color = sty.resolve_color(background.background_color.clone());
                cell_flow.build_display_list_for_background_if_applicable_with_background(
                    state,
                    background,
                    background_color,
                );
            };

            if let Some(sty) = self.colgroup_style {
                build_dl(sty, &mut state);
            }
            if let Some(sty) = self.col_style {
                build_dl(sty, &mut state);
            }
            if let Some(sty) = self.rowgroup_style {
                build_dl(sty, &mut state);
            }
            build_dl(self.row_style, &mut state);
        }
        // the restyle damage will be set in TableCellFlow::build_display_list()
        self.cell
            .block_flow
            .build_display_list_for_block_no_damage(state, border_painting_mode)
    }
}
