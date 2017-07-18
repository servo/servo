/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_code)]

use app_units::Au;
use block::{BlockFlow, ISizeAndMarginsComputer};
use context::LayoutContext;
use display_list_builder::{BlockFlowDisplayListBuilding, BorderPaintingMode, DisplayListBuildState};
use euclid::Point2D;
use flow::{self, EarlyAbsolutePositionInfo, Flow, FlowClass, ImmutableFlowUtils, OpaqueFlow};
use flow_list::MutFlowListIterator;
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx_traits::print_tree::PrintTree;
use layout_debug;
use model::MaybeAuto;
use serde::{Serialize, Serializer};
use std::cmp::max;
use std::fmt;
use std::iter::{Enumerate, IntoIterator, Peekable};
use style::computed_values::{border_collapse, border_spacing, border_top_style};
use style::logical_geometry::{LogicalSize, PhysicalSide, WritingMode};
use style::properties::ComputedValues;
use style::servo::restyle_damage::{REFLOW, REFLOW_OUT_OF_FLOW};
use style::values::computed::{Color, LengthOrPercentageOrAuto};
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize, InternalTable, VecExt};
use table_cell::{CollapsedBordersForCell, TableCellFlow};

/// A single row of a table.
pub struct TableRowFlow {
    /// Fields common to all block flows.
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each cell.
    pub cell_intrinsic_inline_sizes: Vec<CellIntrinsicInlineSize>,

    /// Information about the computed inline-sizes of each column.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,

    /// The number of remaining rows spanned by cells in previous rows, indexed by column.
    ///
    /// Columns that are not included in this vector have the default rowspan of "1".  If there are
    /// no cells with rowspan != 1 in previous rows, this vector may be empty.
    pub incoming_rowspan: Vec<u32>,

    /// The spacing for this row, propagated down from the table during the inline-size assignment
    /// phase.
    pub spacing: border_spacing::T,

    /// The direction of the columns, propagated down from the table during the inline-size
    /// assignment phase.
    pub table_writing_mode: WritingMode,

    /// Information about the borders for each cell that we bubble up to our parent. This is only
    /// computed if `border-collapse` is `collapse`.
    pub preliminary_collapsed_borders: CollapsedBordersForRow,

    /// Information about the borders for each cell, post-collapse. This is only computed if
    /// `border-collapse` is `collapse`.
    pub final_collapsed_borders: CollapsedBordersForRow,

    /// The computed cell spacing widths post-collapse.
    pub collapsed_border_spacing: CollapsedBorderSpacingForRow,
}

impl Serialize for TableRowFlow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.block_flow.serialize(serializer)
    }
}

/// Information about the column inline size and span for each cell.
#[derive(Serialize, Copy, Clone)]
pub struct CellIntrinsicInlineSize {
    /// Inline sizes that this cell contributes to the column.
    pub column_size: ColumnIntrinsicInlineSize,
    /// The column span of this cell.
    pub column_span: u32,
    /// The row span of this cell.
    pub row_span: u32,
}


impl TableRowFlow {
    pub fn from_fragment(fragment: Fragment) -> TableRowFlow {
        let writing_mode = fragment.style().writing_mode;
        TableRowFlow {
            block_flow: BlockFlow::from_fragment(fragment),
            cell_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
            incoming_rowspan: Vec::new(),
            spacing: border_spacing::T {
                horizontal: Au(0),
                vertical: Au(0),
            },
            table_writing_mode: writing_mode,
            preliminary_collapsed_borders: CollapsedBordersForRow::new(),
            final_collapsed_borders: CollapsedBordersForRow::new(),
            collapsed_border_spacing: CollapsedBorderSpacingForRow::new(),
        }
    }

