/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Creates flows and fragments from a DOM tree via a bottom-up, incremental traversal of the DOM.
//!
//! Each step of the traversal considers the node and existing flow, if there is one. If a node is
//! not dirty and an existing flow exists, then the traversal reuses that flow. Otherwise, it
//! proceeds to construct either a flow or a `ConstructionItem`. A construction item is a piece of
//! intermediate data that goes with a DOM node and hasn't found its "home" yet-maybe it's a box,
//! maybe it's an absolute or fixed position thing that hasn't found its containing block yet.
//! Construction items bubble up the tree from children to parents until they find their homes.

#![deny(unsafe_code)]

use block::BlockFlow;
use context::LayoutContext;
use data::{HAS_NEWLY_CONSTRUCTED_FLOW, LayoutDataWrapper};
use flex::FlexFlow;
use floats::FloatKind;
use flow::{MutableFlowUtils, MutableOwnedFlowUtils};
use flow::{self, AbsoluteDescendants, Flow, ImmutableFlowUtils, IS_ABSOLUTELY_POSITIONED};
use flow_ref::{self, FlowRef};
use fragment::{CanvasFragmentInfo, ImageFragmentInfo, InlineAbsoluteFragmentInfo};
use fragment::{Fragment, GeneratedContentInfo, IframeFragmentInfo};
use fragment::{InlineAbsoluteHypotheticalFragmentInfo, TableColumnFragmentInfo};
use fragment::{InlineBlockFragmentInfo, SpecificFragmentInfo, UnscannedTextFragmentInfo};
use fragment::{WhitespaceStrippingResult};
use incremental::{RECONSTRUCT_FLOW, RestyleDamage};
use inline::{InlineFlow, InlineFragmentNodeInfo};
use list_item::{ListItemFlow, ListStyleTypeContent};
use multicol::MulticolFlow;
use parallel;
use table::TableFlow;
use table_caption::TableCaptionFlow;
use table_cell::TableCellFlow;
use table_colgroup::TableColGroupFlow;
use table_row::TableRowFlow;
use table_rowgroup::TableRowGroupFlow;
use table_wrapper::TableWrapperFlow;
use text::TextRunScanner;
use traversal::PostorderNodeMutTraversal;
use wrapper::{PseudoElementType, ThreadSafeLayoutNode};

use gfx::display_list::OpaqueNode;
use script::dom::characterdata::CharacterDataTypeId;
use script::dom::element::ElementTypeId;
use script::dom::htmlelement::HTMLElementTypeId;
use script::dom::htmlobjectelement::is_image_data;
use script::dom::node::NodeTypeId;
use std::borrow::ToOwned;
use std::collections::LinkedList;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use style::computed_values::content::ContentItem;
use style::computed_values::{caption_side, display, empty_cells, float, list_style_position};
use style::computed_values::{position};
use style::properties::{self, ComputedValues};
use url::Url;
use util::linked_list;
use util::opts;

/// The results of flow construction for a DOM node.
#[derive(Clone)]
pub enum ConstructionResult {
    /// This node contributes nothing at all (`display: none`). Alternately, this is what newly
    /// created nodes have their `ConstructionResult` set to.
    None,

    /// This node contributed a flow at the proper position in the tree.
    /// Nothing more needs to be done for this node. It has bubbled up fixed
    /// and absolute descendant flows that have a containing block above it.
    Flow(FlowRef, AbsoluteDescendants),

    /// This node contributed some object or objects that will be needed to construct a proper flow
    /// later up the tree, but these objects have not yet found their home.
    ConstructionItem(ConstructionItem),
}

impl ConstructionResult {
    pub fn swap_out(&mut self) -> ConstructionResult {
        if opts::get().nonincremental_layout {
            return mem::replace(self, ConstructionResult::None)
        }

        // FIXME(pcwalton): Stop doing this with inline fragments. Cloning fragments is very
        // inefficient!
        (*self).clone()
    }

    pub fn debug_id(&self) -> usize {
        match self {
            &ConstructionResult::None => 0,
            &ConstructionResult::ConstructionItem(_) => 0,
            &ConstructionResult::Flow(ref flow_ref, _) => flow::base(&**flow_ref).debug_id(),
        }
    }
}

/// Represents the output of flow construction for a DOM node that has not yet resulted in a
/// complete flow. Construction items bubble up the tree until they find a `Flow` to be attached
/// to.
#[derive(Clone)]
pub enum ConstructionItem {
    /// Inline fragments and associated {ib} splits that have not yet found flows.
    InlineFragments(InlineFragmentsConstructionResult),
    /// Potentially ignorable whitespace.
    Whitespace(OpaqueNode, PseudoElementType<()>, Arc<ComputedValues>, RestyleDamage),
    /// TableColumn Fragment
    TableColumnFragment(Fragment),
}

/// Represents inline fragments and {ib} splits that are bubbling up from an inline.
#[derive(Clone)]
pub struct InlineFragmentsConstructionResult {
    /// Any {ib} splits that we're bubbling up.
    pub splits: LinkedList<InlineBlockSplit>,

    /// Any fragments that succeed the {ib} splits.
    pub fragments: IntermediateInlineFragments,
}

/// Represents an {ib} split that has not yet found the containing block that it belongs to. This
/// is somewhat tricky. An example may be helpful. For this DOM fragment:
///
/// ```html
///     <span>
///     A
///     <div>B</div>
///     C
///     </span>
/// ```
///
/// The resulting `ConstructionItem` for the outer `span` will be:
///
/// ```ignore
///     ConstructionItem::InlineFragments(Some(~[
///         InlineBlockSplit {
///             predecessor_fragments: ~[
///                 A
///             ],
///             block: ~BlockFlow {
///                 B
///             },
///         }),~[
///             C
///         ])
/// ```
#[derive(Clone)]
pub struct InlineBlockSplit {
    /// The inline fragments that precede the flow.
    pub predecessors: IntermediateInlineFragments,

    /// The flow that caused this {ib} split.
    pub flow: FlowRef,
}

/// Holds inline fragments and absolute descendants.
#[derive(Clone)]
pub struct IntermediateInlineFragments {
    /// The list of fragments.
    pub fragments: LinkedList<Fragment>,

    /// The list of absolute descendants of those inline fragments.
    pub absolute_descendants: AbsoluteDescendants,
}

impl IntermediateInlineFragments {
    fn new() -> IntermediateInlineFragments {
        IntermediateInlineFragments {
            fragments: LinkedList::new(),
            absolute_descendants: AbsoluteDescendants::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.fragments.is_empty() && self.absolute_descendants.is_empty()
    }

    fn push_all(&mut self, mut other: IntermediateInlineFragments) {
        self.fragments.append(&mut other.fragments);
        self.absolute_descendants.push_descendants(other.absolute_descendants);
    }
}

/// Holds inline fragments that we're gathering for children of an inline node.
struct InlineFragmentsAccumulator {
    /// The list of fragments.
    fragments: IntermediateInlineFragments,

    /// Whether we've created a range to enclose all the fragments. This will be Some() if the
    /// outer node is an inline and None otherwise.
    enclosing_node: Option<InlineFragmentNodeInfo>,

    /// Restyle damage to use for fragments created in this node.
    restyle_damage: RestyleDamage,

