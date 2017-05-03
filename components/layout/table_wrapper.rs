/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS tables.
//!
//! This follows the "More Precise Definitions of Inline Layout and Table Layout" proposal written
//! by L. David Baron (Mozilla) here:
//!
//!   http://dbaron.org/css/intrinsic/
//!
//! Hereafter this document is referred to as INTRINSIC.

#![deny(unsafe_code)]

use app_units::Au;
use block::{AbsoluteNonReplaced, BlockFlow, FloatNonReplaced, ISizeAndMarginsComputer, ISizeConstraintInput};
use block::{ISizeConstraintSolution, MarginsMayCollapseFlag};
use context::LayoutContext;
use display_list_builder::DisplayListBuildState;
use euclid::Point2D;
use floats::FloatKind;
use flow::{Flow, FlowClass, ImmutableFlowUtils, INLINE_POSITION_IS_STATIC, OpaqueFlow};
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx_traits::print_tree::PrintTree;
use model::MaybeAuto;
use std::cmp::{max, min};
use std::fmt;
use std::ops::Add;
use style::computed_values::{border_collapse, position, table_layout};
use style::context::SharedStyleContext;
use style::logical_geometry::{LogicalRect, LogicalSize};
use style::properties::ServoComputedValues;
use style::values::CSSFloat;
use style::values::computed::LengthOrPercentageOrAuto;
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize};

#[derive(Copy, Clone, Serialize, Debug)]
pub enum TableLayout {
    Fixed,
    Auto
}

/// A table wrapper flow based on a block formatting context.
#[derive(Serialize)]
pub struct TableWrapperFlow {
    pub block_flow: BlockFlow,