    /// Assign block-size for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_row_base(&mut self, layout_context: &LayoutContext) {
        if self.block_flow.base.restyle_damage.contains(REFLOW) {
            // Per CSS 2.1 § 17.5.3, find max_y = max(computed `block-size`, minimum block-size of
            // all cells).
            let mut max_block_size = Au(0);
            let thread_id = self.block_flow.base.thread_id;
            let content_box = self.block_flow.base.position
                - self.block_flow.fragment.border_padding
                - self.block_flow.fragment.margin;
            for kid in self.block_flow.base.child_iter_mut() {
                kid.place_float_if_applicable();
                if !flow::base(kid).flags.is_float() {
                    kid.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                         thread_id,
                                                                         content_box);
                }

                {
                    let child_fragment = kid.as_mut_table_cell().fragment();
                    // TODO: Percentage block-size
                    let child_specified_block_size =
                        MaybeAuto::from_style(child_fragment.style().content_block_size(),
                                              Au(0)).specified_or_zero();
                    max_block_size =
                        max(max_block_size,
                            child_specified_block_size +
                            child_fragment.border_padding.block_start_end());
                }
                let child_node = flow::mut_base(kid);
                child_node.position.start.b = Au(0);
                max_block_size = max(max_block_size, child_node.position.size.block);
            }

            let mut block_size = max_block_size;
            // TODO: Percentage block-size
            block_size = match MaybeAuto::from_style(self.block_flow
                                                         .fragment
                                                         .style()
                                                         .content_block_size(),
                                                     Au(0)) {
                MaybeAuto::Auto => block_size,
                MaybeAuto::Specified(value) => max(value, block_size),
            };

            // Assign the block-size of own fragment
            let mut position = self.block_flow.fragment.border_box;
            position.size.block = block_size;
            self.block_flow.fragment.border_box = position;
            self.block_flow.base.position.size.block = block_size;

            // Assign the block-size of kid fragments, which is the same value as own block-size.
            for kid in self.block_flow.base.child_iter_mut() {
                let child_table_cell = kid.as_mut_table_cell();
                {
                    let kid_fragment = child_table_cell.mut_fragment();
                    let mut position = kid_fragment.border_box;
                    position.size.block = block_size;
                    kid_fragment.border_box = position;
                }

                // Assign the child's block size.
                child_table_cell.block_flow.base.position.size.block = block_size;

                // Now we know the cell height, vertical align the cell's children.
                child_table_cell.valign_children();

                // Write in the size of the relative containing block for children. (This
                // information is also needed to handle RTL.)
                child_table_cell.block_flow.base.early_absolute_position_info =
                    EarlyAbsolutePositionInfo {
                        relative_containing_block_size: self.block_flow
                                                            .fragment
                                                            .content_box()
                                                            .size,
                        relative_containing_block_mode: self.block_flow
                                                            .fragment
                                                            .style()
                                                            .writing_mode,
                    };
            }
        }

        self.block_flow.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    pub fn populate_collapsed_border_spacing<'a, I>(
            &mut self,
            collapsed_inline_direction_border_widths_for_table: &[Au],
            collapsed_block_direction_border_widths_for_table: &mut Peekable<I>)
            where I: Iterator<Item=&'a Au> {
        self.collapsed_border_spacing.inline.clear();
        self.collapsed_border_spacing
            .inline
            .extend(collapsed_inline_direction_border_widths_for_table.into_iter().map(|x| *x));

        if let Some(collapsed_block_direction_border_width_for_table) =
                collapsed_block_direction_border_widths_for_table.next() {
            self.collapsed_border_spacing.block_start =
                *collapsed_block_direction_border_width_for_table
        }
        if let Some(collapsed_block_direction_border_width_for_table) =
                collapsed_block_direction_border_widths_for_table.peek() {
            self.collapsed_border_spacing.block_end =
                **collapsed_block_direction_border_width_for_table
        }
    }
}