    /// Bidi control characters to insert before and after these fragments.
    bidi_control_chars: Option<(&'static str, &'static str)>,
}

impl InlineFragmentsAccumulator {
    fn new() -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: IntermediateInlineFragments::new(),
            enclosing_node: None,
            bidi_control_chars: None,
            restyle_damage: RestyleDamage::empty(),
        }
    }

    fn from_inline_node(node: &ThreadSafeLayoutNode) -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: IntermediateInlineFragments::new(),
            enclosing_node: Some(InlineFragmentNodeInfo {
                address: node.opaque(),
                pseudo: node.get_pseudo_element_type().strip(),
                style: node.style().clone(),
            }),
            bidi_control_chars: None,
            restyle_damage: node.restyle_damage(),
        }
    }

    fn from_inline_node_and_style(node: &ThreadSafeLayoutNode, style: Arc<ComputedValues>)
                                  -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: IntermediateInlineFragments::new(),
            enclosing_node: Some(InlineFragmentNodeInfo {
                address: node.opaque(),
                pseudo: node.get_pseudo_element_type().strip(),
                style: style,
            }),
            bidi_control_chars: None,
            restyle_damage: node.restyle_damage(),
        }
    }

    fn push(&mut self, fragment: Fragment) {
        self.fragments.fragments.push_back(fragment)
    }

    fn push_all(&mut self, mut fragments: IntermediateInlineFragments) {
        self.fragments.fragments.append(&mut fragments.fragments);
        self.fragments.absolute_descendants.push_descendants(fragments.absolute_descendants);
    }

    fn to_intermediate_inline_fragments(self) -> IntermediateInlineFragments {
        let InlineFragmentsAccumulator {
            mut fragments,
            enclosing_node,
            bidi_control_chars,
            restyle_damage,
        } = self;
        if let Some(enclosing_node) = enclosing_node {
            let frag_len = fragments.fragments.len();
            for (idx, frag) in fragments.fragments.iter_mut().enumerate() {

                // frag is first inline fragment in the inline node
                let is_first = idx == 0;
                // frag is the last inline fragment in the inline node
                let is_last = idx == frag_len - 1;

                frag.add_inline_context_style(enclosing_node.clone(), is_first, is_last);
            }

            // Control characters are later discarded in transform_text, so they don't affect the
            // is_first/is_last styles above.
            if let Some((start, end)) = bidi_control_chars {
                fragments.fragments.push_front(
                    control_chars_to_fragment(&enclosing_node, start, restyle_damage));
                fragments.fragments.push_back(
                    control_chars_to_fragment(&enclosing_node, end, restyle_damage));
            }
        }
        fragments
    }
}