    /// Intrinsic column inline sizes according to INTRINSIC § 4.1
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableWrapperFlow {
    pub fn from_fragment(fragment: Fragment) -> TableWrapperFlow {
        TableWrapperFlow::from_fragment_and_float_kind(fragment, None)
    }

    pub fn from_fragment_and_float_kind(fragment: Fragment, float_kind: Option<FloatKind>)
                                        -> TableWrapperFlow {
        let mut block_flow = BlockFlow::from_fragment_and_float_kind(fragment, float_kind);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::T::fixed {
            TableLayout::Fixed
        } else {
            TableLayout::Auto
        };
        TableWrapperFlow {
            block_flow: block_flow,
            column_intrinsic_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    fn border_padding_and_spacing(&mut self) -> (Au, Au) {
        let (mut table_border_padding, mut spacing) = (Au(0), Au(0));
        for kid in self.block_flow.base.child_iter_mut() {
            if kid.is_table() {
                let kid_table = kid.as_table();
                spacing = kid_table.total_horizontal_spacing();
                table_border_padding =
                    kid_table.block_flow.fragment.border_padding.inline_start_end();
                break
            }
        }
        (table_border_padding, spacing)
    }

    // Instructs our first child, which is the table itself, to compute its border and padding.
    //
    // This is a little weird because we're computing border/padding/margins for our child,
    // when normally the child computes it itself. But it has to be this way because the
    // padding will affect where we place the child. This is an odd artifact of the way that
    // tables are separated into table flows and table wrapper flows.
    fn compute_border_and_padding_of_table(&mut self) {
        let available_inline_size = self.block_flow.base.block_container_inline_size;
        let border_collapse = self.block_flow.fragment.style.get_inheritedtable().border_collapse;
        for kid in self.block_flow.base.child_iter_mut() {
            if !kid.is_table() {
                continue
            }

            let kid_table = kid.as_mut_table();
            let kid_block_flow = &mut kid_table.block_flow;
            kid_block_flow.fragment.compute_border_and_padding(available_inline_size,
                                                               border_collapse);
            kid_block_flow.fragment.compute_block_direction_margins(available_inline_size);
            kid_block_flow.fragment.compute_inline_direction_margins(available_inline_size);
            return
        }
    }

    /// Calculates table column sizes for automatic layout per INTRINSIC § 4.3.
    fn calculate_table_column_sizes_for_automatic_layout(
            &mut self,
            intermediate_column_inline_sizes: &mut [IntermediateColumnInlineSize]) {
        let available_inline_size = self.available_inline_size();

        // Compute all the guesses for the column sizes, and sum them.
        let mut total_guess = AutoLayoutCandidateGuess::new();
        let guesses: Vec<AutoLayoutCandidateGuess> =
            self.column_intrinsic_inline_sizes.iter().map(|column_intrinsic_inline_size| {
                let guess = AutoLayoutCandidateGuess::from_column_intrinsic_inline_size(
                    column_intrinsic_inline_size,
                    available_inline_size);
                total_guess = &total_guess + &guess;
                guess
            }).collect();

        // Assign inline sizes.
        let selection = SelectedAutoLayoutCandidateGuess::select(&total_guess,
                                                                 available_inline_size);
        let mut total_used_inline_size = Au(0);
        for (intermediate_column_inline_size, guess) in
                intermediate_column_inline_sizes.iter_mut().zip(guesses.iter()) {
            intermediate_column_inline_size.size = guess.calculate(selection);
            intermediate_column_inline_size.percentage = 0.0;
            total_used_inline_size = total_used_inline_size + intermediate_column_inline_size.size
        }

        // Distribute excess inline-size if necessary per INTRINSIC § 4.4.
        //
        // FIXME(pcwalton, spec): How do I deal with fractional excess?
        let excess_inline_size = available_inline_size - total_used_inline_size;
        if excess_inline_size > Au(0) && selection ==
                SelectedAutoLayoutCandidateGuess::UsePreferredGuessAndDistributeExcessInlineSize {
            let mut info = ExcessInlineSizeDistributionInfo::new();
            for column_intrinsic_inline_size in &self.column_intrinsic_inline_sizes {
                info.update(column_intrinsic_inline_size)
            }

            let mut total_distributed_excess_size = Au(0);
            for (intermediate_column_inline_size, column_intrinsic_inline_size) in
                    intermediate_column_inline_sizes.iter_mut()
                                                    .zip(self.column_intrinsic_inline_sizes
                                                             .iter()) {
                info.distribute_excess_inline_size_to_column(intermediate_column_inline_size,
                                                             column_intrinsic_inline_size,
                                                             excess_inline_size,
                                                             &mut total_distributed_excess_size)
            }
            total_used_inline_size = available_inline_size
        }

        self.set_inline_size(total_used_inline_size)
    }

    fn available_inline_size(&mut self) -> Au {
        let available_inline_size = self.block_flow.fragment.border_box.size.inline;
        let (table_border_padding, spacing) = self.border_padding_and_spacing();

        // FIXME(pcwalton, spec): INTRINSIC § 8 does not properly define how to compute this, but
        // says "the basic idea is the same as the shrink-to-fit width that CSS2.1 defines". So we
        // just use the shrink-to-fit inline size.
        let available_inline_size = match self.block_flow.fragment.style().content_inline_size() {
            LengthOrPercentageOrAuto::Auto => {
                self.block_flow.get_shrink_to_fit_inline_size(available_inline_size) -
                    table_border_padding
            }
            // FIXME(mttr): This fixes #4421 without breaking our current reftests, but I'm not
            // completely sure this is "correct".
            //
            // That said, `available_inline_size` is, as far as I can tell, equal to the table's
            // computed width property (W) and is used from this point forward in a way that seems
            // to correspond with CSS 2.1 § 17.5.2.2 under "Column and caption widths influence the
            // final table width as follows: …"
            _ => available_inline_size,
        };
        available_inline_size - spacing
    }

    fn set_inline_size(&mut self, total_used_inline_size: Au) {
        let (table_border_padding, spacing) = self.border_padding_and_spacing();
        self.block_flow.fragment.border_box.size.inline = total_used_inline_size +
            table_border_padding + spacing;
        self.block_flow.base.position.size.inline = total_used_inline_size +
            table_border_padding + spacing + self.block_flow.fragment.margin.inline_start_end();

        let writing_mode = self.block_flow.base.writing_mode;
        let container_mode = self.block_flow.base.block_container_writing_mode;

        if writing_mode.is_bidi_ltr() != container_mode.is_bidi_ltr() {
            // If our "start" direction is different from our parent flow, then `border_box.start.i`
            // depends on `border_box.size.inline`.
            self.block_flow.fragment.border_box.start.i =
                self.block_flow.base.block_container_inline_size -
                self.block_flow.fragment.margin.inline_end -
                self.block_flow.fragment.border_box.size.inline;
        }
    }

    fn compute_used_inline_size(
            &mut self,
            shared_context: &SharedStyleContext,
            parent_flow_inline_size: Au,
            intermediate_column_inline_sizes: &[IntermediateColumnInlineSize]) {
        let (border_padding, spacing) = self.border_padding_and_spacing();
        let minimum_width_of_all_columns =
            intermediate_column_inline_sizes.iter()
                                            .fold(border_padding + spacing,
                                                  |accumulator, intermediate_column_inline_sizes| {
                accumulator + intermediate_column_inline_sizes.size
            });
        let preferred_width_of_all_columns =
            self.column_intrinsic_inline_sizes.iter()
                                              .fold(border_padding + spacing,
                                                    |accumulator, column_intrinsic_inline_sizes| {
                accumulator + column_intrinsic_inline_sizes.preferred
            });

        // Delegate to the appropriate inline size computer to find the constraint inputs and write
        // the constraint solutions in.
        let border_collapse = self.block_flow.fragment.style.get_inheritedtable().border_collapse;
        if self.block_flow.base.flags.is_float() {
            let inline_size_computer = FloatedTable {
                minimum_width_of_all_columns: minimum_width_of_all_columns,
                preferred_width_of_all_columns: preferred_width_of_all_columns,
                border_collapse: border_collapse,
                table_border_padding: border_padding,
            };
            let input =
                inline_size_computer.compute_inline_size_constraint_inputs(&mut self.block_flow,
                                                                           parent_flow_inline_size,
                                                                           shared_context);

            let solution = inline_size_computer.solve_inline_size_constraints(&mut self.block_flow,
                                                                              &input);
            inline_size_computer.set_inline_size_constraint_solutions(&mut self.block_flow,
                                                                      solution);
            inline_size_computer.set_inline_position_of_flow_if_necessary(&mut self.block_flow,
                                                                          solution);
            return
        }

        if !self.block_flow.base.flags.contains(INLINE_POSITION_IS_STATIC) {
            let inline_size_computer = AbsoluteTable {
                minimum_width_of_all_columns: minimum_width_of_all_columns,
                preferred_width_of_all_columns: preferred_width_of_all_columns,
                border_collapse: border_collapse,
                table_border_padding: border_padding,
            };
            let input =
                inline_size_computer.compute_inline_size_constraint_inputs(&mut self.block_flow,
                                                                           parent_flow_inline_size,
                                                                           shared_context);

            let solution = inline_size_computer.solve_inline_size_constraints(&mut self.block_flow,
                                                                              &input);
            inline_size_computer.set_inline_size_constraint_solutions(&mut self.block_flow,
                                                                      solution);
            inline_size_computer.set_inline_position_of_flow_if_necessary(&mut self.block_flow,
                                                                          solution);
            return
        }

        let inline_size_computer = Table {
            minimum_width_of_all_columns: minimum_width_of_all_columns,
            preferred_width_of_all_columns: preferred_width_of_all_columns,
            border_collapse: border_collapse,
            table_border_padding: border_padding,
        };
        let input =
            inline_size_computer.compute_inline_size_constraint_inputs(&mut self.block_flow,
                                                                       parent_flow_inline_size,
                                                                       shared_context);

        let solution = inline_size_computer.solve_inline_size_constraints(&mut self.block_flow,
                                                                          &input);
        inline_size_computer.set_inline_size_constraint_solutions(&mut self.block_flow, solution);
        inline_size_computer.set_inline_position_of_flow_if_necessary(&mut self.block_flow,
                                                                      solution);
    }
}

impl Flow for TableWrapperFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableWrapper
    }

    fn as_mut_table_wrapper(&mut self) -> &mut TableWrapperFlow {
        self
    }

    fn as_table_wrapper(&self) -> &TableWrapperFlow {
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

    fn bubble_inline_sizes(&mut self) {
        // Get the intrinsic column inline-sizes info from the table flow.
        for kid in self.block_flow.base.child_iter_mut() {
            debug_assert!(kid.is_table_caption() || kid.is_table());
            if kid.is_table() {
                let table = kid.as_table();
                self.column_intrinsic_inline_sizes = table.column_intrinsic_inline_sizes.clone();
            }
        }

        self.block_flow.bubble_inline_sizes();
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow",
               if self.block_flow.base.flags.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        let shared_context = layout_context.shared_context();
        self.block_flow.initialize_container_size_for_root(shared_context);

        let mut intermediate_column_inline_sizes = self.column_intrinsic_inline_sizes
                                                       .iter()
                                                       .map(|column_intrinsic_inline_size| {
            IntermediateColumnInlineSize {
                size: column_intrinsic_inline_size.minimum_length,
                percentage: column_intrinsic_inline_size.percentage,
            }
        }).collect::<Vec<_>>();

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        if self.block_flow.base.flags.is_float() {
            self.block_flow.float.as_mut().unwrap().containing_inline_size =
                containing_block_inline_size;
        }

        // This has to be done before computing our inline size because `compute_used_inline_size`
        // internally consults the border and padding of the table.
        self.compute_border_and_padding_of_table();

        self.compute_used_inline_size(shared_context,
                                      containing_block_inline_size,
                                      &intermediate_column_inline_sizes);

        match self.table_layout {
            TableLayout::Auto => {
                self.calculate_table_column_sizes_for_automatic_layout(
                    &mut intermediate_column_inline_sizes)
            }
            TableLayout::Fixed => {}
        }

        let inline_start_content_edge = self.block_flow.fragment.border_box.start.i;
        let content_inline_size = self.block_flow.fragment.border_box.size.inline;
        let inline_end_content_edge = self.block_flow.fragment.border_padding.inline_end +
                                      self.block_flow.fragment.margin.inline_end;

        // In case of fixed layout, column inline-sizes are calculated in table flow.
        let assigned_column_inline_sizes = match self.table_layout {
            TableLayout::Fixed => None,
            TableLayout::Auto => {
                Some(intermediate_column_inline_sizes.iter().map(|sizes| {
                    ColumnComputedInlineSize {
                        size: sizes.size,
                    }
                }).collect::<Vec<_>>())
            }
        };

        match assigned_column_inline_sizes {
            None => {
                self.block_flow
                    .propagate_assigned_inline_size_to_children(shared_context,
                                                                inline_start_content_edge,
                                                                inline_end_content_edge,
                                                                content_inline_size,
                                                                |_, _, _, _, _, _| {})
            }
            Some(ref assigned_column_inline_sizes) => {
                self.block_flow
                    .propagate_assigned_inline_size_to_children(shared_context,
                                                                inline_start_content_edge,
                                                                inline_end_content_edge,
                                                                content_inline_size,
                                                                |child_flow, _, _, _, _, _| {
                    if child_flow.class() == FlowClass::Table {
                        child_flow.as_mut_table().column_computed_inline_sizes =
                            assigned_column_inline_sizes.to_vec();
                    }
                })
            }
        }

    }

    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_wrapper");
        let remaining = self.block_flow.assign_block_size_block_base(
            layout_context,
            None,
            MarginsMayCollapseFlag::MarginsMayNotCollapse);
        debug_assert!(remaining.is_none());
    }

    fn compute_absolute_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_absolute_position(layout_context)
    }