impl Flow for TableRowFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableRow
    }

    fn as_mut_table_row(&mut self) -> &mut TableRowFlow {
        self
    }

    fn as_table_row(&self) -> &TableRowFlow {
        self
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum inline-sizes. When
    /// called on this context, all child contexts have had their min/pref inline-sizes set. This
    /// function must decide min/pref inline-sizes based on child context inline-sizes and
    /// dimensions of any fragments it is responsible for flowing.
    /// Min/pref inline-sizes set by this function are used in automatic table layout calculation.
    /// The specified column inline-sizes of children cells are used in fixed table layout
    /// calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_row::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        // Bubble up the specified inline-sizes from child table cells.
        let (mut min_inline_size, mut pref_inline_size) = (Au(0), Au(0));
        let collapsing_borders = self.block_flow
                                     .fragment
                                     .style()
                                     .get_inheritedtable()
                                     .border_collapse == border_collapse::T::collapse;
        let row_style = &*self.block_flow.fragment.style;
        self.preliminary_collapsed_borders.reset(
            CollapsedBorder::inline_start(&row_style,
                                          CollapsedBorderProvenance::FromTableRow));

        {
            let children_count = self.block_flow.base.children.len();
            let mut iterator = self.block_flow.base.child_iter_mut().enumerate().peekable();
            while let Some((i, kid)) = iterator.next() {
                assert!(kid.is_table_cell());

                // Collect the specified column inline-size of the cell. This is used in both
                // fixed and automatic table layout calculation.
                let child_specified_inline_size;
                let child_column_span;
                let child_row_span;
                {
                    let child_table_cell = kid.as_mut_table_cell();
                    child_specified_inline_size = child_table_cell.block_flow
                                                                  .fragment
                                                                  .style
                                                                  .content_inline_size();
                    child_column_span = child_table_cell.column_span;
                    child_row_span = child_table_cell.row_span;

                    // Perform border collapse if necessary.
                    if collapsing_borders {
                        perform_inline_direction_border_collapse_for_row(
                            row_style,
                            children_count,
                            i,
                            child_table_cell,
                            &mut iterator,
                            &mut self.preliminary_collapsed_borders)
                    }
                }

                // Collect minimum and preferred inline-sizes of the cell for automatic table layout
                // calculation.
                let child_base = flow::mut_base(kid);
                let child_column_inline_size = ColumnIntrinsicInlineSize {
                    minimum_length: match child_specified_inline_size {
                        LengthOrPercentageOrAuto::Auto |
                        LengthOrPercentageOrAuto::Calc(_) |
                        LengthOrPercentageOrAuto::Percentage(_) => {
                            child_base.intrinsic_inline_sizes.minimum_inline_size
                        }
                        LengthOrPercentageOrAuto::Length(length) => length,
                    },
                    percentage: match child_specified_inline_size {
                        LengthOrPercentageOrAuto::Auto |
                        LengthOrPercentageOrAuto::Calc(_) |
                        LengthOrPercentageOrAuto::Length(_) => 0.0,
                        LengthOrPercentageOrAuto::Percentage(percentage) => percentage.0,
                    },
                    preferred: child_base.intrinsic_inline_sizes.preferred_inline_size,
                    constrained: match child_specified_inline_size {
                        LengthOrPercentageOrAuto::Length(_) => true,
                        LengthOrPercentageOrAuto::Auto |
                        LengthOrPercentageOrAuto::Calc(_) |
                        LengthOrPercentageOrAuto::Percentage(_) => false,
                    },
                };
                min_inline_size = min_inline_size + child_column_inline_size.minimum_length;
                pref_inline_size = pref_inline_size + child_column_inline_size.preferred;
                self.cell_intrinsic_inline_sizes.push(CellIntrinsicInlineSize {
                    column_size: child_column_inline_size,
                    column_span: child_column_span,
                    row_span: child_row_span,
                });
            }
        }

        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size = max(min_inline_size,
                                                                                pref_inline_size);
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table_row::assign_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_row");

        let shared_context = layout_context.shared_context();
        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        // FIXME: In case of border-collapse: collapse, inline_start_content_edge should be
        // border_inline_start.
        let inline_start_content_edge = Au(0);
        let inline_end_content_edge = Au(0);

        let inline_size_computer = InternalTable {
            border_collapse: self.block_flow.fragment.style.get_inheritedtable().border_collapse,
        };
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      shared_context,
                                                      containing_block_inline_size);

        // Spread out the completed inline sizes among columns with spans > 1.
        let num_columns = self.column_computed_inline_sizes.len();
        let mut computed_inline_size_for_cells = Vec::with_capacity(num_columns);
        let mut col = 0;

        for cell_intrinsic_inline_size in &self.cell_intrinsic_inline_sizes {
            // Skip any column occupied by a cell from a previous row.
            while col < self.incoming_rowspan.len() && self.incoming_rowspan[col] != 1 {
                let size = match self.column_computed_inline_sizes.get(col) {
                    Some(column_computed_inline_size) => *column_computed_inline_size,
                    None => ColumnComputedInlineSize { size: Au(0) } // See FIXME below.
                };
                computed_inline_size_for_cells.push(size);
                col += 1;
            }
            // Start with the computed inline size for the first column in the span.
            let mut column_computed_inline_size =
                match self.column_computed_inline_sizes.get(col) {
                    Some(column_computed_inline_size) => *column_computed_inline_size,
                    None => {
                        // We're in fixed layout mode and there are more cells in this row than
                        // columns we know about. According to CSS 2.1 § 17.5.2.1, the behavior is
                        // now undefined. So just use zero.
                        //
                        // FIXME(pcwalton): $10 says this isn't Web compatible.
                        ColumnComputedInlineSize {
                            size: Au(0),
                        }
                    }
                };
            col += 1;

            // Add in computed inline sizes for any extra columns in the span.
            for _ in 1..cell_intrinsic_inline_size.column_span {
                let extra_column_computed_inline_size =
                    match self.column_computed_inline_sizes.get(col) {
                        Some(column_computed_inline_size) => column_computed_inline_size,
                        None => break,
                    };
                column_computed_inline_size.size = column_computed_inline_size.size +
                    extra_column_computed_inline_size.size + self.spacing.horizontal;
                col += 1;
            }

            computed_inline_size_for_cells.push(column_computed_inline_size)
        }

        // Set up border collapse info.
        let border_collapse_info =
            match self.block_flow.fragment.style().get_inheritedtable().border_collapse {
                border_collapse::T::collapse => {
                    Some(BorderCollapseInfoForChildTableCell {
                        collapsed_borders_for_row: &self.final_collapsed_borders,
                        collapsed_border_spacing_for_row: &self.collapsed_border_spacing,
                    })
                }
                border_collapse::T::separate => None,
            };

        // Push those inline sizes down to the cells.
        let spacing = self.spacing;
        let row_writing_mode = self.block_flow.base.writing_mode;
        let table_writing_mode = self.table_writing_mode;
        let incoming_rowspan = &self.incoming_rowspan;
        let mut column_index = 0;

        self.block_flow.propagate_assigned_inline_size_to_children(shared_context,
                                                                   inline_start_content_edge,
                                                                   inline_end_content_edge,
                                                                   containing_block_inline_size,
                                                                   |child_flow,
                                                                    child_index,
                                                                    content_inline_size,
                                                                    _writing_mode,
                                                                    inline_start_margin_edge,
                                                                    inline_end_margin_edge| {
            set_inline_position_of_child_flow(
                child_flow,
                child_index,
                &mut column_index,
                incoming_rowspan,
                row_writing_mode,
                table_writing_mode,
                &computed_inline_size_for_cells,
                &spacing,
                &border_collapse_info,
                content_inline_size,
                inline_start_margin_edge,
                inline_end_margin_edge);
        })
    }

    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_row");
        self.assign_block_size_table_row_base(layout_context);
    }

    fn compute_absolute_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_absolute_position(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        let border_painting_mode = match self.block_flow
                                             .fragment
                                             .style
                                             .get_inheritedtable()
                                             .border_collapse {
            border_collapse::T::separate => BorderPaintingMode::Separate,
            border_collapse::T::collapse => BorderPaintingMode::Hidden,
        };

        self.block_flow.build_display_list_for_block(state, border_painting_mode);
    }

    fn collect_stacking_contexts(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.collect_stacking_contexts(state);
    }

    fn repair_style(&mut self, new_style: &::ServoArc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position)
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        self.block_flow.print_extra_flow_children(print_tree);
    }
}