/// An object that knows how to create flows.
pub struct FlowConstructor<'a> {
    /// The layout context.
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> FlowConstructor<'a> {
    /// Creates a new flow constructor.
    pub fn new<'b>(layout_context: &'b LayoutContext<'b>)
                   -> FlowConstructor<'b> {
        FlowConstructor {
            layout_context: layout_context,
        }
    }

    #[inline]
    fn set_flow_construction_result(&self, node: &ThreadSafeLayoutNode, result: ConstructionResult) {
        match result {
            ConstructionResult::None => {
                let mut layout_data_ref = node.mutate_layout_data();
                let layout_data = layout_data_ref.as_mut().expect("no layout data");
                layout_data.remove_compositor_layers(self.layout_context.shared.constellation_chan.clone());
            }
            _ => {}
        }

        node.set_flow_construction_result(result);
    }

    /// Builds the fragment for the given block or subclass thereof.
    fn build_fragment_for_block(&mut self, node: &ThreadSafeLayoutNode) -> Fragment {
        let specific_fragment_info = match node.type_id() {
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLIFrameElement))) => {
                SpecificFragmentInfo::Iframe(box IframeFragmentInfo::new(node))
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLImageElement))) => {
                let image_info = box ImageFragmentInfo::new(node,
                                                            node.image_url(),
                                                            &self.layout_context);
                SpecificFragmentInfo::Image(image_info)
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLObjectElement))) => {
                let image_info = box ImageFragmentInfo::new(node,
                                                            node.object_data(),
                                                            &self.layout_context);
                SpecificFragmentInfo::Image(image_info)
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTableElement))) => {
                SpecificFragmentInfo::TableWrapper
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTableColElement))) => {
                SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node))
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTableCellElement(_)))) => {
                SpecificFragmentInfo::TableCell
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTableRowElement))) |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTableSectionElement))) => {
                SpecificFragmentInfo::TableRow
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLCanvasElement))) => {
                SpecificFragmentInfo::Canvas(box CanvasFragmentInfo::new(node))
            }
            _ => {
                // This includes pseudo-elements.
                SpecificFragmentInfo::Generic
            }
        };

        Fragment::new(node, specific_fragment_info)
    }

    /// Generates anonymous table objects per CSS 2.1 § 17.2.1.
    fn generate_anonymous_table_flows_if_necessary(&mut self,
                                                   flow: &mut FlowRef,
                                                   child: &mut FlowRef,
                                                   child_node: &ThreadSafeLayoutNode) {
        if !flow.is_block_flow() {
            return
        }

        if child.is_table_cell() {
            let mut style = child_node.style().clone();
            properties::modify_style_for_anonymous_table_object(&mut style, display::T::table_row);
            let fragment = Fragment::from_opaque_node_and_style(child_node.opaque(),
                                                                PseudoElementType::Normal,
                                                                style,
                                                                child_node.restyle_damage(),
                                                                SpecificFragmentInfo::TableRow);
            let mut new_child: FlowRef = Arc::new(TableRowFlow::from_fragment(fragment));
            new_child.add_new_child(child.clone());
            child.finish();
            *child = new_child
        }
        if child.is_table_row() || child.is_table_rowgroup() {
            let mut style = child_node.style().clone();
            properties::modify_style_for_anonymous_table_object(&mut style, display::T::table);
            let fragment = Fragment::from_opaque_node_and_style(child_node.opaque(),
                                                                PseudoElementType::Normal,
                                                                style,
                                                                child_node.restyle_damage(),
                                                                SpecificFragmentInfo::Table);
            let mut new_child: FlowRef = Arc::new(TableFlow::from_fragment(fragment));
            new_child.add_new_child(child.clone());
            child.finish();
            *child = new_child
        }
        if child.is_table() {
            let mut style = child_node.style().clone();
            properties::modify_style_for_anonymous_table_object(&mut style, display::T::table);
            let fragment =
                Fragment::from_opaque_node_and_style(child_node.opaque(),
                                                     PseudoElementType::Normal,
                                                     style,
                                                     child_node.restyle_damage(),
                                                     SpecificFragmentInfo::TableWrapper);
            let mut new_child: FlowRef = Arc::new(TableWrapperFlow::from_fragment(fragment, None));
            new_child.add_new_child(child.clone());
            child.finish();
            *child = new_child
        }
    }

    /// Creates an inline flow from a set of inline fragments, then adds it as a child of the given
    /// flow or pushes it onto the given flow list.
    ///
    /// `#[inline(always)]` because this is performance critical and LLVM will not inline it
    /// otherwise.
    #[inline(always)]
    fn flush_inline_fragments_to_flow_or_list(&mut self,
                                              fragment_accumulator: InlineFragmentsAccumulator,
                                              flow: &mut FlowRef,
                                              flow_list: &mut Vec<FlowRef>,
                                              absolute_descendants: &mut AbsoluteDescendants,
                                              node: &ThreadSafeLayoutNode) {
        let mut fragments = fragment_accumulator.to_intermediate_inline_fragments();
        if fragments.is_empty() {
            return
        };

        strip_ignorable_whitespace_from_start(&mut fragments.fragments);
        strip_ignorable_whitespace_from_end(&mut fragments.fragments);
        if fragments.fragments.is_empty() {
            absolute_descendants.push_descendants(fragments.absolute_descendants);
            return
        }

        // Build a list of all the inline-block fragments before fragments is moved.
        let mut inline_block_flows = vec!();
        for fragment in &fragments.fragments {
            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref info) => {
                    inline_block_flows.push(info.flow_ref.clone())
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref info) => {
                    inline_block_flows.push(info.flow_ref.clone())
                }
                SpecificFragmentInfo::InlineAbsolute(ref info) => {
                    inline_block_flows.push(info.flow_ref.clone())
                }
                _ => {}
            }
        }

        // We must scan for runs before computing minimum ascent and descent because scanning
        // for runs might collapse so much whitespace away that only hypothetical fragments
        // remain. In that case the inline flow will compute its ascent and descent to be zero.
        let scanned_fragments =
            TextRunScanner::new().scan_for_runs(&mut self.layout_context.font_context(),
                                                fragments.fragments);
        let mut inline_flow_ref: FlowRef = Arc::new(
            InlineFlow::from_fragments(scanned_fragments, node.style().writing_mode));

        // Add all the inline-block fragments as children of the inline flow.
        for inline_block_flow in &inline_block_flows {
            inline_flow_ref.add_new_child(inline_block_flow.clone());
        }

        // Set up absolute descendants as necessary.
        //
        // TODO(pcwalton): The inline flow itself may need to become the containing block for
        // absolute descendants in order to handle cases like:
        //
        //      <div>
        //          <span style="position: relative">
        //              <span style="position: absolute; ..."></span>
        //          </span>
        //      </div>
        //
        // See the comment above `flow::AbsoluteDescendantInfo` for more information.
        absolute_descendants.push_descendants(fragments.absolute_descendants);

        {
            // FIXME(#6503): Use Arc::get_mut().unwrap() here.
            let inline_flow = flow_ref::deref_mut(&mut inline_flow_ref).as_mut_inline();


            let (ascent, descent) =
                inline_flow.compute_minimum_ascent_and_descent(&mut self.layout_context
                                                                        .font_context(),
                                                               &**node.style());
            inline_flow.minimum_block_size_above_baseline = ascent;
            inline_flow.minimum_depth_below_baseline = descent;
        }

        inline_flow_ref.finish();

        if flow.need_anonymous_flow(&*inline_flow_ref) {
            flow_list.push(inline_flow_ref)
        } else {
            flow.add_new_child(inline_flow_ref)
        }
    }

    fn build_block_flow_using_construction_result_of_child(
            &mut self,
            flow: &mut FlowRef,
            consecutive_siblings: &mut Vec<FlowRef>,
            node: &ThreadSafeLayoutNode,
            kid: ThreadSafeLayoutNode,
            inline_fragment_accumulator: &mut InlineFragmentsAccumulator,
            abs_descendants: &mut AbsoluteDescendants) {
        match kid.swap_out_construction_result() {
            ConstructionResult::None => {}
            ConstructionResult::Flow(mut kid_flow, kid_abs_descendants) => {
                // If kid_flow is TableCaptionFlow, kid_flow should be added under
                // TableWrapperFlow.
                if flow.is_table() && kid_flow.is_table_caption() {
                    self.set_flow_construction_result(&kid,
                                                      ConstructionResult::Flow(kid_flow,
                                                                               AbsoluteDescendants::new()))
                } else if flow.need_anonymous_flow(&*kid_flow) {
                    consecutive_siblings.push(kid_flow)
                } else {
                    // Flush any inline fragments that we were gathering up. This allows us to
                    // handle {ib} splits.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.fragments.len());
                    let old_inline_fragment_accumulator =
                        mem::replace(inline_fragment_accumulator,
                                     InlineFragmentsAccumulator::new());
                    self.flush_inline_fragments_to_flow_or_list(
                        old_inline_fragment_accumulator,
                        flow,
                        consecutive_siblings,
                        abs_descendants,
                        node);
                    if !consecutive_siblings.is_empty() {
                        let consecutive_siblings = mem::replace(consecutive_siblings, vec!());
                        self.generate_anonymous_missing_child(consecutive_siblings, flow, node);
                    }
                    self.generate_anonymous_table_flows_if_necessary(flow, &mut kid_flow, &kid);
                    flow.add_new_child(kid_flow);
                }
                abs_descendants.push_descendants(kid_abs_descendants);
            }
            ConstructionResult::ConstructionItem(ConstructionItem::InlineFragments(
                    InlineFragmentsConstructionResult {
                        splits,
                        fragments: successor_fragments,
                    })) => {
                // Add any {ib} splits.
                for split in splits {
                    // Pull apart the {ib} split object and push its predecessor fragments
                    // onto the list.
                    let InlineBlockSplit {
                        predecessors,
                        flow: kid_flow
                    } = split;
                    inline_fragment_accumulator.push_all(predecessors);

                    // Flush any inline fragments that we were gathering up.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.fragments.len());
                    let old_inline_fragment_accumulator =
                        mem::replace(inline_fragment_accumulator,
                                     InlineFragmentsAccumulator::new());
                    self.flush_inline_fragments_to_flow_or_list(
                            old_inline_fragment_accumulator,
                            flow,
                            consecutive_siblings,
                            &mut inline_fragment_accumulator.fragments.absolute_descendants,
                            node);

                    // Push the flow generated by the {ib} split onto our list of
                    // flows.
                    if flow.need_anonymous_flow(&*kid_flow) {
                        consecutive_siblings.push(kid_flow)
                    } else {
                        flow.add_new_child(kid_flow)
                    }
                }

                // Add the fragments to the list we're maintaining.
                inline_fragment_accumulator.push_all(successor_fragments);
            }
            ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                    whitespace_node,
                    whitespace_pseudo,
                    mut whitespace_style,
                    whitespace_damage)) => {
                // Add whitespace results. They will be stripped out later on when
                // between block elements, and retained when between inline elements.
                let fragment_info = SpecificFragmentInfo::UnscannedText(
                    UnscannedTextFragmentInfo::from_text(" ".to_owned()));
                properties::modify_style_for_replaced_content(&mut whitespace_style);
                let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                    whitespace_pseudo,
                                                                    whitespace_style,
                                                                    whitespace_damage,
                                                                    fragment_info);
                inline_fragment_accumulator.fragments.fragments.push_back(fragment);
            }
            ConstructionResult::ConstructionItem(ConstructionItem::TableColumnFragment(_)) => {
                // TODO: Implement anonymous table objects for missing parents
                // CSS 2.1 § 17.2.1, step 3-2
            }
        }
    }

    /// Constructs a block flow, beginning with the given `initial_fragments` if present and then
    /// appending the construction results of children to the child list of the block flow. {ib}
    /// splits and absolutely-positioned descendants are handled correctly.
    fn build_flow_for_block_starting_with_fragments(
            &mut self,
            mut flow: FlowRef,
            node: &ThreadSafeLayoutNode,
            initial_fragments: IntermediateInlineFragments)
            -> ConstructionResult {
        // Gather up fragments for the inline flows we might need to create.
        let mut inline_fragment_accumulator = InlineFragmentsAccumulator::new();
        let mut consecutive_siblings = vec!();

        inline_fragment_accumulator.fragments.push_all(initial_fragments);

        // List of absolute descendants, in tree order.
        let mut abs_descendants = AbsoluteDescendants::new();
        for kid in node.children() {
            if kid.get_pseudo_element_type() != PseudoElementType::Normal {
                self.process(&kid);
            }

            self.build_block_flow_using_construction_result_of_child(
                &mut flow,
                &mut consecutive_siblings,
                node,
                kid,
                &mut inline_fragment_accumulator,
                &mut abs_descendants);
        }

        // Perform a final flush of any inline fragments that we were gathering up to handle {ib}
        // splits, after stripping ignorable whitespace.
        self.flush_inline_fragments_to_flow_or_list(inline_fragment_accumulator,
                                                    &mut flow,
                                                    &mut consecutive_siblings,
                                                    &mut abs_descendants,
                                                    node);
        if !consecutive_siblings.is_empty() {
            self.generate_anonymous_missing_child(consecutive_siblings, &mut flow, node);
        }

        // The flow is done.
        flow.finish();

        // Set up the absolute descendants.
        let contains_positioned_fragments = flow.contains_positioned_fragments();
        let is_absolutely_positioned = flow::base(&*flow).flags.contains(IS_ABSOLUTELY_POSITIONED);
        if contains_positioned_fragments {
            // This is the containing block for all the absolute descendants.
            flow.set_absolute_descendants(abs_descendants);

            abs_descendants = AbsoluteDescendants::new();
            if is_absolutely_positioned {
                // This is now the only absolute flow in the subtree which hasn't yet
                // reached its CB.
                abs_descendants.push(flow.clone());
            }
        }
        ConstructionResult::Flow(flow, abs_descendants)
    }

    /// Constructs a flow for the given block node and its children. This method creates an
    /// initial fragment as appropriate and then dispatches to
    /// `build_flow_for_block_starting_with_fragments`. Currently the following kinds of flows get
    /// initial content:
    ///
    /// * Generated content gets the initial content specified by the `content` attribute of the
    ///   CSS.
    /// * `<input>` and `<textarea>` elements get their content.
    ///
    /// FIXME(pcwalton): It is not clear to me that there isn't a cleaner way to handle
    /// `<textarea>`.
    fn build_flow_for_block_like(&mut self, flow: FlowRef, node: &ThreadSafeLayoutNode)
                            -> ConstructionResult {
        let mut initial_fragments = IntermediateInlineFragments::new();
        let node_is_input_or_text_area =
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                       HTMLElementTypeId::HTMLInputElement))) ||
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                       HTMLElementTypeId::HTMLTextAreaElement)));
        if node.get_pseudo_element_type() != PseudoElementType::Normal ||
                node_is_input_or_text_area {
            // A TextArea's text contents are displayed through the input text
            // box, so don't construct them.
            if node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTextAreaElement))) {
                for kid in node.children() {
                    self.set_flow_construction_result(&kid, ConstructionResult::None)
                }
            }

            let mut style = node.style().clone();
            if node_is_input_or_text_area {
                properties::modify_style_for_input_text(&mut style);
            }

            self.create_fragments_for_node_text_content(&mut initial_fragments, node, &style)
        }

        self.build_flow_for_block_starting_with_fragments(flow, node, initial_fragments)
    }

    /// Pushes fragments appropriate for the content of the given node onto the given list.
    fn create_fragments_for_node_text_content(&self,
                                              fragments: &mut IntermediateInlineFragments,
                                              node: &ThreadSafeLayoutNode,
                                              style: &Arc<ComputedValues>) {
        // Fast path: If there is no text content, return immediately.
        let text_content = node.text_content();
        if text_content.is_empty() {
            return
        }

        let mut style = (*style).clone();
        properties::modify_style_for_text(&mut style);
        for content_item in text_content {
            let specific = match content_item {
                ContentItem::String(string) => {
                    let info = UnscannedTextFragmentInfo::from_text(string);
                    SpecificFragmentInfo::UnscannedText(info)
                }
                content_item => {
                    let content_item = box GeneratedContentInfo::ContentItem(content_item);
                    SpecificFragmentInfo::GeneratedContent(content_item)
                }
            };

            fragments.fragments
                     .push_back(Fragment::from_opaque_node_and_style(node.opaque(),
                                                                     node.get_pseudo_element_type()
                                                                         .strip(),
                                                                     style.clone(),
                                                                     node.restyle_damage(),
                                                                     specific))
        }
    }


    /// Builds a flow for a node with `display: block`. This yields a `BlockFlow` with possibly
    /// other `BlockFlow`s or `InlineFlow`s underneath it, depending on whether {ib} splits needed
    /// to happen.
    fn build_flow_for_block(&mut self, node: &ThreadSafeLayoutNode, float_kind: Option<FloatKind>)
                            -> ConstructionResult {
        let fragment = self.build_fragment_for_block(node);
        let flow: FlowRef = if node.style().is_multicol() {
            Arc::new(MulticolFlow::from_fragment(fragment, float_kind))
        } else {
            Arc::new(BlockFlow::from_fragment(fragment, float_kind))
        };
        self.build_flow_for_block_like(flow, node)
    }

    /// Bubbles up {ib} splits.
    fn accumulate_inline_block_splits(&mut self,
                                      splits: LinkedList<InlineBlockSplit>,
                                      node: &ThreadSafeLayoutNode,
                                      fragment_accumulator: &mut InlineFragmentsAccumulator,
                                      opt_inline_block_splits: &mut LinkedList<InlineBlockSplit>) {
        for split in splits {
            let InlineBlockSplit {
                predecessors,
                flow: kid_flow
            } = split;
            fragment_accumulator.push_all(predecessors);

            let split = InlineBlockSplit {
                predecessors: mem::replace(
                    fragment_accumulator,
                    InlineFragmentsAccumulator::from_inline_node(
                        node)).to_intermediate_inline_fragments(),
                flow: kid_flow,
            };
            opt_inline_block_splits.push_back(split)
        }
    }

    /// Concatenates the fragments of kids, adding in our own borders/padding/margins if necessary.
    /// Returns the `InlineFragmentsConstructionResult`, if any. There will be no
    /// `InlineFragmentsConstructionResult` if this node consisted entirely of ignorable
    /// whitespace.
    fn build_fragments_for_nonreplaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                                      -> ConstructionResult {
        let mut opt_inline_block_splits: LinkedList<InlineBlockSplit> = LinkedList::new();
        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.bidi_control_chars = bidi_control_chars(&*node.style());

        let mut abs_descendants = AbsoluteDescendants::new();

        // Concatenate all the fragments of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            if kid.get_pseudo_element_type() != PseudoElementType::Normal {
                self.process(&kid);
            }
            match kid.swap_out_construction_result() {
                ConstructionResult::None => {}
                ConstructionResult::Flow(flow, kid_abs_descendants) => {
                    if !flow::base(&*flow).flags.contains(IS_ABSOLUTELY_POSITIONED) {
                        // {ib} split. Flush the accumulator to our new split and make a new
                        // accumulator to hold any subsequent fragments we come across.
                        let split = InlineBlockSplit {
                            predecessors:
                                mem::replace(
                                    &mut fragment_accumulator,
                                    InlineFragmentsAccumulator::from_inline_node(
                                        node)).to_intermediate_inline_fragments(),
                            flow: flow,
                        };
                        opt_inline_block_splits.push_back(split);
                        abs_descendants.push_descendants(kid_abs_descendants);
                    } else {
                        // Push the absolutely-positioned kid as an inline containing block.
                        let kid_node = flow.as_block().fragment.node;
                        let kid_pseudo = flow.as_block().fragment.pseudo.clone();
                        let kid_style = flow.as_block().fragment.style.clone();
                        let kid_restyle_damage = flow.as_block().fragment.restyle_damage;
                        let fragment_info = SpecificFragmentInfo::InlineAbsolute(
                            InlineAbsoluteFragmentInfo::new(flow));
                        fragment_accumulator.push(Fragment::from_opaque_node_and_style(
                                kid_node,
                                kid_pseudo,
                                kid_style,
                                kid_restyle_damage,
                                fragment_info));
                        fragment_accumulator.fragments
                                            .absolute_descendants
                                            .push_descendants(kid_abs_descendants);
                    }
                }
                ConstructionResult::ConstructionItem(ConstructionItem::InlineFragments(
                        InlineFragmentsConstructionResult {
                            splits,
                            fragments: successors,
                        })) => {

                    // Bubble up {ib} splits.
                    self.accumulate_inline_block_splits(splits,
                                                        node,
                                                        &mut fragment_accumulator,
                                                        &mut opt_inline_block_splits);

                    // Push residual fragments.
                    fragment_accumulator.push_all(successors);
                }
                ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                        whitespace_node,
                        whitespace_pseudo,
                        mut whitespace_style,
                        whitespace_damage)) => {
                    // Instantiate the whitespace fragment.
                    let fragment_info = SpecificFragmentInfo::UnscannedText(
                        UnscannedTextFragmentInfo::from_text(" ".to_owned()));
                    properties::modify_style_for_replaced_content(&mut whitespace_style);
                    let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                        whitespace_pseudo,
                                                                        whitespace_style,
                                                                        whitespace_damage,
                                                                        fragment_info);
                    fragment_accumulator.fragments.fragments.push_back(fragment)
                }
                ConstructionResult::ConstructionItem(ConstructionItem::TableColumnFragment(_)) => {
                    // TODO: Implement anonymous table objects for missing parents
                    // CSS 2.1 § 17.2.1, step 3-2
                }
            }
        }

        // Finally, make a new construction result.
        if opt_inline_block_splits.len() > 0 || !fragment_accumulator.fragments.is_empty()
                || abs_descendants.len() > 0 {
            fragment_accumulator.fragments.absolute_descendants.push_descendants(abs_descendants);
            let construction_item = ConstructionItem::InlineFragments(
                    InlineFragmentsConstructionResult {
                splits: opt_inline_block_splits,
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
            });
            ConstructionResult::ConstructionItem(construction_item)
        } else {
            ConstructionResult::None
        }
    }

    /// Creates an `InlineFragmentsConstructionResult` for replaced content. Replaced content
    /// doesn't render its children, so this just nukes a child's fragments and creates a
    /// `Fragment`.
    fn build_fragments_for_replaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                                   -> ConstructionResult {
        for kid in node.children() {
            self.set_flow_construction_result(&kid, ConstructionResult::None)
        }

        // If this node is ignorable whitespace, bail out now.
        //
        // FIXME(#2001, pcwalton): Don't do this if there's padding or borders.
        if node.is_ignorable_whitespace() {
            return ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                node.opaque(),
                node.get_pseudo_element_type().strip(),
                node.style().clone(),
                node.restyle_damage()))
        }

        // Modify the style as necessary. (See the comment in
        // `properties::modify_style_for_replaced_content()`.)
        let mut style = (*node.style()).clone();
        properties::modify_style_for_replaced_content(&mut style);

        // If this is generated content, then we need to initialize the accumulator with the
        // fragment corresponding to that content. Otherwise, just initialize with the ordinary
        // fragment that needs to be generated for this inline node.
        let mut fragments = IntermediateInlineFragments::new();
        match (node.get_pseudo_element_type(), node.type_id()) {
            (_, Some(NodeTypeId::CharacterData(CharacterDataTypeId::Text))) => {
                self.create_fragments_for_node_text_content(&mut fragments, node, &style)
            }
            (PseudoElementType::Normal, _) => {
                fragments.fragments.push_back(self.build_fragment_for_block(node));
            }
            (_, _) => self.create_fragments_for_node_text_content(&mut fragments, node, &style),
        }

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: LinkedList::new(),
                fragments: fragments,
            });
        ConstructionResult::ConstructionItem(construction_item)
    }

    fn build_fragment_for_inline_block(&mut self, node: &ThreadSafeLayoutNode)
                                       -> ConstructionResult {
        let block_flow_result = self.build_flow_for_block(node, None);
        let (block_flow, abs_descendants) = match block_flow_result {
            ConstructionResult::Flow(block_flow, abs_descendants) => (block_flow, abs_descendants),
            _ => unreachable!()
        };

        let mut modified_style = (*node.style()).clone();
        properties::modify_style_for_outer_inline_block_fragment(&mut modified_style);
        let fragment_info = SpecificFragmentInfo::InlineBlock(InlineBlockFragmentInfo::new(
                block_flow));
        let fragment = Fragment::from_opaque_node_and_style(node.opaque(),
                                                            node.get_pseudo_element_type().strip(),
                                                            modified_style.clone(),
                                                            node.restyle_damage(),
                                                            fragment_info);

        let mut fragment_accumulator =
            InlineFragmentsAccumulator::from_inline_node_and_style(node, modified_style);
        fragment_accumulator.fragments.fragments.push_back(fragment);
        fragment_accumulator.fragments.absolute_descendants.push_descendants(abs_descendants);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: LinkedList::new(),
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
            });
        ConstructionResult::ConstructionItem(construction_item)
    }

    /// This is an annoying case, because the computed `display` value is `block`, but the
    /// hypothetical box is inline.
    fn build_fragment_for_absolutely_positioned_inline(&mut self, node: &ThreadSafeLayoutNode)
                                                       -> ConstructionResult {
        let block_flow_result = self.build_flow_for_block(node, None);
        let (block_flow, abs_descendants) = match block_flow_result {
            ConstructionResult::Flow(block_flow, abs_descendants) => (block_flow, abs_descendants),
            _ => unreachable!()
        };

        let fragment_info = SpecificFragmentInfo::InlineAbsoluteHypothetical(
            InlineAbsoluteHypotheticalFragmentInfo::new(block_flow));
        let mut style = node.style().clone();
        properties::modify_style_for_inline_absolute_hypothetical_fragment(&mut style);
        let fragment = Fragment::from_opaque_node_and_style(node.opaque(),
                                                            PseudoElementType::Normal,
                                                            style,
                                                            node.restyle_damage(),
                                                            fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.fragments.push_back(fragment);
        fragment_accumulator.fragments.absolute_descendants.push_descendants(abs_descendants);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: LinkedList::new(),
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
            });
        ConstructionResult::ConstructionItem(construction_item)
    }

    /// Builds one or more fragments for a node with `display: inline`. This yields an
    /// `InlineFragmentsConstructionResult`.
    fn build_fragments_for_inline(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        // Is this node replaced content?
        if !node.is_replaced_content() {
            // Go to a path that concatenates our kids' fragments.
            self.build_fragments_for_nonreplaced_inline_content(node)
        } else {
            // Otherwise, just nuke our kids' fragments, create our fragment if any, and be done
            // with it.
            self.build_fragments_for_replaced_inline_content(node)
        }
    }

    /// Places any table captions found under the given table wrapper, if the value of their
    /// `caption-side` property is equal to the given `side`.
    fn place_table_caption_under_table_wrapper_on_side(&mut self,
                                                       table_wrapper_flow: &mut FlowRef,
                                                       node: &ThreadSafeLayoutNode,
                                                       side: caption_side::T) {
        // Only flows that are table captions are matched here.
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                ConstructionResult::Flow(kid_flow, _) => {
                    if kid_flow.is_table_caption() &&
                        kid_flow.as_block()
                                .fragment
                                .style()
                                .get_inheritedtable()
                                .caption_side == side {
                        table_wrapper_flow.add_new_child(kid_flow);
                    }
                }
                ConstructionResult::None | ConstructionResult::ConstructionItem(_) => {}
            }
        }
    }

    /// Generates an anonymous table flow according to CSS 2.1 § 17.2.1, step 2.
    /// If necessary, generate recursively another anonymous table flow.
    fn generate_anonymous_missing_child(&mut self,
                                        child_flows: Vec<FlowRef>,
                                        flow: &mut FlowRef,
                                        node: &ThreadSafeLayoutNode) {
        let mut anonymous_flow = flow.generate_missing_child_flow(node);
        let mut consecutive_siblings = vec!();
        for kid_flow in child_flows {
            if anonymous_flow.need_anonymous_flow(&*kid_flow) {
                consecutive_siblings.push(kid_flow);
                continue;
            }
            if !consecutive_siblings.is_empty() {
                self.generate_anonymous_missing_child(consecutive_siblings,
                                                      &mut anonymous_flow,
                                                      node);
                consecutive_siblings = vec!();
            }
            anonymous_flow.add_new_child(kid_flow);
        }
        if !consecutive_siblings.is_empty() {
            self.generate_anonymous_missing_child(consecutive_siblings, &mut anonymous_flow, node);
        }
        // The flow is done.
        anonymous_flow.finish();
        flow.add_new_child(anonymous_flow);
    }

    /// Builds a flow for a node with `display: table`. This yields a `TableWrapperFlow` with
    /// possibly other `TableCaptionFlow`s or `TableFlow`s underneath it.
    fn build_flow_for_table_wrapper(&mut self, node: &ThreadSafeLayoutNode, float_value: float::T)
                                    -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableWrapper);
        let mut wrapper_flow: FlowRef = Arc::new(
            TableWrapperFlow::from_fragment(fragment, FloatKind::from_property(float_value)));

        let table_fragment = Fragment::new(node, SpecificFragmentInfo::Table);
        let table_flow = Arc::new(TableFlow::from_fragment(table_fragment));

        // First populate the table flow with its children.
        let construction_result = self.build_flow_for_block_like(table_flow, node);

        let mut abs_descendants = AbsoluteDescendants::new();
        let mut fixed_descendants = AbsoluteDescendants::new();

        // The order of the caption and the table are not necessarily the same order as in the DOM
        // tree. All caption blocks are placed before or after the table flow, depending on the
        // value of `caption-side`.
        self.place_table_caption_under_table_wrapper_on_side(&mut wrapper_flow,
                                                             node,
                                                             caption_side::T::top);

        match construction_result {
            ConstructionResult::Flow(table_flow, table_abs_descendants) => {
                wrapper_flow.add_new_child(table_flow);
                abs_descendants.push_descendants(table_abs_descendants);
            }
            _ => {}
        }

        // If the value of `caption-side` is `bottom`, place it now.
        self.place_table_caption_under_table_wrapper_on_side(&mut wrapper_flow,
                                                             node,
                                                             caption_side::T::bottom);

        // The flow is done.
        wrapper_flow.finish();
        let contains_positioned_fragments = wrapper_flow.contains_positioned_fragments();
        let is_fixed_positioned = wrapper_flow.as_block().is_fixed();
        let is_absolutely_positioned =
            flow::base(&*wrapper_flow).flags.contains(IS_ABSOLUTELY_POSITIONED);
        if contains_positioned_fragments {
            // This is the containing block for all the absolute descendants.
            wrapper_flow.set_absolute_descendants(abs_descendants);

            abs_descendants = AbsoluteDescendants::new();

            if is_fixed_positioned {
                // Send itself along with the other fixed descendants.
                fixed_descendants.push(wrapper_flow.clone());
            } else if is_absolutely_positioned {
                // This is now the only absolute flow in the subtree which hasn't yet
                // reached its containing block.
                abs_descendants.push(wrapper_flow.clone());
            }
        }

        ConstructionResult::Flow(wrapper_flow, abs_descendants)
    }

    /// Builds a flow for a node with `display: table-caption`. This yields a `TableCaptionFlow`
    /// with possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_caption(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = self.build_fragment_for_block(node);
        let flow = Arc::new(TableCaptionFlow::from_fragment(fragment));
        self.build_flow_for_block_like(flow, node)
    }

    /// Builds a flow for a node with `display: table-row-group`. This yields a `TableRowGroupFlow`
    /// with possibly other `TableRowFlow`s underneath it.
    fn build_flow_for_table_rowgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableRow);
        let flow = Arc::new(TableRowGroupFlow::from_fragment(fragment));
        self.build_flow_for_block_like(flow, node)
    }

    /// Builds a flow for a node with `display: table-row`. This yields a `TableRowFlow` with
    /// possibly other `TableCellFlow`s underneath it.
    fn build_flow_for_table_row(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableRow);
        let flow = Arc::new(TableRowFlow::from_fragment(fragment));
        self.build_flow_for_block_like(flow, node)
    }

    /// Builds a flow for a node with `display: table-cell`. This yields a `TableCellFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_cell(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableCell);

        // Determine if the table cell should be hidden. Per CSS 2.1 § 17.6.1.1, this will be true
        // if the cell has any in-flow elements (even empty ones!) and has `empty-cells` set to
        // `hide`.
        let hide = node.style().get_inheritedtable().empty_cells == empty_cells::T::hide &&
            node.children().all(|kid| {
                let position = kid.style().get_box().position;
                !kid.is_content() ||
                position == position::T::absolute ||
                position == position::T::fixed
            });

        let flow = Arc::new(
            TableCellFlow::from_node_fragment_and_visibility_flag(node, fragment, !hide));
        self.build_flow_for_block_like(flow, node)
    }

    /// Builds a flow for a node with `display: list-item`. This yields a `ListItemFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_list_item(&mut self, node: &ThreadSafeLayoutNode, flotation: float::T)
                                -> ConstructionResult {
        let flotation = FloatKind::from_property(flotation);
        let marker_fragments = match node.style().get_list().list_style_image.0 {
            Some(ref url) => {
                let image_info = box ImageFragmentInfo::new(node,
                                                            Some((*url).clone()),
                                                            &self.layout_context);
                vec![Fragment::new(node, SpecificFragmentInfo::Image(image_info))]
            }
            None => {
                match ListStyleTypeContent::from_list_style_type(node.style()
                                                                     .get_list()
                                                                     .list_style_type) {
                    ListStyleTypeContent::None => Vec::new(),
                    ListStyleTypeContent::StaticText(ch) => {
                        let text = format!("{}\u{a0}", ch);
                        let mut unscanned_marker_fragments = LinkedList::new();
                        unscanned_marker_fragments.push_back(Fragment::new(
                            node,
                            SpecificFragmentInfo::UnscannedText(
                                UnscannedTextFragmentInfo::from_text(text))));
                        let marker_fragments = TextRunScanner::new().scan_for_runs(
                            &mut self.layout_context.font_context(),
                            unscanned_marker_fragments);
                        marker_fragments.fragments
                    }
                    ListStyleTypeContent::GeneratedContent(info) => {
                        vec![Fragment::new(node, SpecificFragmentInfo::GeneratedContent(info))]
                    }
                }
            }
        };

        // If the list marker is outside, it becomes the special "outside fragment" that list item
        // flows have. If it's inside, it's just a plain old fragment. Note that this means that
        // we adopt Gecko's behavior rather than WebKit's when the marker causes an {ib} split,
        // which has caused some malaise (Bugzilla #36854) but CSS 2.1 § 12.5.1 lets me do it, so
        // there.
        let mut initial_fragments = IntermediateInlineFragments::new();
        let main_fragment = self.build_fragment_for_block(node);
        let flow = match node.style().get_list().list_style_position {
            list_style_position::T::outside => {
                Arc::new(ListItemFlow::from_fragments_and_flotation(
                    main_fragment, marker_fragments, flotation))
            }
            list_style_position::T::inside => {
                for marker_fragment in marker_fragments {
                    initial_fragments.fragments.push_back(marker_fragment)
                }
                Arc::new(ListItemFlow::from_fragments_and_flotation(
                    main_fragment, vec![], flotation))
            }
        };

        self.build_flow_for_block_starting_with_fragments(flow, node, initial_fragments)
    }

    /// Creates a fragment for a node with `display: table-column`.
    fn build_fragments_for_table_column(&mut self, node: &ThreadSafeLayoutNode)
                                        -> ConstructionResult {
        // CSS 2.1 § 17.2.1. Treat all child fragments of a `table-column` as `display: none`.
        for kid in node.children() {
            self.set_flow_construction_result(&kid, ConstructionResult::None)
        }

        let specific = SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node));
        let construction_item = ConstructionItem::TableColumnFragment(Fragment::new(node,
                                                                                    specific));
        ConstructionResult::ConstructionItem(construction_item)
    }

    /// Builds a flow for a node with `display: table-column-group`.
    /// This yields a `TableColGroupFlow`.
    fn build_flow_for_table_colgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment =
            Fragment::new(node,
                          SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node)));
        let mut col_fragments = vec!();
        for kid in node.children() {
            // CSS 2.1 § 17.2.1. Treat all non-column child fragments of `table-column-group`
            // as `display: none`.
            match kid.swap_out_construction_result() {
                ConstructionResult::ConstructionItem(ConstructionItem::TableColumnFragment(
                        fragment)) => {
                    col_fragments.push(fragment);
                }
                _ => {}
            }
        }
        if col_fragments.is_empty() {
            debug!("add SpecificFragmentInfo::TableColumn for empty colgroup");
            let specific = SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node));
            col_fragments.push(Fragment::new(node, specific));
        }
        let mut flow: FlowRef = Arc::new(TableColGroupFlow::from_fragments(fragment, col_fragments));
        flow.finish();

        ConstructionResult::Flow(flow, AbsoluteDescendants::new())
    }

    /// Builds a flow for a node with 'display: flex'.
    fn build_flow_for_flex(&mut self, node: &ThreadSafeLayoutNode, float_kind: Option<FloatKind>)
                           -> ConstructionResult {
        let fragment = self.build_fragment_for_block(node);
        let flow = Arc::new(FlexFlow::from_fragment(fragment, float_kind));
        self.build_flow_for_block_like(flow, node)
    }

    /// Attempts to perform incremental repair to account for recent changes to this node. This
    /// can fail and return false, indicating that flows will need to be reconstructed.
    ///
    /// TODO(pcwalton): Add some more fast paths, like toggling `display: none`, adding block kids
    /// to block parents with no {ib} splits, adding out-of-flow kids, etc.
    pub fn repair_if_possible(&mut self, node: &ThreadSafeLayoutNode) -> bool {
        // We can skip reconstructing the flow if we don't have to reconstruct and none of our kids
        // did either.
        if node.restyle_damage().contains(RECONSTRUCT_FLOW) {
            return false
        }

        let mut need_to_reconstruct = false;
        for kid in node.children() {
            if kid.flags().contains(HAS_NEWLY_CONSTRUCTED_FLOW) {
                kid.remove_flags(HAS_NEWLY_CONSTRUCTED_FLOW);
                need_to_reconstruct = true
            }
        }
        if need_to_reconstruct {
            return false
        }

        let mut style = node.style().clone();
        let mut layout_data_ref = node.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");
        let damage = layout_data.data.restyle_damage;
        match node.construction_result_mut(layout_data) {
            &mut ConstructionResult::None => true,
            &mut ConstructionResult::Flow(ref mut flow, _) => {
                // The node's flow is of the same type and has the same set of children and can
                // therefore be repaired by simply propagating damage and style to the flow.
                if !flow.is_block_flow() {
                    return false
                }
                let flow = flow_ref::deref_mut(flow);
                flow::mut_base(flow).restyle_damage.insert(damage);
                flow.repair_style_and_bubble_inline_sizes(&style);
                true
            }
            &mut ConstructionResult::ConstructionItem(ConstructionItem::InlineFragments(
                    ref mut inline_fragments_construction_result)) => {
                if !inline_fragments_construction_result.splits.is_empty() {
                    return false
                }

                for fragment in inline_fragments_construction_result.fragments
                                                                    .fragments
                                                                    .iter_mut() {
                    // Only mutate the styles of fragments that represent the dirty node (including
                    // pseudo-element).
                    if fragment.node != node.opaque() {
                        continue
                    }
                    if fragment.pseudo != node.get_pseudo_element_type().strip() {
                        continue
                    }

                    match fragment.specific {
                        SpecificFragmentInfo::InlineBlock(ref mut inline_block_fragment) => {
                            let flow_ref = flow_ref::deref_mut(&mut inline_block_fragment.flow_ref);
                            flow::mut_base(flow_ref).restyle_damage.insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            flow_ref.repair_style_and_bubble_inline_sizes(&style);
                        }
                        SpecificFragmentInfo::InlineAbsoluteHypothetical(
                                ref mut inline_absolute_hypothetical_fragment) => {
                            let flow_ref = flow_ref::deref_mut(
                                &mut inline_absolute_hypothetical_fragment.flow_ref);
                            flow::mut_base(flow_ref).restyle_damage.insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            flow_ref.repair_style_and_bubble_inline_sizes(&style);
                        }
                        SpecificFragmentInfo::InlineAbsolute(ref mut inline_absolute_fragment) => {
                            let flow_ref = flow_ref::deref_mut(
                                &mut inline_absolute_fragment.flow_ref);
                            flow::mut_base(flow_ref).restyle_damage.insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            flow_ref.repair_style_and_bubble_inline_sizes(&style);
                        }
                        SpecificFragmentInfo::ScannedText(_) |
                        SpecificFragmentInfo::UnscannedText(_) => {
                            properties::modify_style_for_text(&mut style);
                            properties::modify_style_for_replaced_content(&mut style);
                            fragment.repair_style(&style);
                        }
                        _ => {
                            if node.is_replaced_content() {
                                properties::modify_style_for_replaced_content(&mut style);
                            }
                            fragment.repair_style(&style);
                        }
                    }
                }
                true
            }
            &mut ConstructionResult::ConstructionItem(_) => {
                false
            }
        }
    }
}

