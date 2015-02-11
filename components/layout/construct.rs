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

#![deny(unsafe_blocks)]

use css::node_style::StyledNode;
use block::BlockFlow;
use context::LayoutContext;
use floats::FloatKind;
use flow::{Flow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use flow::{Descendants, AbsDescendants};
use flow::{IS_ABSOLUTELY_POSITIONED};
use flow;
use flow_ref::FlowRef;
use fragment::{Fragment, IframeFragmentInfo};
use fragment::ImageFragmentInfo;
use fragment::CanvasFragmentInfo;
use fragment::InlineAbsoluteHypotheticalFragmentInfo;
use fragment::{InlineBlockFragmentInfo, SpecificFragmentInfo};
use fragment::TableColumnFragmentInfo;
use fragment::UnscannedTextFragmentInfo;
use incremental::{RECONSTRUCT_FLOW, RestyleDamage};
use inline::InlineFlow;
use list_item::{self, ListItemFlow};
use parallel;
use table_wrapper::TableWrapperFlow;
use table::TableFlow;
use table_caption::TableCaptionFlow;
use table_colgroup::TableColGroupFlow;
use table_rowgroup::TableRowGroupFlow;
use table_row::TableRowFlow;
use table_cell::TableCellFlow;
use text::TextRunScanner;
use util::{HAS_NEWLY_CONSTRUCTED_FLOW, LayoutDataAccess, OpaqueNodeMethods, LayoutDataWrapper};
use wrapper::{PostorderNodeMutTraversal, PseudoElementType, TLayoutNode, ThreadSafeLayoutNode};

use gfx::display_list::OpaqueNode;
use script::dom::element::ElementTypeId;
use script::dom::htmlelement::HTMLElementTypeId;
use script::dom::htmlobjectelement::is_image_data;
use script::dom::node::NodeTypeId;
use servo_util::opts;
use std::borrow::ToOwned;
use std::collections::DList;
use std::mem;
use std::sync::atomic::Ordering;
use style::computed_values::{caption_side, display, empty_cells, float, list_style_position};
use style::computed_values::{position};
use style::properties::{ComputedValues, make_inline};
use std::sync::Arc;
use url::Url;

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

        (*self).clone()
    }

    pub fn debug_id(&self) -> uint {
        match self {
            &ConstructionResult::None => 0u,
            &ConstructionResult::ConstructionItem(_) => 0u,
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
    pub splits: DList<InlineBlockSplit>,

    /// Any fragments that succeed the {ib} splits.
    pub fragments: DList<Fragment>,

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
    pub predecessors: DList<Fragment>,

    /// The flow that caused this {ib} split.
    pub flow: FlowRef,
}

/// Holds inline fragments that we're gathering for children of an inline node.
struct InlineFragmentsAccumulator {
    /// The list of fragments.
    fragments: DList<Fragment>,

    /// Whether we've created a range to enclose all the fragments. This will be Some() if the
    /// outer node is an inline and None otherwise.
    enclosing_style: Option<Arc<ComputedValues>>,
}

impl InlineFragmentsAccumulator {
    fn new() -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: DList::new(),
            enclosing_style: None,
        }
    }

    fn from_inline_node(node: &ThreadSafeLayoutNode) -> InlineFragmentsAccumulator {
        let fragments = DList::new();
        InlineFragmentsAccumulator {
            fragments: fragments,
            enclosing_style: Some(node.style().clone()),
        }
    }

    fn push_all(&mut self, mut fragments: DList<Fragment>) {
        if fragments.len() == 0 {
            return
        }

        self.fragments.append(&mut fragments)
    }

    fn to_dlist(self) -> DList<Fragment> {
        let InlineFragmentsAccumulator {
            mut fragments,
            enclosing_style
        } = self;

        match enclosing_style {
            Some(enclosing_style) => {
                for frag in fragments.iter_mut() {
                    frag.add_inline_context_style(enclosing_style.clone());
                }
            }
            None => {}
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

    /// Builds the `ImageFragmentInfo` for the given image. This is out of line to guide inlining.
    fn build_fragment_info_for_image(&mut self, node: &ThreadSafeLayoutNode, url: Option<Url>)
                                -> SpecificFragmentInfo {
        match url {
            None => SpecificFragmentInfo::Generic,
            Some(url) => {
                // FIXME(pcwalton): The fact that image fragments store the cache within them makes
                // little sense to me.
                SpecificFragmentInfo::Image(box ImageFragmentInfo::new(node,
                                                         url,
                                                         self.layout_context
                                                             .shared
                                                             .image_cache
                                                             .clone()))
            }
        }
    }

    /// Builds specific `Fragment` info for the given node.
    ///
    /// This does *not* construct the text for generated content (but, for generated content with
    /// `display: block`, it does construct the generic fragment corresponding to the block).
    /// Construction of the text fragment is done specially by `build_flow_using_children()` and
    /// `build_fragments_for_replaced_inline_content()`.
    pub fn build_specific_fragment_info_for_node(&mut self, node: &ThreadSafeLayoutNode)
                                                 -> SpecificFragmentInfo {
        match node.type_id() {
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement))) => {
                SpecificFragmentInfo::Iframe(box IframeFragmentInfo::new(node))
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement))) => {
                self.build_fragment_info_for_image(node, node.image_url())
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement))) => {
                let data = node.get_object_data();
                self.build_fragment_info_for_image(node, data)
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement))) => SpecificFragmentInfo::TableWrapper,
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableColElement))) => {
                SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node))
            }
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_)))) => SpecificFragmentInfo::TableCell,
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement))) |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement))) => SpecificFragmentInfo::TableRow,
            Some(NodeTypeId::Text) => SpecificFragmentInfo::UnscannedText(UnscannedTextFragmentInfo::new(node)),
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement))) => {
                SpecificFragmentInfo::Canvas(box CanvasFragmentInfo::new(node))
            }
            _ => {
                // This includes pseudo-elements.
                SpecificFragmentInfo::Generic
            }
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
        let mut fragments = fragment_accumulator.to_dlist();
        if fragments.is_empty() {
            return
        };

        match whitespace_stripping {
            WhitespaceStrippingMode::None => {}
            WhitespaceStrippingMode::FromStart => {
                strip_ignorable_whitespace_from_start(&mut fragments);
                if fragments.is_empty() {
                    return
                };
            }
            WhitespaceStrippingMode::FromEnd => {
                strip_ignorable_whitespace_from_end(&mut fragments);
                if fragments.is_empty() {
                    return
                };
            }
        }

        // Build a list of all the inline-block fragments before fragments is moved.
        let mut inline_block_flows = vec!();
        for f in fragments.iter() {
            match f.specific {
                SpecificFragmentInfo::InlineBlock(ref info) => inline_block_flows.push(info.flow_ref.clone()),
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref info) => {
                    inline_block_flows.push(info.flow_ref.clone())
                }
                _ => {}
            }
        }

        // We must scan for runs before computing minimum ascent and descent because scanning
        // for runs might collapse so much whitespace away that only hypothetical fragments
        // remain. In that case the inline flow will compute its ascent and descent to be zero.
        let fragments = TextRunScanner::new().scan_for_runs(self.layout_context.font_context(),
                                                            fragments);
        let mut inline_flow_ref =
            FlowRef::new(box InlineFlow::from_fragments(fragments, node.style().writing_mode));

        // Add all the inline-block fragments as children of the inline flow.
        for inline_block_flow in inline_block_flows.iter() {
            inline_flow_ref.add_new_child(inline_block_flow.clone());
        }

        {
            let inline_flow = inline_flow_ref.as_inline();


            let (ascent, descent) =
                inline_flow.compute_minimum_ascent_and_descent(self.layout_context.font_context(),
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
            ConstructionResult::Flow(kid_flow, kid_abs_descendants) => {
                // If kid_flow is TableCaptionFlow, kid_flow should be added under
                // TableWrapperFlow.
                if flow.is_table() && kid_flow.is_table_caption() {
                    kid.set_flow_construction_result(ConstructionResult::Flow(kid_flow,
                                                                            Descendants::new()))
                } else if flow.need_anonymous_flow(&*kid_flow) {
                    consecutive_siblings.push(kid_flow)
                } else {
                    // Flush any inline fragments that we were gathering up. This allows us to
                    // handle {ib} splits.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.len());
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
                           inline_fragment_accumulator.fragments.len());
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
            ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(whitespace_node,
                                                                          whitespace_style,
                                                                          whitespace_damage)) => {
                // Add whitespace results. They will be stripped out later on when
                // between block elements, and retained when between inline elements.
                let fragment_info =
                    SpecificFragmentInfo::UnscannedText(UnscannedTextFragmentInfo::from_text(" ".to_owned()));
                let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                    whitespace_style,
                                                                    whitespace_damage,
                                                                    fragment_info);
                inline_fragment_accumulator.fragments.push_back(fragment);
            }
            ConstructionResult::ConstructionItem(ConstructionItem::TableColumnFragment(_)) => {
                // TODO: Implement anonymous table objects for missing parents
                // CSS 2.1 § 17.2.1, step 3-2
            }
        }
    }

    /// Constructs a block flow, beginning with the given `initial_fragment` if present and then
    /// appending the construction results of children to the child list of the block flow. {ib}
    /// splits and absolutely-positioned descendants are handled correctly.
    fn build_flow_for_block_starting_with_fragment(&mut self,
                                                   mut flow: FlowRef,
                                                   node: &ThreadSafeLayoutNode,
                                                   initial_fragment: Option<Fragment>)
                                                   -> ConstructionResult {
        // Gather up fragments for the inline flows we might need to create.
        let mut inline_fragment_accumulator = InlineFragmentsAccumulator::new();
        let mut consecutive_siblings = vec!();

        let mut first_fragment = match initial_fragment {
            None => true,
            Some(initial_fragment) => {
                inline_fragment_accumulator.fragments.push_back(initial_fragment);
                false
            }
        };

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
        let is_positioned = flow.as_block().is_positioned();
        let is_absolutely_positioned = flow::base(&*flow).flags.contains(IS_ABSOLUTELY_POSITIONED);
        if is_positioned {
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
    /// `build_flow_for_block_starting_with_fragment`. Currently the following kinds of flows get
    /// initial content:
    ///
    /// * Generated content gets the initial content specified by the `content` attribute of the
    ///   CSS.
    /// * `<input>` and `<textarea>` elements get their content.
    ///
    /// FIXME(pcwalton): It is not clear to me that there isn't a cleaner way to handle
    /// `<textarea>`.
    fn build_flow_for_block(&mut self, flow: FlowRef, node: &ThreadSafeLayoutNode)
                            -> ConstructionResult {
        let initial_fragment = if node.get_pseudo_element_type() != PseudoElementType::Normal ||
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement))) ||
           node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))) {
            // A TextArea's text contents are displayed through the input text
            // box, so don't construct them.
            if node.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))) {
                for kid in node.children() {
                    kid.set_flow_construction_result(ConstructionResult::None)
                }
            }
            Some(Fragment::new_from_specific_info(
                    node,
                    SpecificFragmentInfo::UnscannedText(UnscannedTextFragmentInfo::new(node))))
        } else {
            None
        };

        self.build_flow_for_block_starting_with_fragment(flow, node, initial_fragment)
    }

    /// Builds a flow for a node with `display: block`. This yields a `BlockFlow` with possibly
    /// other `BlockFlow`s or `InlineFlow`s underneath it, depending on whether {ib} splits needed
    /// to happen.
    fn build_flow_for_nonfloated_block(&mut self, node: &ThreadSafeLayoutNode)
                                       -> ConstructionResult {
        let flow = box BlockFlow::from_node(self, node) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds the flow for a node with `float: {left|right}`. This yields a float `BlockFlow` with
    /// a `BlockFlow` underneath it.
    fn build_flow_for_floated_block(&mut self, node: &ThreadSafeLayoutNode, float_kind: FloatKind)
                                    -> ConstructionResult {
        let flow = box BlockFlow::float_from_node(self, node, float_kind) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Concatenates the fragments of kids, adding in our own borders/padding/margins if necessary.
    /// Returns the `InlineFragmentsConstructionResult`, if any. There will be no
    /// `InlineFragmentsConstructionResult` if this node consisted entirely of ignorable
    /// whitespace.
    fn build_fragments_for_nonreplaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                                  -> ConstructionResult {
        let mut opt_inline_block_splits: DList<InlineBlockSplit> = DList::new();
        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        let mut abs_descendants = Descendants::new();

        // Concatenate all the fragments of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            if kid.get_pseudo_element_type() != PseudoElementType::Normal {
                self.process(&kid);
            }
            match kid.swap_out_construction_result() {
                ConstructionResult::None => {}
                ConstructionResult::Flow(flow, kid_abs_descendants) => {
                    // {ib} split. Flush the accumulator to our new split and make a new
                    // accumulator to hold any subsequent fragments we come across.
                    let split = InlineBlockSplit {
                        predecessors:
                            mem::replace(
                                &mut fragment_accumulator,
                                InlineFragmentsAccumulator::from_inline_node(node)).to_dlist(),
                        flow: flow,
                    };
                    opt_inline_block_splits.push_back(split);
                    abs_descendants.push_descendants(kid_abs_descendants);
                }
                ConstructionResult::ConstructionItem(ConstructionItem::InlineFragments(
                        InlineFragmentsConstructionResult {
                            splits,
                            fragments: successors,
                            abs_descendants: kid_abs_descendants,
                        })) => {

                    // Bubble up {ib} splits.
                    for split in splits.into_iter() {
                        let InlineBlockSplit {
                            predecessors,
                            flow: kid_flow
                        } = split;
                        fragment_accumulator.push_all(predecessors);

                        let split = InlineBlockSplit {
                            predecessors:
                                mem::replace(&mut fragment_accumulator,
                                             InlineFragmentsAccumulator::from_inline_node(node))
                                    .to_dlist(),
                            flow: kid_flow,
                        };
                        opt_inline_block_splits.push_back(split)
                    }

                    // Push residual fragments.
                    fragment_accumulator.push_all(successors);
                    abs_descendants.push_descendants(kid_abs_descendants);
                }
                ConstructionResult::ConstructionItem(ConstructionItem::Whitespace(
                        whitespace_node,
                        whitespace_style,
                        whitespace_damage)) => {
                    // Instantiate the whitespace fragment.
                    let fragment_info = SpecificFragmentInfo::UnscannedText(UnscannedTextFragmentInfo::from_text(
                            " ".to_owned()));
                    let fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                        whitespace_style,
                                                                        whitespace_damage,
                                                                        fragment_info);
                    fragment_accumulator.fragments.push_back(fragment)
                }
                ConstructionResult::ConstructionItem(ConstructionItem::TableColumnFragment(_)) => {
                    // TODO: Implement anonymous table objects for missing parents
                    // CSS 2.1 § 17.2.1, step 3-2
                }
            }
        }

        // Finally, make a new construction result.
        if opt_inline_block_splits.len() > 0 || fragment_accumulator.fragments.len() > 0
                || abs_descendants.len() > 0 {
            let construction_item = ConstructionItem::InlineFragments(
                    InlineFragmentsConstructionResult {
                splits: opt_inline_block_splits,
                fragments: fragment_accumulator.to_dlist(),
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
            kid.set_flow_construction_result(ConstructionResult::None)
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

        // If the value of `display` property is not `inline`, then we have a situation like
        // `<div style="position:absolute">foo bar baz</div>`. The fragments for `foo`, `bar`, and
        // `baz` had better not be absolutely positioned!
        let mut style = (*node.style()).clone();
        if style.get_box().display != display::T::inline {
            style = Arc::new(make_inline(&*style))
        }

        // If this is generated content, then we need to initialize the accumulator with the
        // fragment corresponding to that content. Otherwise, just initialize with the ordinary
        // fragment that needs to be generated for this inline node.
        let fragment = if node.get_pseudo_element_type() != PseudoElementType::Normal {
            let fragment_info =
                SpecificFragmentInfo::UnscannedText(UnscannedTextFragmentInfo::new(node));
            Fragment::from_opaque_node_and_style(
                OpaqueNodeMethods::from_thread_safe_layout_node(node),
                style,
                node.restyle_damage(),
                fragment_info)
        } else {
            Fragment::from_opaque_node_and_style(
                OpaqueNodeMethods::from_thread_safe_layout_node(node),
                style,
                node.restyle_damage(),
                self.build_specific_fragment_info_for_node(node))
        };

        let mut fragments = DList::new();
        fragments.push_back(fragment);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: DList::new(),
                fragments: fragments,
                abs_descendants: Descendants::new(),
            });
        ConstructionResult::ConstructionItem(construction_item)
    }

    fn build_fragment_for_inline_block(&mut self, node: &ThreadSafeLayoutNode)
                                       -> ConstructionResult {
        let block_flow_result = self.build_flow_for_nonfloated_block(node);
        let (block_flow, abs_descendants) = match block_flow_result {
            ConstructionResult::Flow(block_flow, abs_descendants) => (block_flow, abs_descendants),
            _ => unreachable!()
        };

        let fragment_info = SpecificFragmentInfo::InlineBlock(InlineBlockFragmentInfo::new(
                block_flow));
        let fragment = Fragment::new_from_specific_info(node, fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.push_back(fragment);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: DList::new(),
                fragments: fragment_accumulator.to_dlist(),
                abs_descendants: abs_descendants,
            });
        ConstructionResult::ConstructionItem(construction_item)
    }

    /// This is an annoying case, because the computed `display` value is `block`, but the
    /// hypothetical box is inline.
    fn build_fragment_for_absolutely_positioned_inline(&mut self, node: &ThreadSafeLayoutNode)
                                                       -> ConstructionResult {
        let block_flow_result = self.build_flow_for_nonfloated_block(node);
        let (block_flow, abs_descendants) = match block_flow_result {
            ConstructionResult::Flow(block_flow, abs_descendants) => (block_flow, abs_descendants),
            _ => unreachable!()
        };

        let fragment_info = SpecificFragmentInfo::InlineAbsoluteHypothetical(
            InlineAbsoluteHypotheticalFragmentInfo::new(block_flow));
        let fragment = Fragment::new_from_specific_info(node, fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.push_back(fragment);

        let construction_item =
            ConstructionItem::InlineFragments(InlineFragmentsConstructionResult {
                splits: DList::new(),
                fragments: fragment_accumulator.to_dlist(),
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
        let fragment = Fragment::new_from_specific_info(node, SpecificFragmentInfo::TableWrapper);
        let wrapper_flow = match float_value {
            float::T::none => box TableWrapperFlow::from_node_and_fragment(node, fragment),
            _ => {
                let float_kind = FloatKind::from_property(float_value);
                box TableWrapperFlow::float_from_node_and_fragment(node, fragment, float_kind)
            }
        };
        let mut wrapper_flow = FlowRef::new(wrapper_flow as Box<Flow>);

        let table_fragment = Fragment::new_from_specific_info(node, SpecificFragmentInfo::Table);
        let table_flow = box TableFlow::from_node_and_fragment(node, table_fragment);
        let table_flow = FlowRef::new(table_flow as Box<Flow>);

        // First populate the table flow with its children.
        let construction_result = self.build_flow_for_block(table_flow, node);

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
        let is_positioned = wrapper_flow.as_block().is_positioned();
        let is_fixed_positioned = wrapper_flow.as_block().is_fixed();
        let is_absolutely_positioned = flow::base(&*wrapper_flow).flags.contains(IS_ABSOLUTELY_POSITIONED);
        if is_positioned {
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
        let flow = box TableCaptionFlow::from_node(self, node) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-row-group`. This yields a `TableRowGroupFlow`
    /// with possibly other `TableRowFlow`s underneath it.
    fn build_flow_for_table_rowgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, SpecificFragmentInfo::TableRow);
        let flow = box TableRowGroupFlow::from_node_and_fragment(node, fragment);
        let flow = flow as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-row`. This yields a `TableRowFlow` with
    /// possibly other `TableCellFlow`s underneath it.
    fn build_flow_for_table_row(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, SpecificFragmentInfo::TableRow);
        let flow = box TableRowFlow::from_node_and_fragment(node, fragment) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-cell`. This yields a `TableCellFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_cell(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, SpecificFragmentInfo::TableCell);

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
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: list-item`. This yields a `ListItemFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_list_item(&mut self, node: &ThreadSafeLayoutNode, flotation: float::T)
                                -> ConstructionResult {
        let flotation = match flotation {
            float::T::none => None,
            flotation => Some(FloatKind::from_property(flotation)),
        };
        let marker_fragment = match node.style().get_list().list_style_image {
            Some(ref url) => {
                Some(Fragment::new_from_specific_info(
                    node,
                    self.build_fragment_info_for_image(node, Some((*url).clone()))))
            }
            None => {
                match list_item::static_text_for_list_style_type(node.style()
                                                                     .get_list()
                                                                     .list_style_type) {
                    None => None,
                    Some(text) => {
                        let text = text.to_owned();
                        let mut unscanned_marker_fragments = DList::new();
                        unscanned_marker_fragments.push_back(Fragment::new_from_specific_info(
                            node,
                            SpecificFragmentInfo::UnscannedText(
                                UnscannedTextFragmentInfo::from_text(text))));
                        let marker_fragments = TextRunScanner::new().scan_for_runs(
                            self.layout_context.font_context(),
                            unscanned_marker_fragments);
                        debug_assert!(marker_fragments.len() == 1);
                        marker_fragments.fragments.into_iter().next()
                    }
                }
            }
        };

        // If the list marker is outside, it becomes the special "outside fragment" that list item
        // flows have. If it's inside, it's just a plain old fragment. Note that this means that
        // we adopt Gecko's behavior rather than WebKit's when the marker causes an {ib} split,
        // which has caused some malaise (Bugzilla #36854) but CSS 2.1 § 12.5.1 lets me do it, so
        // there.
        let flow;
        let initial_fragment;
        match node.style().get_list().list_style_position {
            list_style_position::T::outside => {
                flow = box ListItemFlow::from_node_marker_and_flotation(self,
                                                                        node,
                                                                        marker_fragment,
                                                                        flotation);
                initial_fragment = None;
            }
            list_style_position::T::inside => {
                flow = box ListItemFlow::from_node_marker_and_flotation(self,
                                                                        node,
                                                                        None,
                                                                        flotation);
                initial_fragment = marker_fragment;
            }
        }

        self.build_flow_for_block_starting_with_fragment(FlowRef::new(flow as Box<Flow>),
                                                         node,
                                                         initial_fragment)
    }

    /// Creates a fragment for a node with `display: table-column`.
    fn build_fragments_for_table_column(&mut self, node: &ThreadSafeLayoutNode)
                                        -> ConstructionResult {
        // CSS 2.1 § 17.2.1. Treat all child fragments of a `table-column` as `display: none`.
        for kid in node.children() {
            kid.set_flow_construction_result(ConstructionResult::None)
        }

        let specific = SpecificFragmentInfo::TableColumn(TableColumnFragmentInfo::new(node));
        let construction_item = ConstructionItem::TableColumnFragment(
            Fragment::new_from_specific_info(node, specific)
        );
        ConstructionResult::ConstructionItem(construction_item)
    }

    /// Builds a flow for a node with `display: table-column-group`.
    /// This yields a `TableColGroupFlow`.
    fn build_flow_for_table_colgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(
            node,
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
            col_fragments.push(Fragment::new_from_specific_info(node, specific));
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

        match node.swap_out_construction_result() {
            ConstructionResult::None => true,
            ConstructionResult::Flow(mut flow, _) => {
                // The node's flow is of the same type and has the same set of children and can
                // therefore be repaired by simply propagating damage and style to the flow.
                flow::mut_base(&mut *flow).restyle_damage.insert(node.restyle_damage());
                flow.repair_style(node.style());
                true
            }
            ConstructionResult::ConstructionItem(_) => {
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
            Some(NodeTypeId::Text) => (display::T::inline, float::T::none, position::T::static_),
            Some(NodeTypeId::Comment) |
            Some(NodeTypeId::DocumentType) |
            Some(NodeTypeId::DocumentFragment) |
            Some(NodeTypeId::Document) |
            Some(NodeTypeId::ProcessingInstruction) => {
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
                    drop(child.swap_out_construction_result())
                }
            }

            // Table items contribute table flow construction results.
            (display::T::table, float_value, _) => {
                let construction_result = self.build_flow_for_table_wrapper(node, float_value);
                node.set_flow_construction_result(construction_result)
            }

            // Absolutely positioned elements will have computed value of
            // `float` as 'none' and `display` as per the table.
            // Only match here for block items. If an item is absolutely
            // positioned, but inline we shouldn't try to construct a block
            // flow here - instead, let it match the inline case
            // below.
            (display::T::block, _, position::T::absolute) |
            (_, _, position::T::fixed) => {
                node.set_flow_construction_result(self.build_flow_for_nonfloated_block(node))
            }

            // List items contribute their own special flows.
            (display::T::list_item, float_value, _) => {
                node.set_flow_construction_result(self.build_flow_for_list_item(node,
                                                                                float_value))
            }

            // Inline items that are absolutely-positioned contribute inline fragment construction
            // results with a hypothetical fragment.
            (display::T::inline, _, position::T::absolute) => {
                let construction_result =
                    self.build_fragment_for_absolutely_positioned_inline(node);
                node.set_flow_construction_result(construction_result)
            }

            // Inline items contribute inline fragment construction results.
            //
            // FIXME(pcwalton, #3307): This is not sufficient to handle floated generated content.
            (display::T::inline, float::T::none, _) => {
                let construction_result = self.build_fragments_for_inline(node);
                node.set_flow_construction_result(construction_result)
            }

            // Inline-block items contribute inline fragment construction results.
            (display::T::inline_block, float::T::none, _) => {
                let construction_result = self.build_fragment_for_inline_block(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_caption, _, _) => {
                let construction_result = self.build_flow_for_table_caption(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_column_group, _, _) => {
                let construction_result = self.build_flow_for_table_colgroup(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_column, _, _) => {
                let construction_result = self.build_fragments_for_table_column(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_row_group, _, _) |
            (display::T::table_header_group, _, _) |
            (display::T::table_footer_group, _, _) => {
                let construction_result = self.build_flow_for_table_rowgroup(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_row, _, _) => {
                let construction_result = self.build_flow_for_table_row(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::T::table_cell, _, _) => {
                let construction_result = self.build_flow_for_table_cell(node);
                node.set_flow_construction_result(construction_result)
            }

            // Block flows that are not floated contribute block flow construction results.
            //
            // TODO(pcwalton): Make this only trigger for blocks and handle the other `display`
            // properties separately.

            (_, float::T::none, _) => {
                node.set_flow_construction_result(self.build_flow_for_nonfloated_block(node))
            }

            // Floated flows contribute float flow construction results.
            (_, float_value, _) => {
                let float_kind = FloatKind::from_property(float_value);
                node.set_flow_construction_result(
                    self.build_flow_for_floated_block(node, float_kind))
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

    fn get_construction_result<'a>(self, layout_data: &'a mut LayoutDataWrapper) -> &'a mut ConstructionResult;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(self, result: ConstructionResult);

    /// Replaces the flow construction result in a node with `ConstructionResult::None` and returns the
    /// old value.
    fn swap_out_construction_result(self) -> ConstructionResult;
}

impl<'ln> NodeUtils for ThreadSafeLayoutNode<'ln> {
    fn is_replaced_content(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Text) |
            Some(NodeTypeId::ProcessingInstruction) |
            Some(NodeTypeId::Comment) |
            Some(NodeTypeId::DocumentType) |
            Some(NodeTypeId::DocumentFragment) |
            Some(NodeTypeId::Document) |
            None |
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement))) => true,
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement))) => self.has_object_data(),
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement))) => true,
            Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement))) => true,
            Some(NodeTypeId::Element(_)) => false,
        }
    }

    fn get_construction_result<'a>(self, layout_data: &'a mut LayoutDataWrapper) -> &'a mut ConstructionResult {
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

        let dst = self.get_construction_result(layout_data);

        *dst = result;
    }

    #[inline(always)]
    fn swap_out_construction_result(self) -> ConstructionResult {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        self.get_construction_result(layout_data).swap_out()
    }
}