impl fmt::Debug for TableRowFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowFlow: {:?}", self.block_flow)
    }
}

#[derive(Clone, Debug)]
pub struct CollapsedBordersForRow {
    /// The size of this vector should be equal to the number of cells plus one.
    pub inline: Vec<CollapsedBorder>,
    /// The size of this vector should be equal to the number of cells.
    pub block_start: Vec<CollapsedBorder>,
    /// The size of this vector should be equal to the number of cells.
    pub block_end: Vec<CollapsedBorder>,
}

impl CollapsedBordersForRow {
    pub fn new() -> CollapsedBordersForRow {
        CollapsedBordersForRow {
            inline: Vec::new(),
            block_start: Vec::new(),
            block_end: Vec::new(),
        }
    }

    pub fn reset(&mut self, first_inline_border: CollapsedBorder) {
        self.inline.clear();
        self.inline.push(first_inline_border);
        self.block_start.clear();
        self.block_end.clear()
    }
}

#[derive(Clone, Debug)]
pub struct CollapsedBorderSpacingForRow {
    /// The spacing in between each column.
    inline: Vec<Au>,
    /// The spacing above this row.
    pub block_start: Au,
    /// The spacing below this row.
    block_end: Au,
}

impl CollapsedBorderSpacingForRow {
    fn new() -> CollapsedBorderSpacingForRow {
        CollapsedBorderSpacingForRow {
            inline: Vec::new(),
            block_start: Au(0),
            block_end: Au(0),
        }
    }
}