impl<'a> PostorderNodeMutTraversal for FlowConstructor<'a> {
    // Construct Flow based on 'display', 'position', and 'float' values.
    //
    // CSS 2.1 Section 9.7
    //
    // TODO: This should actually consult the table in that section to get the
    // final computed value for 'display'.
    fn process(&mut self, node: &ThreadSafeLayoutNode) -> bool {
        // Get the `display` property for this node, and determine whether this node is floated.
        let (display, float, positioning) = match node.type_id() {
            None => {
                // Pseudo-element.
                let style = node.style();
                let display = match node.get_pseudo_element_type() {
                    PseudoElementType::Normal => display::T::inline,
                    PseudoElementType::Before(display) => display,
                    PseudoElementType::After(display) => display,
                };
                (display, style.get_box().float, style.get_box().position)
            }
            Some(NodeTypeId::Element(_)) => {
                let style = node.style();
                let munged_display = if style.get_box()._servo_display_for_hypothetical_box ==
                        display::T::inline {
                    display::T::inline
                } else {
                    style.get_box().display
                };
                (munged_display, style.get_box().float, style.get_box().position)
            }
            Some(NodeTypeId::CharacterData(CharacterDataTypeId::Text)) =>
                (display::T::inline, float::T::none, position::T::static_),
            Some(NodeTypeId::CharacterData(CharacterDataTypeId::Comment)) |
            Some(NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)) |
            Some(NodeTypeId::DocumentType) |
            Some(NodeTypeId::DocumentFragment) |
            Some(NodeTypeId::Document) => {
                (display::T::none, float::T::none, position::T::static_)
            }
        };