    fn place_float_if_applicable<'a>(&mut self) {
        self.block_flow.place_float_if_applicable()
    }

    fn assign_block_size_for_inorder_child_if_necessary(&mut self,
                                                        layout_context: &LayoutContext,
                                                        parent_thread_id: u8,
                                                        content_box: LogicalRect<Au>)
                                                        -> bool {
        self.block_flow.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                         parent_thread_id,
                                                                         content_box)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.build_display_list(state);
    }

    fn collect_stacking_contexts(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.collect_stacking_contexts(state);
    }

    fn repair_style(&mut self, new_style: &::StyleArc<ServoComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
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

    fn positioning(&self) -> position::T {
        self.block_flow.positioning()
    }
}

impl fmt::Debug for TableWrapperFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.block_flow.base.flags.is_float() {
            write!(f, "TableWrapperFlow(Float): {:?}", self.block_flow)
        } else {
            write!(f, "TableWrapperFlow: {:?}", self.block_flow)
        }
    }
}

/// The layout "guesses" defined in INTRINSIC § 4.3.
struct AutoLayoutCandidateGuess {
    /// The column inline-size assignment where each column is assigned its intrinsic minimum
    /// inline-size.
    minimum_guess: Au,

    /// The column inline-size assignment where:
    ///   * A column with an intrinsic percentage inline-size greater than 0% is assigned the
    ///     larger of:
    ///     - Its intrinsic percentage inline-size times the assignable inline-size;
    ///     - Its intrinsic minimum inline-size;
    ///   * Other columns receive their intrinsic minimum inline-size.
    minimum_percentage_guess: Au,