/// Methods for interacting with HTMLObjectElement nodes
trait ObjectElement<'a> {
    /// Returns None if this node is not matching attributes.
    fn get_type_and_data(&self) -> (Option<&'a str>, Option<&'a str>);

    /// Returns true if this node has object data that is correct uri.
    fn has_object_data(&self) -> bool;

    /// Returns the "data" attribute value parsed as a URL
    fn get_object_data(&self) -> Option<Url>;
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

    fn get_object_data(&self) -> Option<Url> {
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
        let base = flow::mut_base(&mut **self);

        {
            let kid_base = flow::mut_base(&mut *new_child);
            kid_base.parallel.parent = parallel::mut_owned_flow_to_unsafe_flow(self);
        }

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
pub fn strip_ignorable_whitespace_from_start(this: &mut DList<Fragment>) {
    if this.is_empty() {
        return   // Fast path.
    }

    while !this.is_empty() && this.front().as_ref().unwrap().is_ignorable_whitespace() {
        debug!("stripping ignorable whitespace from start");
        drop(this.pop_front());
    }
}

/// Strips ignorable whitespace from the end of a list of fragments.
pub fn strip_ignorable_whitespace_from_end(this: &mut DList<Fragment>) {
    if this.is_empty() {
        return
    }

    while !this.is_empty() && this.back().as_ref().unwrap().is_ignorable_whitespace() {
        debug!("stripping ignorable whitespace from end");
        drop(this.pop_back());
    }
}