/// All aspects of a border that can collapse with adjacent borders. See CSS 2.1 § 17.6.2.1.
#[derive(Copy, Clone, Debug)]
pub struct CollapsedBorder {
    /// The style of the border.
    pub style: border_top_style::T,
    /// The width of the border.
    pub width: Au,
    /// The color of the border.
    pub color: Color,
    /// The type of item that this border comes from.
    pub provenance: CollapsedBorderProvenance,
}

impl Serialize for CollapsedBorder {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }
}

/// Where a border style comes from.
///
/// The integer values here correspond to the border conflict resolution rules in CSS 2.1 §
/// 17.6.2.1. Higher values override lower values.
// FIXME(#8586): FromTableRow, FromTableRowGroup, FromTableColumn,
// FromTableColumnGroup are unused
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum CollapsedBorderProvenance {
    FromPreviousTableCell = 6,
    FromNextTableCell = 5,
    FromTableRow = 4,
    FromTableRowGroup = 3,
    FromTableColumn = 2,
    FromTableColumnGroup = 1,
    FromTable = 0,
}

impl CollapsedBorder {
    /// Creates a collapsible border style for no border.
    pub fn new() -> CollapsedBorder {
        CollapsedBorder {
            style: border_top_style::T::none,
            width: Au(0),
            color: Color::transparent(),
            provenance: CollapsedBorderProvenance::FromTable,
        }
    }