    /// The column inline-size assignment where:
    ///   * Each column with an intrinsic percentage inline-size greater than 0% is assigned the
    ///     larger of:
    ///     - Its intrinsic percentage inline-size times the assignable inline-size;
    ///     - Its intrinsic minimum inline-size;
    ///   * Any other column that is constrained is assigned its intrinsic preferred inline-size;
    ///   * Other columns are assigned their intrinsic minimum inline-size.
    minimum_specified_guess: Au,

    /// The column inline-size assignment where:
    ///   * Each column with an intrinsic percentage inline-size greater than 0% is assigned the
    ///     larger of:
    ///     - Its intrinsic percentage inline-size times the assignable inline-size;
    ///     - Its intrinsic minimum inline-size;
    ///   * Other columns are assigned their intrinsic preferred inline-size.
    preferred_guess: Au,
}

impl AutoLayoutCandidateGuess {
    /// Creates a guess with all elements initialized to zero.
    fn new() -> AutoLayoutCandidateGuess {
        AutoLayoutCandidateGuess {
            minimum_guess: Au(0),
            minimum_percentage_guess: Au(0),
            minimum_specified_guess: Au(0),
            preferred_guess: Au(0),
        }
    }

    /// Fills in the inline-size guesses for this column per INTRINSIC § 4.3.
    fn from_column_intrinsic_inline_size(column_intrinsic_inline_size: &ColumnIntrinsicInlineSize,
                                         assignable_inline_size: Au)
                                         -> AutoLayoutCandidateGuess {
        let minimum_percentage_guess =
            max(assignable_inline_size.scale_by(column_intrinsic_inline_size.percentage),
                column_intrinsic_inline_size.minimum_length);
        AutoLayoutCandidateGuess {
            minimum_guess: column_intrinsic_inline_size.minimum_length,
            minimum_percentage_guess: minimum_percentage_guess,
            // FIXME(pcwalton): We need the notion of *constrainedness* per INTRINSIC § 4 to
            // implement this one correctly.
            minimum_specified_guess: if column_intrinsic_inline_size.percentage > 0.0 {
                minimum_percentage_guess
            } else if column_intrinsic_inline_size.constrained {
                column_intrinsic_inline_size.preferred
            } else {
                column_intrinsic_inline_size.minimum_length
            },
            preferred_guess: if column_intrinsic_inline_size.percentage > 0.0 {
                minimum_percentage_guess
            } else {
                column_intrinsic_inline_size.preferred
            },
        }
    }