        debug!("building flow for node: {:?} {:?} {:?} {:?}", display, float, positioning, node.type_id());

        // Switch on display and floatedness.
        match (display, float, positioning) {
            // `display: none` contributes no flow construction result. Nuke the flow construction
            // results of children.
            (display::T::none, _, _) => {
                for child in node.children() {
                    self.set_flow_construction_result(&child, ConstructionResult::None);
                }
                self.set_flow_construction_result(node, ConstructionResult::None);
            }

            // Table items contribute table flow construction results.
            (display::T::table, float_value, _) => {
                let construction_result = self.build_flow_for_table_wrapper(node, float_value);
                self.set_flow_construction_result(node, construction_result)
            }

            // Absolutely positioned elements will have computed value of
            // `float` as 'none' and `display` as per the table.
            // Only match here for block items. If an item is absolutely
            // positioned, but inline we shouldn't try to construct a block
            // flow here - instead, let it match the inline case
            // below.
            (display::T::block, _, position::T::absolute) |
            (_, _, position::T::fixed) => {
                let construction_result = self.build_flow_for_block(node, None);
                self.set_flow_construction_result(node, construction_result)
            }

            // List items contribute their own special flows.
            (display::T::list_item, float_value, _) => {
                let construction_result = self.build_flow_for_list_item(node, float_value);
                self.set_flow_construction_result(node, construction_result)
            }

            // Inline items that are absolutely-positioned contribute inline fragment construction
            // results with a hypothetical fragment.
            (display::T::inline, _, position::T::absolute) => {
                let construction_result =
                    self.build_fragment_for_absolutely_positioned_inline(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Inline items contribute inline fragment construction results.
            //
            // FIXME(pcwalton, #3307): This is not sufficient to handle floated generated content.
            (display::T::inline, float::T::none, _) => {
                let construction_result = self.build_fragments_for_inline(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Inline-block items contribute inline fragment construction results.
            (display::T::inline_block, float::T::none, _) => {
                let construction_result = self.build_fragment_for_inline_block(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_caption, _, _) => {
                let construction_result = self.build_flow_for_table_caption(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_column_group, _, _) => {
                let construction_result = self.build_flow_for_table_colgroup(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_column, _, _) => {
                let construction_result = self.build_fragments_for_table_column(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_row_group, _, _) |
            (display::T::table_header_group, _, _) |
            (display::T::table_footer_group, _, _) => {
                let construction_result = self.build_flow_for_table_rowgroup(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_row, _, _) => {
                let construction_result = self.build_flow_for_table_row(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_cell, _, _) => {
                let construction_result = self.build_flow_for_table_cell(node);
                self.set_flow_construction_result(node, construction_result)
            }

            // Flex items contribute flex flow construction results.
            (display::T::flex, float_value, _) => {
                let float_kind = FloatKind::from_property(float_value);
                let construction_result = self.build_flow_for_flex(node, float_kind);
                self.set_flow_construction_result(node, construction_result)
            }

            // Block flows that are not floated contribute block flow construction results.
            //
            // TODO(pcwalton): Make this only trigger for blocks and handle the other `display`
            // properties separately.

            (_, float_value, _) => {
                let float_kind = FloatKind::from_property(float_value);
                let construction_result = self.build_flow_for_block(node, float_kind);
                self.set_flow_construction_result(node, construction_result)
            }
        }

        node.insert_flags(HAS_NEWLY_CONSTRUCTED_FLOW);
        true
    }
}

/// A utility trait with some useful methods for node queries.
trait NodeUtils {
    /// Returns true if this node doesn't render its kids and false otherwise.
    fn is_replaced_content(&self) -> bool;

    fn construction_result_mut<'a>(self, layout_data: &'a mut LayoutDataWrapper)
                                   -> &'a mut ConstructionResult;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(self, result: ConstructionResult);

    /// Replaces the flow construction result in a node with `ConstructionResult::None` and returns
    /// the old value.
    fn swap_out_construction_result(self) -> ConstructionResult;
}

impl<'ln> NodeUtils for ThreadSafeLayoutNode<'ln> {
    fn is_replaced_content(&self) -> bool {
        match self.type_id() {
            None |
            Some(NodeTypeId::CharacterData(_)) |
            Some(NodeTypeId::DocumentType) |
            Some(NodeTypeId::DocumentFragment) |
            Some(NodeTypeId::Document) |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLImageElement))) |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLIFrameElement))) |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLCanvasElement))) => true,
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLObjectElement))) => self.has_object_data(),
            Some(NodeTypeId::Element(_)) => false,
        }
    }

    fn construction_result_mut<'a>(self, layout_data: &'a mut LayoutDataWrapper) -> &'a mut ConstructionResult {
        match self.get_pseudo_element_type() {
            PseudoElementType::Before(_) => &mut layout_data.data.before_flow_construction_result,
            PseudoElementType::After (_) => &mut layout_data.data.after_flow_construction_result,
            PseudoElementType::Normal    => &mut layout_data.data.flow_construction_result,
        }
    }

    #[inline(always)]
    fn set_flow_construction_result(self, result: ConstructionResult) {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        let dst = self.construction_result_mut(layout_data);

        *dst = result;
    }

    #[inline(always)]
    fn swap_out_construction_result(self) -> ConstructionResult {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        self.construction_result_mut(layout_data).swap_out()
    }
}

