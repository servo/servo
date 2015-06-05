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
use css::node_style::StyledNode;
use data::{HAS_NEWLY_CONSTRUCTED_FLOW, LayoutDataAccess, LayoutDataWrapper};
use floats::FloatKind;
use flow::{Descendants, AbsDescendants};
use flow::{Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow::{IS_ABSOLUTELY_POSITIONED};
use flow;
use flow_ref::FlowRef;
use fragment::{Fragment, GeneratedContentInfo, IframeFragmentInfo};
use fragment::{CanvasFragmentInfo, ImageFragmentInfo, InlineAbsoluteFragmentInfo};
use fragment::{InlineAbsoluteHypotheticalFragmentInfo, TableColumnFragmentInfo};
use fragment::{InlineBlockFragmentInfo, SpecificFragmentInfo, UnscannedTextFragmentInfo};
use incremental::{RECONSTRUCT_FLOW, RestyleDamage};
use inline::{InlineFlow, InlineFragmentNodeInfo};
use list_item::{ListItemFlow, ListStyleTypeContent};
use multicol::MulticolFlow;
use opaque_node::OpaqueNodeMethods;
use parallel;
use table::TableFlow;
use table_caption::TableCaptionFlow;
use table_cell::TableCellFlow;
use table_colgroup::TableColGroupFlow;
use table_row::TableRowFlow;
use table_rowgroup::TableRowGroupFlow;
use table_wrapper::TableWrapperFlow;
use text::TextRunScanner;
use wrapper::{PostorderNodeMutTraversal, PseudoElementType, TLayoutNode, ThreadSafeLayoutNode};

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
    Flow(FlowRef, AbsDescendants),

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
    Whitespace(OpaqueNode, Arc<ComputedValues>, RestyleDamage),
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

    /// Any absolute descendants that we're bubbling up.
    pub abs_descendants: AbsDescendants,
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
    pub absolute_descendants: AbsDescendants,
}

impl IntermediateInlineFragments {
    fn new() -> IntermediateInlineFragments {
        IntermediateInlineFragments {
            fragments: LinkedList::new(),
            absolute_descendants: Descendants::new(),
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
}

impl InlineFragmentsAccumulator {
    fn new() -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: IntermediateInlineFragments::new(),
            enclosing_node: None,
        }
    }

    fn from_inline_node(node: &ThreadSafeLayoutNode) -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: IntermediateInlineFragments::new(),
            enclosing_node: Some(InlineFragmentNodeInfo {
                address: OpaqueNodeMethods::from_thread_safe_layout_node(node),
                style: node.style().clone(),
            }),
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
        }
        fragments
    }
}