    /// Calculates the inline-size, interpolating appropriately based on the value of `selection`.
    ///
    /// This does *not* distribute excess inline-size. That must be done later if necessary.
    fn calculate(&self, selection: SelectedAutoLayoutCandidateGuess) -> Au {
        match selection {
            SelectedAutoLayoutCandidateGuess::UseMinimumGuess => self.minimum_guess,
            SelectedAutoLayoutCandidateGuess::
                    InterpolateBetweenMinimumGuessAndMinimumPercentageGuess(weight) => {
                interp(self.minimum_guess, self.minimum_percentage_guess, weight)
            }
            SelectedAutoLayoutCandidateGuess::
                    InterpolateBetweenMinimumPercentageGuessAndMinimumSpecifiedGuess(weight) => {
                interp(self.minimum_percentage_guess, self.minimum_specified_guess, weight)
            }
            SelectedAutoLayoutCandidateGuess::
                    InterpolateBetweenMinimumSpecifiedGuessAndPreferredGuess(weight) => {
                interp(self.minimum_specified_guess, self.preferred_guess, weight)
            }
            SelectedAutoLayoutCandidateGuess::UsePreferredGuessAndDistributeExcessInlineSize => {
                self.preferred_guess
            }
        }
    }
}

impl<'a> Add for &'a AutoLayoutCandidateGuess {
    type Output = AutoLayoutCandidateGuess;
    #[inline]
    fn add(self, other: &AutoLayoutCandidateGuess) -> AutoLayoutCandidateGuess {
        AutoLayoutCandidateGuess {
            minimum_guess: self.minimum_guess + other.minimum_guess,
            minimum_percentage_guess:
                self.minimum_percentage_guess + other.minimum_percentage_guess,
            minimum_specified_guess: self.minimum_specified_guess + other.minimum_specified_guess,
            preferred_guess: self.preferred_guess + other.preferred_guess,
        }
    }
}