/// Methods for interacting with HTMLObjectElement nodes
trait ObjectElement<'a> {
    /// Returns None if this node is not matching attributes.
    fn get_type_and_data(&self) -> (Option<&'a str>, Option<&'a str>);

    /// Returns true if this node has object data that is correct uri.
    fn has_object_data(&self) -> bool;

    /// Returns the "data" attribute value parsed as a URL
    fn object_data(&self) -> Option<Url>;
}

impl<'ln> ObjectElement<'ln> for ThreadSafeLayoutNode<'ln> {
    fn get_type_and_data(&self) -> (Option<&'ln str>, Option<&'ln str>) {
        let elem = self.as_element();
        (elem.get_attr(&ns!(""), &atom!("type")), elem.get_attr(&ns!(""), &atom!("data")))
    }

    fn has_object_data(&self) -> bool {
        match self.get_type_and_data() {
            (None, Some(uri)) => is_image_data(uri),
            _ => false
        }
    }

    fn object_data(&self) -> Option<Url> {
        match self.get_type_and_data() {
            (None, Some(uri)) if is_image_data(uri) => Url::parse(uri).ok(),
            _ => None
        }
    }
}

pub trait FlowConstructionUtils {
    /// Adds a new flow as a child of this flow. Removes the flow from the given leaf set if
    /// it's present.
    fn add_new_child(&mut self, new_child: FlowRef);