    /// Creates a collapsed border from the block-start border described in the given CSS style
    /// object.
    fn top(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
           -> CollapsedBorder {
        CollapsedBorder {
            style: css_style.get_border().border_top_style,
            width: css_style.get_border().border_top_width,
            color: css_style.get_border().border_top_color,
            provenance: provenance,
        }
    }

    /// Creates a collapsed border style from the right border described in the given CSS style
    /// object.
    fn right(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
             -> CollapsedBorder {
        CollapsedBorder {
            style: css_style.get_border().border_right_style,
            width: css_style.get_border().border_right_width,
            color: css_style.get_border().border_right_color,
            provenance: provenance,
        }
    }

    /// Creates a collapsed border style from the bottom border described in the given CSS style
    /// object.
    fn bottom(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
              -> CollapsedBorder {
        CollapsedBorder {
            style: css_style.get_border().border_bottom_style,
            width: css_style.get_border().border_bottom_width,
            color: css_style.get_border().border_bottom_color,
            provenance: provenance,
        }
    }

    /// Creates a collapsed border style from the left border described in the given CSS style
    /// object.
    fn left(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
            -> CollapsedBorder {
        CollapsedBorder {
            style: css_style.get_border().border_left_style,
            width: css_style.get_border().border_left_width,
            color: css_style.get_border().border_left_color,
            provenance: provenance,
        }
    }

    /// Creates a collapsed border style from the given physical side.
    fn from_side(side: PhysicalSide,
                 css_style: &ComputedValues,
                 provenance: CollapsedBorderProvenance)
                 -> CollapsedBorder {
        match side {
            PhysicalSide::Top => CollapsedBorder::top(css_style, provenance),
            PhysicalSide::Right => CollapsedBorder::right(css_style, provenance),
            PhysicalSide::Bottom => CollapsedBorder::bottom(css_style, provenance),
            PhysicalSide::Left => CollapsedBorder::left(css_style, provenance),
        }
    }

    /// Creates a collapsed border style from the inline-start border described in the given CSS
    /// style object.
    pub fn inline_start(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
                        -> CollapsedBorder {
        CollapsedBorder::from_side(css_style.writing_mode.inline_start_physical_side(),
                                   css_style,
                                   provenance)
    }

    /// Creates a collapsed border style from the inline-start border described in the given CSS
    /// style object.
    pub fn inline_end(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
                      -> CollapsedBorder {
        CollapsedBorder::from_side(css_style.writing_mode.inline_end_physical_side(),
                                   css_style,
                                   provenance)
    }

    /// Creates a collapsed border style from the block-start border described in the given CSS
    /// style object.
    pub fn block_start(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
                       -> CollapsedBorder {
        CollapsedBorder::from_side(css_style.writing_mode.block_start_physical_side(),
                                   css_style,
                                   provenance)
    }

    /// Creates a collapsed border style from the block-end border described in the given CSS style
    /// object.
    pub fn block_end(css_style: &ComputedValues, provenance: CollapsedBorderProvenance)
                     -> CollapsedBorder {
        CollapsedBorder::from_side(css_style.writing_mode.block_end_physical_side(),
                                   css_style,
                                   provenance)
    }

    /// If `other` has a higher priority per CSS 2.1 § 17.6.2.1, replaces `self` with it.
    pub fn combine(&mut self, other: &CollapsedBorder) {
        match (self.style, other.style) {
            // Step 1.
            (border_top_style::T::hidden, _) => {}
            (_, border_top_style::T::hidden) => *self = *other,
            // Step 2.
            (border_top_style::T::none, _) => *self = *other,
            (_, border_top_style::T::none) => {}
            // Step 3.
            _ if self.width > other.width => {}
            _ if self.width < other.width => *self = *other,
            (this_style, other_style) if this_style > other_style => {}
            (this_style, other_style) if this_style < other_style => *self = *other,
            // Step 4.
            _ if (self.provenance as i8) >= other.provenance as i8 => {}
            _ => *self = *other,
        }
    }
}

/// Pushes column inline size, incoming rowspan, and border collapse info down to a child.
pub fn propagate_column_inline_sizes_to_child(
        child_flow: &mut Flow,
        table_writing_mode: WritingMode,
        column_computed_inline_sizes: &[ColumnComputedInlineSize],
        border_spacing: &border_spacing::T,
        incoming_rowspan: &mut Vec<u32>) {
    // If the child is a row group or a row, the column inline-size and rowspan info should be copied from its
    // parent.
    //
    // FIXME(pcwalton): This seems inefficient. Reference count it instead?
    match child_flow.class() {
        FlowClass::TableRowGroup => {
            incoming_rowspan.clear();
            let child_table_rowgroup_flow = child_flow.as_mut_table_rowgroup();
            child_table_rowgroup_flow.spacing = *border_spacing;
            for kid in child_table_rowgroup_flow.block_flow.base.child_iter_mut() {
                propagate_column_inline_sizes_to_child(kid,
                                                       table_writing_mode,
                                                       column_computed_inline_sizes,
                                                       border_spacing,
                                                       incoming_rowspan);
            }
        }
        FlowClass::TableRow => {
            let child_table_row_flow = child_flow.as_mut_table_row();
            child_table_row_flow.column_computed_inline_sizes =
                column_computed_inline_sizes.to_vec();
            child_table_row_flow.spacing = *border_spacing;
            child_table_row_flow.table_writing_mode = table_writing_mode;
            child_table_row_flow.incoming_rowspan = incoming_rowspan.clone();

            // Update the incoming rowspan for the next row.
            let mut col = 0;
            for cell in &child_table_row_flow.cell_intrinsic_inline_sizes {
                // Skip any column occupied by a cell from a previous row.
                while col < incoming_rowspan.len() && incoming_rowspan[col] != 1 {
                    if incoming_rowspan[col] > 1 {
                        incoming_rowspan[col] -= 1;
                    }
                    col += 1;
                }
                for _ in 0..cell.column_span {
                    if col < incoming_rowspan.len() && incoming_rowspan[col] > 1 {
                        incoming_rowspan[col] -= 1;
                    }
                    // If this cell spans later rows, record its rowspan.
                    if cell.row_span != 1 {
                        if incoming_rowspan.len() < col + 1 {
                            incoming_rowspan.resize(col + 1, 1);
                        }
                        // HTML § 4.9.11: For rowspan, the value 0 means the cell is to span all
                        // the remaining rows in the rowgroup.
                        if cell.row_span > incoming_rowspan[col] || cell.row_span == 0 {
                            incoming_rowspan[col] = cell.row_span;
                        }
                    }
                    col += 1;
                }
            }
        }
        c => warn!("unexpected flow in table {:?}", c)
    }
}

/// Lay out table cells inline according to the computer column sizes.
fn set_inline_position_of_child_flow(
        child_flow: &mut Flow,
        child_index: usize,
        column_index: &mut usize,
        incoming_rowspan: &[u32],
        row_writing_mode: WritingMode,
        table_writing_mode: WritingMode,
        column_computed_inline_sizes: &[ColumnComputedInlineSize],
        border_spacing: &border_spacing::T,
        border_collapse_info: &Option<BorderCollapseInfoForChildTableCell>,
        parent_content_inline_size: Au,
        inline_start_margin_edge: &mut Au,
        inline_end_margin_edge: &mut Au) {
    if !child_flow.is_table_cell() {
        return
    }

    let reverse_column_order = table_writing_mode.is_bidi_ltr() != row_writing_mode.is_bidi_ltr();

    // Advance past any column occupied by a cell from a previous row.
    while *column_index < incoming_rowspan.len() && incoming_rowspan[*column_index] != 1 {
        let column_inline_size = column_computed_inline_sizes[*column_index].size;
        let border_inline_size = match *border_collapse_info {
            Some(_) => Au(0), // FIXME: Make collapsed borders account for colspan/rowspan.
            None => border_spacing.horizontal,
        };
        if reverse_column_order {
            *inline_end_margin_edge += column_inline_size + border_inline_size;
        } else {
            *inline_start_margin_edge += column_inline_size + border_inline_size;
        }
        *column_index += 1;
    }

    // Handle border collapsing, if necessary.
    let child_table_cell = child_flow.as_mut_table_cell();
    match *border_collapse_info {
        Some(ref border_collapse_info) => {
            // Write in the child's border collapse state.
            child_table_cell.collapsed_borders = CollapsedBordersForCell {
                inline_start_border: border_collapse_info.collapsed_borders_for_row
                                                         .inline
                                                         .get(child_index)
                                                         .map_or(CollapsedBorder::new(), |x| *x),
                inline_end_border: border_collapse_info.collapsed_borders_for_row
                                                       .inline
                                                       .get(child_index + 1)
                                                       .map_or(CollapsedBorder::new(), |x| *x),
                block_start_border: border_collapse_info.collapsed_borders_for_row
                                                        .block_start
                                                        .get(child_index)
                                                        .map_or(CollapsedBorder::new(), |x| *x),
                block_end_border: border_collapse_info.collapsed_borders_for_row
                                                      .block_end
                                                      .get(child_index)
                                                      .map_or(CollapsedBorder::new(), |x| *x),
                inline_start_width: border_collapse_info.collapsed_border_spacing_for_row
                                                        .inline
                                                        .get(child_index)
                                                        .map_or(Au(0), |x| *x),
                inline_end_width: border_collapse_info.collapsed_border_spacing_for_row
                                                      .inline
                                                      .get(child_index + 1)
                                                      .map_or(Au(0), |x| *x),
                block_start_width: border_collapse_info.collapsed_border_spacing_for_row
                                                       .block_start,
                block_end_width: border_collapse_info.collapsed_border_spacing_for_row.block_end,
            };

            // Move over past the collapsed border.
            if reverse_column_order {
                *inline_end_margin_edge += child_table_cell.collapsed_borders.inline_start_width;
            } else {
                *inline_start_margin_edge += child_table_cell.collapsed_borders.inline_start_width;
            }
        }
        None => {
            // Take spacing into account.
            if reverse_column_order {
                *inline_end_margin_edge += border_spacing.horizontal;
            } else {
                *inline_start_margin_edge += border_spacing.horizontal;
            }
        }
    }

    let column_inline_size = column_computed_inline_sizes[*column_index].size;
    *column_index += 1;

    let kid_base = &mut child_table_cell.block_flow.base;
    kid_base.block_container_inline_size = column_inline_size;

    if reverse_column_order {
        // Columns begin from the inline-end edge.
        kid_base.position.start.i =
            parent_content_inline_size - *inline_end_margin_edge - column_inline_size;
        *inline_end_margin_edge += column_inline_size;
    } else {
        // Columns begin from the inline-start edge.
        kid_base.position.start.i = *inline_start_margin_edge;
        *inline_start_margin_edge += column_inline_size;
    }
}

#[derive(Copy, Clone)]
pub struct BorderCollapseInfoForChildTableCell<'a> {
    collapsed_borders_for_row: &'a CollapsedBordersForRow,
    collapsed_border_spacing_for_row: &'a CollapsedBorderSpacingForRow,
}

/// Performs border-collapse in the inline direction for all the cells' inside borders in the
/// inline-direction cells and propagates the outside borders (the far left and right) up to the
/// table row. This is done eagerly here so that at least the inline inside border collapse
/// computations can be parallelized across all the rows of the table.
fn perform_inline_direction_border_collapse_for_row(
        row_style: &ComputedValues,
        children_count: usize,
        child_index: usize,
        child_table_cell: &mut TableCellFlow,
        iterator: &mut Peekable<Enumerate<MutFlowListIterator>>,
        preliminary_collapsed_borders: &mut CollapsedBordersForRow) {
    // In the first cell, combine its border with the one coming from the row.
    if child_index == 0 {
        let first_inline_border = &mut preliminary_collapsed_borders.inline[0];
        first_inline_border.combine(
            &CollapsedBorder::inline_start(&*child_table_cell.block_flow.fragment.style,
                                           CollapsedBorderProvenance::FromNextTableCell));
    }

    let inline_collapsed_border = preliminary_collapsed_borders.inline.push_or_set(
        child_index + 1,
        CollapsedBorder::inline_end(&*child_table_cell.block_flow.fragment.style,
                                    CollapsedBorderProvenance::FromPreviousTableCell));

    if let Some(&(_, ref next_child_flow)) = iterator.peek() {
        let next_child_flow = next_child_flow.as_block();
        inline_collapsed_border.combine(
            &CollapsedBorder::inline_start(&*next_child_flow.fragment.style,
                                           CollapsedBorderProvenance::FromNextTableCell))
    };

    // In the last cell, also take into account the border that may
    // come from the row.
    if child_index + 1 == children_count {
        inline_collapsed_border.combine(
            &CollapsedBorder::inline_end(&row_style,
                                         CollapsedBorderProvenance::FromTableRow));
    }

    let mut block_start_border =
        CollapsedBorder::block_start(&*child_table_cell.block_flow.fragment.style,
                                     CollapsedBorderProvenance::FromNextTableCell);
    block_start_border.combine(
        &CollapsedBorder::block_start(row_style, CollapsedBorderProvenance::FromTableRow));
    preliminary_collapsed_borders.block_start.push_or_set(child_index, block_start_border);
    let mut block_end_border =
        CollapsedBorder::block_end(&*child_table_cell.block_flow.fragment.style,
                                        CollapsedBorderProvenance::FromPreviousTableCell);
    block_end_border.combine(
        &CollapsedBorder::block_end(row_style, CollapsedBorderProvenance::FromTableRow));

    preliminary_collapsed_borders.block_end.push_or_set(child_index, block_end_border);
}