/// The `CSSFloat` member specifies the weight of the smaller of the two guesses, on a scale from
/// 0.0 to 1.0.
#[derive(Copy, Clone, PartialEq, Debug)]
enum SelectedAutoLayoutCandidateGuess {
    UseMinimumGuess,
    InterpolateBetweenMinimumGuessAndMinimumPercentageGuess(CSSFloat),
    InterpolateBetweenMinimumPercentageGuessAndMinimumSpecifiedGuess(CSSFloat),
    InterpolateBetweenMinimumSpecifiedGuessAndPreferredGuess(CSSFloat),
    UsePreferredGuessAndDistributeExcessInlineSize,
}

impl SelectedAutoLayoutCandidateGuess {
    /// See INTRINSIC § 4.3.
    ///
    /// FIXME(pcwalton, INTRINSIC spec): INTRINSIC doesn't specify whether these are exclusive or
    /// inclusive ranges.
    fn select(guess: &AutoLayoutCandidateGuess, assignable_inline_size: Au)
              -> SelectedAutoLayoutCandidateGuess {
        if assignable_inline_size < guess.minimum_guess {
            SelectedAutoLayoutCandidateGuess::UseMinimumGuess
        } else if assignable_inline_size < guess.minimum_percentage_guess {
            let weight = weight(guess.minimum_guess,
                                assignable_inline_size,
                                guess.minimum_percentage_guess);
            SelectedAutoLayoutCandidateGuess::InterpolateBetweenMinimumGuessAndMinimumPercentageGuess(weight)
        } else if assignable_inline_size < guess.minimum_specified_guess {
            let weight = weight(guess.minimum_percentage_guess,
                                assignable_inline_size,
                                guess.minimum_specified_guess);
            SelectedAutoLayoutCandidateGuess::InterpolateBetweenMinimumPercentageGuessAndMinimumSpecifiedGuess(weight)
        } else if assignable_inline_size < guess.preferred_guess {
            let weight = weight(guess.minimum_specified_guess,
                                assignable_inline_size,
                                guess.preferred_guess);
            SelectedAutoLayoutCandidateGuess::InterpolateBetweenMinimumSpecifiedGuessAndPreferredGuess(weight)
        } else {
            SelectedAutoLayoutCandidateGuess::UsePreferredGuessAndDistributeExcessInlineSize
        }
    }
}

/// Computes the weight needed to linearly interpolate `middle` between two guesses `low` and
/// `high` as specified by INTRINSIC § 4.3.
fn weight(low: Au, middle: Au, high: Au) -> CSSFloat {
    (middle - low).to_f32_px() / (high - low).to_f32_px()
}

/// Linearly interpolates between two guesses, as specified by INTRINSIC § 4.3.
fn interp(low: Au, high: Au, weight: CSSFloat) -> Au {
    low + (high - low).scale_by(weight)
}

struct ExcessInlineSizeDistributionInfo {
    preferred_inline_size_of_nonconstrained_columns_with_no_percentage: Au,
    count_of_nonconstrained_columns_with_no_percentage: u32,
    preferred_inline_size_of_constrained_columns_with_no_percentage: Au,
    total_percentage: CSSFloat,
    column_count: u32,
}