    /// Finishes a flow. Once a flow is finished, no more child flows or boxes may be added to it.
    /// This will normally run the bubble-inline-sizes (minimum and preferred -- i.e. intrinsic --
    /// inline-size) calculation, unless the global `bubble_inline-sizes_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic inline-
    /// sizes properly computed. (This is not, however, a memory safety problem.)
    fn finish(&mut self);
}

impl FlowConstructionUtils for FlowRef {
    /// Adds a new flow as a child of this flow. Fails if this flow is marked as a leaf.
    ///
    /// This must not be public because only the layout constructor can do this.
    fn add_new_child(&mut self, mut new_child: FlowRef) {
        {
            let kid_base = flow::mut_base(flow_ref::deref_mut(&mut new_child));
            kid_base.parallel.parent = parallel::mut_owned_flow_to_unsafe_flow(self);
        }

        let base = flow::mut_base(flow_ref::deref_mut(self));
        base.children.push_back(new_child);
        let _ = base.parallel.children_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Finishes a flow. Once a flow is finished, no more child flows or fragments may be added to
    /// it. This will normally run the bubble-inline-sizes (minimum and preferred -- i.e. intrinsic
    /// -- inline-size) calculation, unless the global `bubble_inline-sizes_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic inline-sizes
    /// properly computed. (This is not, however, a memory safety problem.)
    ///
    /// This must not be public because only the layout constructor can do this.
    fn finish(&mut self) {
        if !opts::get().bubble_inline_sizes_separately {
            flow_ref::deref_mut(self).bubble_inline_sizes()
        }
    }
}

/// Strips ignorable whitespace from the start of a list of fragments.
pub fn strip_ignorable_whitespace_from_start(this: &mut LinkedList<Fragment>) {
    if this.is_empty() {
        return   // Fast path.
    }

    let mut leading_fragments_consisting_of_solely_bidi_control_characters = LinkedList::new();
    while !this.is_empty() {
        match this.front_mut().as_mut().unwrap().strip_leading_whitespace_if_necessary() {
            WhitespaceStrippingResult::RetainFragment => break,
            WhitespaceStrippingResult::FragmentContainedOnlyBidiControlCharacters => {
                leading_fragments_consisting_of_solely_bidi_control_characters.push_back(
                    this.pop_front().unwrap())
            }
            WhitespaceStrippingResult::FragmentContainedOnlyWhitespace => {
                this.pop_front();
            }
        }
    }
    linked_list::prepend_from(this,
                              &mut leading_fragments_consisting_of_solely_bidi_control_characters)
}

/// Strips ignorable whitespace from the end of a list of fragments.
pub fn strip_ignorable_whitespace_from_end(this: &mut LinkedList<Fragment>) {
    if this.is_empty() {
        return
    }

    let mut trailing_fragments_consisting_of_solely_bidi_control_characters = LinkedList::new();
    while !this.is_empty() {
        match this.back_mut().as_mut().unwrap().strip_trailing_whitespace_if_necessary() {
            WhitespaceStrippingResult::RetainFragment => break,
            WhitespaceStrippingResult::FragmentContainedOnlyBidiControlCharacters => {
                trailing_fragments_consisting_of_solely_bidi_control_characters.push_front(
                    this.pop_back().unwrap())
            }
            WhitespaceStrippingResult::FragmentContainedOnlyWhitespace => {
                this.pop_back();
            }
        }
    }
    this.append(&mut trailing_fragments_consisting_of_solely_bidi_control_characters);
}

/// If the 'unicode-bidi' property has a value other than 'normal', return the bidi control codes
/// to inject before and after the text content of the element.
fn bidi_control_chars(style: &Arc<ComputedValues>) -> Option<(&'static str, &'static str)> {
    use style::computed_values::direction::T::*;
    use style::computed_values::unicode_bidi::T::*;

    let unicode_bidi = style.get_text().unicode_bidi;
    let direction = style.get_inheritedbox().direction;

    // See the table in http://dev.w3.org/csswg/css-writing-modes/#unicode-bidi
    match (unicode_bidi, direction) {
        (normal, _)             => None,
        (embed, ltr)            => Some(("\u{202A}", "\u{202C}")),
        (embed, rtl)            => Some(("\u{202B}", "\u{202C}")),
        (isolate, ltr)          => Some(("\u{2066}", "\u{2069}")),
        (isolate, rtl)          => Some(("\u{2067}", "\u{2069}")),
        (bidi_override, ltr)    => Some(("\u{202D}", "\u{202C}")),
        (bidi_override, rtl)    => Some(("\u{202E}", "\u{202C}")),
        (isolate_override, ltr) => Some(("\u{2068}\u{202D}", "\u{202C}\u{2069}")),
        (isolate_override, rtl) => Some(("\u{2068}\u{202E}", "\u{202C}\u{2069}")),
        (plaintext, _)          => Some(("\u{2068}", "\u{2069}")),
    }
}

fn control_chars_to_fragment(node: &InlineFragmentNodeInfo, text: &str,
                             restyle_damage: RestyleDamage) -> Fragment {
    let info = SpecificFragmentInfo::UnscannedText(
        UnscannedTextFragmentInfo::from_text(String::from(text)));
    Fragment::from_opaque_node_and_style(node.address,
                                         node.pseudo,
                                         node.style.clone(),
                                         restyle_damage,
                                         info)
}