enum WhitespaceStrippingMode {
    None,
    FromStart,
    FromEnd,
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
            let fragment = Fragment::new(child_node, SpecificFragmentInfo::TableRow);
            let mut new_child = FlowRef::new(box TableRowFlow::from_node_and_fragment(child_node,
                                                                                      fragment));
            new_child.add_new_child(child.clone());
            child.finish();
            *child = new_child
        }
        if child.is_table_row() || child.is_table_rowgroup() {
            let fragment = Fragment::new(child_node, SpecificFragmentInfo::Table);
            let mut new_child = FlowRef::new(box TableFlow::from_node_and_fragment(child_node,
                                                                                   fragment));
            new_child.add_new_child(child.clone());
            child.finish();
            *child = new_child
        }
        if child.is_table() {
            let fragment = Fragment::new(child_node, SpecificFragmentInfo::TableWrapper);
            let mut new_child =
                FlowRef::new(box TableWrapperFlow::from_node_and_fragment(child_node,
                                                                          fragment,
                                                                          None));
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
                                              whitespace_stripping: WhitespaceStrippingMode,
                                              node: &ThreadSafeLayoutNode) {
        let mut fragments = fragment_accumulator.to_intermediate_inline_fragments();
        if fragments.is_empty() {
            return
        };

        match whitespace_stripping {
            WhitespaceStrippingMode::None => {}
            WhitespaceStrippingMode::FromStart => {
                strip_ignorable_whitespace_from_start(&mut fragments.fragments);
                if fragments.is_empty() {
                    return
                };
            }
            WhitespaceStrippingMode::FromEnd => {
                strip_ignorable_whitespace_from_end(&mut fragments.fragments);
                if fragments.is_empty() {
                    return
                };
            }
        }

        // Build a list of all the inline-block fragments before fragments is moved.
        let mut inline_block_flows = vec!();
        for fragment in fragments.fragments.iter() {
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
        let mut inline_flow_ref =
            FlowRef::new(box InlineFlow::from_fragments(scanned_fragments,
                                                        node.style().writing_mode));

        // Add all the inline-block fragments as children of the inline flow.
        for inline_block_flow in inline_block_flows.iter() {
            inline_flow_ref.add_new_child(inline_block_flow.clone());
        }

        // Set up absolute descendants as necessary.
        let contains_positioned_fragments = inline_flow_ref.contains_positioned_fragments();
        if contains_positioned_fragments {
            // This is the containing block for all the absolute descendants.
            inline_flow_ref.set_absolute_descendants(fragments.absolute_descendants);
        }

        {
            let inline_flow = inline_flow_ref.as_inline();


            let (ascent, descent) =
                inline_flow.compute_minimum_ascent_and_descent(&mut self.layout_context.font_context(),
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

    fn build_block_flow_using_construction_result_of_child(&mut self,
                                                           flow: &mut FlowRef,
                                                           consecutive_siblings: &mut Vec<FlowRef>,
                                                           node: &ThreadSafeLayoutNode,
                                                           kid: ThreadSafeLayoutNode,
                                                           inline_fragment_accumulator:
                                                           &mut InlineFragmentsAccumulator,
                                                           abs_descendants: &mut Descendants,
                                                           first_fragment: &mut bool) {
        match kid.swap_out_construction_result() {
            ConstructionResult::None => {}
            ConstructionResult::Flow(mut kid_flow, kid_abs_descendants) => {
                // If kid_flow is TableCaptionFlow, kid_flow should be added under
                // TableWrapperFlow.
                if flow.is_table() && kid_flow.is_table_caption() {
                    self.set_flow_construction_result(&kid,
                                                      ConstructionResult::Flow(kid_flow,
                                                                               Descendants::new()))
                } else if flow.need_anonymous_flow(&*kid_flow) {
                    consecutive_siblings.push(kid_flow)
                } else {
                    // Flush any inline fragments that we were gathering up. This allows us to
                    // handle {ib} splits.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.fragments.len());
                    self.flush_inline_fragments_to_flow_or_list(
                        mem::replace(inline_fragment_accumulator,
                                     InlineFragmentsAccumulator::new()),
                        flow,
                        consecutive_siblings,
                        WhitespaceStrippingMode::FromStart,
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
                        abs_descendants: kid_abs_descendants,
                    })) => {
                // Add any {ib} splits.
                for split in splits.into_iter() {
                    // Pull apart the {ib} split object and push its predecessor fragments
                    // onto the list.
                    let InlineBlockSplit {
                        predecessors,
                        flow: kid_flow
                    } = split;
                    inline_fragment_accumulator.push_all(predecessors);

                    // If this is the first fragment in flow, then strip ignorable
                    // whitespace per CSS 2.1 § 9.2.1.1.
                    let whitespace_stripping = if *first_fragment {
                        *first_fragment = false;
                        WhitespaceStrippingMode::FromStart
                    } else {
                        WhitespaceStrippingMode::None
                    };

                    // Flush any inline fragments that we were gathering up.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.fragments.len());
                    self.flush_inline_fragments_to_flow_or_list(
                            mem::replace(inline_fragment_accumulator,
                                         InlineFragmentsAccumulator::new()),
                            flow,
                            consecutive_siblings,
                            whitespace_stripping,
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
                abs_descendants.push_descendants(kid_abs_descendants);
            }
            ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                    whitespace_node,
                    mut whitespace_style,
                    whitespace_damage)) => {
                // Add whitespace results. They will be stripped out later on when
                // between block elements, and retained when between inline elements.
                let fragment_info = SpecificFragmentInfo::UnscannedText(
                    UnscannedTextFragmentInfo::from_text(" ".to_owned()));
                properties::modify_style_for_replaced_content(&mut whitespace_style);
                let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
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
        let mut first_fragment = inline_fragment_accumulator.fragments.is_empty();

        // List of absolute descendants, in tree order.
        let mut abs_descendants = Descendants::new();
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
                &mut abs_descendants,
                &mut first_fragment);
        }

        // Perform a final flush of any inline fragments that we were gathering up to handle {ib}
        // splits, after stripping ignorable whitespace.
        self.flush_inline_fragments_to_flow_or_list(inline_fragment_accumulator,
                                                    &mut flow,
                                                    &mut consecutive_siblings,
                                                    WhitespaceStrippingMode::FromEnd,
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

            abs_descendants = Descendants::new();
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
        if node.get_pseudo_element_type() != PseudoElementType::Normal ||
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                       HTMLElementTypeId::HTMLInputElement))) ||
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                       HTMLElementTypeId::HTMLTextAreaElement))) {
            // A TextArea's text contents are displayed through the input text
            // box, so don't construct them.
            if node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(
                        HTMLElementTypeId::HTMLTextAreaElement))) {
                for kid in node.children() {
                    self.set_flow_construction_result(&kid, ConstructionResult::None)
                }
            }

            self.create_fragments_for_node_text_content(&mut initial_fragments,
                                                        node,
                                                        node.style());
        }

        self.build_flow_for_block_starting_with_fragments(flow, node, initial_fragments)
    }

    /// Pushes fragments appropriate for the content of the given node onto the given list.
    fn create_fragments_for_node_text_content(&self,
                                              fragments: &mut IntermediateInlineFragments,
                                              node: &ThreadSafeLayoutNode,
                                              style: &Arc<ComputedValues>) {
        for content_item in node.text_content().into_iter() {
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

            let opaque_node = OpaqueNodeMethods::from_thread_safe_layout_node(node);
            fragments.fragments
                     .push_back(Fragment::from_opaque_node_and_style(opaque_node,
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
        let flow = if node.style().is_multicol() {
            box MulticolFlow::from_node_and_fragment(node, fragment, float_kind) as Box<Flow>
        } else {
            box BlockFlow::from_node_and_fragment(node, fragment, float_kind) as Box<Flow>
        };
        self.build_flow_for_block_like(FlowRef::new(flow), node)
    }

    /// Bubbles up {ib} splits.
    fn accumulate_inline_block_splits(&mut self,
                                      splits: LinkedList<InlineBlockSplit>,
                                      node: &ThreadSafeLayoutNode,
                                      fragment_accumulator: &mut InlineFragmentsAccumulator,
                                      opt_inline_block_splits: &mut LinkedList<InlineBlockSplit>) {
        for split in splits.into_iter() {
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
        let mut abs_descendants = Descendants::new();

        // Concatenate all the fragments of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            if kid.get_pseudo_element_type() != PseudoElementType::Normal {
                self.process(&kid);
            }
            match kid.swap_out_construction_result() {
                ConstructionResult::None => {}
                ConstructionResult::Flow(mut flow, kid_abs_descendants) => {
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
                        let kid_style = flow.as_block().fragment.style.clone();
                        let kid_restyle_damage = flow.as_block().fragment.restyle_damage;
                        let fragment_info = SpecificFragmentInfo::InlineAbsolute(
                            InlineAbsoluteFragmentInfo::new(flow));
                        fragment_accumulator.push(Fragment::from_opaque_node_and_style(
                                kid_node,
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
                            abs_descendants: kid_abs_descendants,
                        })) => {

                    // Bubble up {ib} splits.
                    self.accumulate_inline_block_splits(splits,
                                                        node,
                                                        &mut fragment_accumulator,
                                                        &mut opt_inline_block_splits);

                    // Push residual fragments.
                    fragment_accumulator.push_all(successors);
                    abs_descendants.push_descendants(kid_abs_descendants);
                }
                ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                        whitespace_node,
                        mut whitespace_style,
                        whitespace_damage)) => {
                    // Instantiate the whitespace fragment.
                    let fragment_info = SpecificFragmentInfo::UnscannedText(
                        UnscannedTextFragmentInfo::from_text(" ".to_owned()));
                    properties::modify_style_for_replaced_content(&mut whitespace_style);
                    let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
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
            let construction_item = ConstructionItem::InlineFragments(
                    InlineFragmentsConstructionResult {
                splits: opt_inline_block_splits,
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
                abs_descendants: abs_descendants,
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
            let opaque_node = OpaqueNodeMethods::from_thread_safe_layout_node(node);
            return ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                opaque_node,
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
                abs_descendants: Descendants::new(),
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

        let fragment_info = SpecificFragmentInfo::InlineBlock(InlineBlockFragmentInfo::new(
                block_flow));
        let fragment = Fragment::new(node, fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.fragments.push_back(fragment);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: LinkedList::new(),
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
                abs_descendants: abs_descendants,
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
        let fragment = Fragment::new(node, fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.fragments.push_back(fragment);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: LinkedList::new(),
                fragments: fragment_accumulator.to_intermediate_inline_fragments(),
                abs_descendants: abs_descendants,
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
                ConstructionResult::Flow(mut kid_flow, _) => {
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
        for kid_flow in child_flows.into_iter() {
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
        let wrapper_flow = box TableWrapperFlow::from_node_and_fragment(
            node, fragment, FloatKind::from_property(float_value));
        let mut wrapper_flow = FlowRef::new(wrapper_flow as Box<Flow>);

        let table_fragment = Fragment::new(node, SpecificFragmentInfo::Table);
        let table_flow = box TableFlow::from_node_and_fragment(node, table_fragment);
        let table_flow = FlowRef::new(table_flow as Box<Flow>);

        // First populate the table flow with its children.
        let construction_result = self.build_flow_for_block_like(table_flow, node);

        let mut abs_descendants = Descendants::new();
        let mut fixed_descendants = Descendants::new();

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

            abs_descendants = Descendants::new();

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
        let flow = box TableCaptionFlow::from_node_and_fragment(node, fragment) as Box<Flow>;
        self.build_flow_for_block_like(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-row-group`. This yields a `TableRowGroupFlow`
    /// with possibly other `TableRowFlow`s underneath it.
    fn build_flow_for_table_rowgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableRow);
        let flow = box TableRowGroupFlow::from_node_and_fragment(node, fragment);
        let flow = flow as Box<Flow>;
        self.build_flow_for_block_like(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-row`. This yields a `TableRowFlow` with
    /// possibly other `TableCellFlow`s underneath it.
    fn build_flow_for_table_row(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new(node, SpecificFragmentInfo::TableRow);
        let flow = box TableRowFlow::from_node_and_fragment(node, fragment) as Box<Flow>;
        self.build_flow_for_block_like(FlowRef::new(flow), node)
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

        let flow = box TableCellFlow::from_node_fragment_and_visibility_flag(node, fragment, !hide)
            as Box<Flow>;
        self.build_flow_for_block_like(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: list-item`. This yields a `ListItemFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_list_item(&mut self, node: &ThreadSafeLayoutNode, flotation: float::T)
                                -> ConstructionResult {
        let flotation = FloatKind::from_property(flotation);
        let marker_fragment = match node.style().get_list().list_style_image {
            Some(ref url) => {
                let image_info = box ImageFragmentInfo::new(node,
                                                            Some((*url).clone()),
                                                            &self.layout_context);
                Some(Fragment::new(node, SpecificFragmentInfo::Image(image_info)))
            }
            None => {
                match ListStyleTypeContent::from_list_style_type(node.style()
                                                                     .get_list()
                                                                     .list_style_type) {
                    ListStyleTypeContent::None => None,
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
                        debug_assert!(marker_fragments.len() == 1);
                        marker_fragments.fragments.into_iter().next()
                    }
                    ListStyleTypeContent::GeneratedContent(info) => {
                        Some(Fragment::new(node, SpecificFragmentInfo::GeneratedContent(info)))
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
                box ListItemFlow::from_node_fragments_and_flotation(node,
                                                                    main_fragment,
                                                                    marker_fragment,
                                                                    flotation)
            }
            list_style_position::T::inside => {
                if let Some(marker_fragment) = marker_fragment {
                    initial_fragments.fragments.push_back(marker_fragment)
                }
                box ListItemFlow::from_node_fragments_and_flotation(node,
                                                                    main_fragment,
                                                                    None,
                                                                    flotation)
            }
        };

        self.build_flow_for_block_starting_with_fragments(FlowRef::new(flow as Box<Flow>),
                                                          node,
                                                          initial_fragments)
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
        let flow = box TableColGroupFlow::from_node_and_fragments(node, fragment, col_fragments);
        let mut flow = FlowRef::new(flow as Box<Flow>);
        flow.finish();

        ConstructionResult::Flow(flow, Descendants::new())
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

        let mut layout_data_ref = node.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");
        let style = (*node.get_style(&layout_data)).clone();
        let damage = layout_data.data.restyle_damage;
        match node.construction_result_mut(layout_data) {
            &mut ConstructionResult::None => true,
            &mut ConstructionResult::Flow(ref mut flow, _) => {
                // The node's flow is of the same type and has the same set of children and can
                // therefore be repaired by simply propagating damage and style to the flow.
                if !flow.is_block_flow() {
                    return false
                }
                flow::mut_base(&mut **flow).restyle_damage.insert(damage);
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
                    match fragment.specific {
                        SpecificFragmentInfo::InlineBlock(ref mut inline_block_fragment) => {
                            flow::mut_base(&mut *inline_block_fragment.flow_ref).restyle_damage
                                                                                .insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            inline_block_fragment.flow_ref
                                                 .repair_style_and_bubble_inline_sizes(&style);
                        }
                        SpecificFragmentInfo::InlineAbsoluteHypothetical(
                                ref mut inline_absolute_hypothetical_fragment) => {
                            flow::mut_base(&mut *inline_absolute_hypothetical_fragment.flow_ref)
                                .restyle_damage.insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            inline_absolute_hypothetical_fragment
                                .flow_ref
                                .repair_style_and_bubble_inline_sizes(&style);
                        }
                        SpecificFragmentInfo::InlineAbsolute(ref mut inline_absolute_fragment) => {
                            flow::mut_base(&mut *inline_absolute_fragment.flow_ref).restyle_damage
                                                                                   .insert(damage);
                            // FIXME(pcwalton): Fragment restyle damage too?
                            inline_absolute_fragment.flow_ref
                                                    .repair_style_and_bubble_inline_sizes(&style);
                        }
                        _ => {
                            fragment.repair_style(&style);
                            return true
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

        debug!("building flow for node: {:?} {:?} {:?}", display, float, node.type_id());

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
            let kid_base = flow::mut_base(&mut *new_child);
            kid_base.parallel.parent = parallel::mut_owned_flow_to_unsafe_flow(self);
        }

        let base = flow::mut_base(&mut **self);
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
            self.bubble_inline_sizes()
        }
    }
}

/// Strips ignorable whitespace from the start of a list of fragments.
pub fn strip_ignorable_whitespace_from_start(this: &mut LinkedList<Fragment>) {
    if this.is_empty() {
        return   // Fast path.
    }

    while !this.is_empty() && this.front().as_ref().unwrap().is_ignorable_whitespace() {
        debug!("stripping ignorable whitespace from start");
        drop(this.pop_front());
    }
}

/// Strips ignorable whitespace from the end of a list of fragments.
pub fn strip_ignorable_whitespace_from_end(this: &mut LinkedList<Fragment>) {
    if this.is_empty() {
        return
    }

    while !this.is_empty() && this.back().as_ref().unwrap().is_ignorable_whitespace() {
        debug!("stripping ignorable whitespace from end");
        drop(this.pop_back());
    }
}