impl ExcessInlineSizeDistributionInfo {
    fn new() -> ExcessInlineSizeDistributionInfo {
        ExcessInlineSizeDistributionInfo {
            preferred_inline_size_of_nonconstrained_columns_with_no_percentage: Au(0),
            count_of_nonconstrained_columns_with_no_percentage: 0,
            preferred_inline_size_of_constrained_columns_with_no_percentage: Au(0),
            total_percentage: 0.0,
            column_count: 0,
        }
    }

    fn update(&mut self, column_intrinsic_inline_size: &ColumnIntrinsicInlineSize) {
        if !column_intrinsic_inline_size.constrained &&
                column_intrinsic_inline_size.percentage == 0.0 {
            self.preferred_inline_size_of_nonconstrained_columns_with_no_percentage =
                self.preferred_inline_size_of_nonconstrained_columns_with_no_percentage +
                column_intrinsic_inline_size.preferred;
            self.count_of_nonconstrained_columns_with_no_percentage += 1
        }
        if column_intrinsic_inline_size.constrained &&
                column_intrinsic_inline_size.percentage == 0.0 {
            self.preferred_inline_size_of_constrained_columns_with_no_percentage =
                self.preferred_inline_size_of_constrained_columns_with_no_percentage +
                column_intrinsic_inline_size.preferred
        }
        self.total_percentage += column_intrinsic_inline_size.percentage;
        self.column_count += 1
    }

    /// Based on the information here, distributes excess inline-size to the given column per
    /// INTRINSIC § 4.4.
    ///
    /// `#[inline]` so the compiler will hoist out the branch, which is loop-invariant.
    #[inline]
    fn distribute_excess_inline_size_to_column(
            &self,
            intermediate_column_inline_size: &mut IntermediateColumnInlineSize,
            column_intrinsic_inline_size: &ColumnIntrinsicInlineSize,
            excess_inline_size: Au,
            total_distributed_excess_size: &mut Au) {
        let proportion =
            if self.preferred_inline_size_of_nonconstrained_columns_with_no_percentage > Au(0) {
                // FIXME(spec, pcwalton): Gecko and WebKit do *something* here when there are
                // nonconstrained columns with no percentage *and* no preferred width. What do they
                // do?
                if !column_intrinsic_inline_size.constrained &&
                        column_intrinsic_inline_size.percentage == 0.0 {
                    column_intrinsic_inline_size.preferred.to_f32_px() /
                        self.preferred_inline_size_of_nonconstrained_columns_with_no_percentage
                            .to_f32_px()
                } else {
                    0.0
                }
            } else if self.count_of_nonconstrained_columns_with_no_percentage > 0 {
                1.0 / (self.count_of_nonconstrained_columns_with_no_percentage as CSSFloat)
            } else if self.preferred_inline_size_of_constrained_columns_with_no_percentage >
                    Au(0) {
                column_intrinsic_inline_size.preferred.to_f32_px() /
                    self.preferred_inline_size_of_constrained_columns_with_no_percentage.to_f32_px()
            } else if self.total_percentage > 0.0 {
                column_intrinsic_inline_size.percentage / self.total_percentage
            } else {
                1.0 / (self.column_count as CSSFloat)
            };

        // The `min` here has the effect of throwing away fractional excess at the end of the
        // table.
        let amount_to_distribute = min(excess_inline_size.scale_by(proportion),
                                       excess_inline_size - *total_distributed_excess_size);
        *total_distributed_excess_size = *total_distributed_excess_size + amount_to_distribute;
        intermediate_column_inline_size.size = intermediate_column_inline_size.size +
            amount_to_distribute
    }
}

/// An intermediate column size assignment.
struct IntermediateColumnInlineSize {
    size: Au,
    percentage: f32,
}

/// Returns the computed inline size of the table wrapper represented by `block`.
///
/// `table_border_padding` is the sum of the sizes of all border and padding in the inline
/// direction of the table contained within this table wrapper.
fn initial_computed_inline_size(block: &mut BlockFlow,
                                containing_block_inline_size: Au,
                                minimum_width_of_all_columns: Au,
                                preferred_width_of_all_columns: Au,
                                table_border_padding: Au)
                                -> MaybeAuto {
    let inline_size_from_style = MaybeAuto::from_style(block.fragment.style.content_inline_size(),
                                                       containing_block_inline_size);
    match inline_size_from_style {
        MaybeAuto::Auto => {
            if preferred_width_of_all_columns + table_border_padding <= containing_block_inline_size {
                MaybeAuto::Specified(preferred_width_of_all_columns + table_border_padding)
            } else if minimum_width_of_all_columns > containing_block_inline_size {
                MaybeAuto::Specified(minimum_width_of_all_columns)
            } else {
                MaybeAuto::Auto
            }
        }
        MaybeAuto::Specified(inline_size_from_style) => {
            MaybeAuto::Specified(max(inline_size_from_style - table_border_padding,
                                     minimum_width_of_all_columns))
        }
    }
}

struct Table {
    minimum_width_of_all_columns: Au,
    preferred_width_of_all_columns: Au,
    border_collapse: border_collapse::T,
    table_border_padding: Au,
}

impl ISizeAndMarginsComputer for Table {
    fn compute_border_and_padding(&self, block: &mut BlockFlow, containing_block_inline_size: Au) {
        block.fragment.compute_border_and_padding(containing_block_inline_size,
                                                  self.border_collapse)
    }

    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let containing_block_inline_size =
            self.containing_block_inline_size(block, parent_flow_inline_size, shared_context);
        initial_computed_inline_size(block,
                                     containing_block_inline_size,
                                     self.minimum_width_of_all_columns,
                                     self.preferred_width_of_all_columns,
                                     self.table_border_padding)
    }

    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        self.solve_block_inline_size_constraints(block, input)
    }
}

struct FloatedTable {
    minimum_width_of_all_columns: Au,
    preferred_width_of_all_columns: Au,
    border_collapse: border_collapse::T,
    table_border_padding: Au,
}

impl ISizeAndMarginsComputer for FloatedTable {
    fn compute_border_and_padding(&self, block: &mut BlockFlow, containing_block_inline_size: Au) {
        block.fragment.compute_border_and_padding(containing_block_inline_size,
                                                  self.border_collapse)
    }

    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let containing_block_inline_size =
            self.containing_block_inline_size(block,
                                              parent_flow_inline_size,
                                              shared_context);
        initial_computed_inline_size(block,
                                     containing_block_inline_size,
                                     self.minimum_width_of_all_columns,
                                     self.preferred_width_of_all_columns,
                                     self.table_border_padding)
    }

    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        FloatNonReplaced.solve_inline_size_constraints(block, input)
    }
}

struct AbsoluteTable {
    minimum_width_of_all_columns: Au,
    preferred_width_of_all_columns: Au,
    border_collapse: border_collapse::T,
    table_border_padding: Au,
}

impl ISizeAndMarginsComputer for AbsoluteTable {
    fn compute_border_and_padding(&self, block: &mut BlockFlow, containing_block_inline_size: Au) {
        block.fragment.compute_border_and_padding(containing_block_inline_size,
                                                  self.border_collapse)
    }

    fn initial_computed_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> MaybeAuto {
        let containing_block_inline_size =
            self.containing_block_inline_size(block,
                                              parent_flow_inline_size,
                                              shared_context);
        initial_computed_inline_size(block,
                                     containing_block_inline_size,
                                     self.minimum_width_of_all_columns,
                                     self.preferred_width_of_all_columns,
                                     self.table_border_padding)
    }

    fn containing_block_inline_size(&self,
                                    block: &mut BlockFlow,
                                    parent_flow_inline_size: Au,
                                    shared_context: &SharedStyleContext)
                                    -> Au {
        AbsoluteNonReplaced.containing_block_inline_size(block,
                                                         parent_flow_inline_size,
                                                         shared_context)
    }

    fn solve_inline_size_constraints(&self,
                                     block: &mut BlockFlow,
                                     input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        AbsoluteNonReplaced.solve_inline_size_constraints(block, input)
    }

    fn set_inline_position_of_flow_if_necessary(&self,
                                                block: &mut BlockFlow,
                                                solution: ISizeConstraintSolution) {
        AbsoluteNonReplaced.set_inline_position_of_flow_if_necessary(block, solution);
    }

}
