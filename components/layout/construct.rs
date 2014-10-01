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
//!
//! TODO(pcwalton): There is no incremental reflow yet. This scheme requires that nodes either have
//! weak references to flows or that there be some mechanism to efficiently (O(1) time) "blow
//! apart" a flow tree and have the flows migrate "home" to their respective DOM nodes while we
//! perform flow tree construction. The precise mechanism for this will take some experimentation
//! to get right.

#![deny(unsafe_block)]

use css::node_style::StyledNode;
use block::BlockFlow;
use context::LayoutContext;
use floats::FloatKind;
use flow::{Flow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use flow::{Descendants, AbsDescendants};
use flow;
use flow_ref::FlowRef;
use fragment::{InlineBlockFragment, InlineBlockFragmentInfo, InputFragment};
use fragment::{Fragment, GenericFragment, IframeFragment, IframeFragmentInfo};
use fragment::{ImageFragment, ImageFragmentInfo, SpecificFragmentInfo, TableFragment};
use fragment::{TableCellFragment, TableColumnFragment, TableColumnFragmentInfo};
use fragment::{TableRowFragment, TableWrapperFragment, UnscannedTextFragment};
use fragment::{UnscannedTextFragmentInfo, InputFragmentInfo};
use inline::{InlineFragments, InlineFlow};
use parallel;
use table_wrapper::TableWrapperFlow;
use table::TableFlow;
use table_caption::TableCaptionFlow;
use table_colgroup::TableColGroupFlow;
use table_rowgroup::TableRowGroupFlow;
use table_row::TableRowFlow;
use table_cell::TableCellFlow;
use text::TextRunScanner;
use util::{LayoutDataAccess, OpaqueNodeMethods};
use wrapper::{PostorderNodeMutTraversal, TLayoutNode, ThreadSafeLayoutNode};
use wrapper::{Before, After, Normal};

use gfx::display_list::OpaqueNode;
use script::dom::element::{HTMLIFrameElementTypeId, HTMLImageElementTypeId};
use script::dom::element::{HTMLObjectElementTypeId, HTMLInputElementTypeId};
use script::dom::element::{HTMLTableColElementTypeId, HTMLTableDataCellElementTypeId};
use script::dom::element::{HTMLTableElementTypeId, HTMLTableHeaderCellElementTypeId};
use script::dom::element::{HTMLTableRowElementTypeId, HTMLTableSectionElementTypeId};
use script::dom::node::{CommentNodeTypeId, DoctypeNodeTypeId, DocumentFragmentNodeTypeId};
use script::dom::node::{DocumentNodeTypeId, ElementNodeTypeId, ProcessingInstructionNodeTypeId};
use script::dom::node::{TextNodeTypeId};
use script::dom::htmlobjectelement::is_image_data;
use std::mem;
use std::sync::atomics::Relaxed;
use style::ComputedValues;
use style::computed_values::{display, position, float};
use sync::Arc;
use url::Url;

/// The results of flow construction for a DOM node.
pub enum ConstructionResult {
    /// This node contributes nothing at all (`display: none`). Alternately, this is what newly
    /// created nodes have their `ConstructionResult` set to.
    NoConstructionResult,

    /// This node contributed a flow at the proper position in the tree.
    /// Nothing more needs to be done for this node. It has bubbled up fixed
    /// and absolute descendant flows that have a containing block above it.
    FlowConstructionResult(FlowRef, AbsDescendants),

    /// This node contributed some object or objects that will be needed to construct a proper flow
    /// later up the tree, but these objects have not yet found their home.
    ConstructionItemConstructionResult(ConstructionItem),
}

/// Represents the output of flow construction for a DOM node that has not yet resulted in a
/// complete flow. Construction items bubble up the tree until they find a `Flow` to be attached
/// to.
pub enum ConstructionItem {
    /// Inline fragments and associated {ib} splits that have not yet found flows.
    InlineFragmentsConstructionItem(InlineFragmentsConstructionResult),
    /// Potentially ignorable whitespace.
    WhitespaceConstructionItem(OpaqueNode, Arc<ComputedValues>),
    /// TableColumn Fragment
    TableColumnFragmentConstructionItem(Fragment),
}

/// Represents inline fragments and {ib} splits that are bubbling up from an inline.
pub struct InlineFragmentsConstructionResult {
    /// Any {ib} splits that we're bubbling up.
    pub splits: Vec<InlineBlockSplit>,

    /// Any fragments that succeed the {ib} splits.
    pub fragments: InlineFragments,

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
///     InlineFragmentsConstructionItem(Some(~[
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
pub struct InlineBlockSplit {
    /// The inline fragments that precede the flow.
    pub predecessors: InlineFragments,

    /// The flow that caused this {ib} split.
    pub flow: FlowRef,
}

/// Holds inline fragments that we're gathering for children of an inline node.
struct InlineFragmentsAccumulator {
    /// The list of fragments.
    fragments: InlineFragments,

    /// Whether we've created a range to enclose all the fragments. This will be Some() if the outer node
    /// is an inline and None otherwise.
    enclosing_style: Option<Arc<ComputedValues>>,
}

impl InlineFragmentsAccumulator {
    fn new() -> InlineFragmentsAccumulator {
        InlineFragmentsAccumulator {
            fragments: InlineFragments::new(),
            enclosing_style: None,
        }
    }

    fn from_inline_node(node: &ThreadSafeLayoutNode) -> InlineFragmentsAccumulator {
        let fragments = InlineFragments::new();
        InlineFragmentsAccumulator {
            fragments: fragments,
            enclosing_style: Some(node.style().clone()),
        }
    }

    fn finish(self) -> InlineFragments {
        let InlineFragmentsAccumulator {
            fragments: mut fragments,
            enclosing_style
        } = self;

        match enclosing_style {
            Some(enclosing_style) => {
                for frag in fragments.fragments.iter_mut() {
                    frag.add_inline_context_style(enclosing_style.clone());
                }
            }
            None => {}
        }
        fragments
    }
}

enum WhitespaceStrippingMode {
    NoWhitespaceStripping,
    StripWhitespaceFromStart,
    StripWhitespaceFromEnd,
}

/// An object that knows how to create flows.
pub struct FlowConstructor<'a> {
    /// The layout context.
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> FlowConstructor<'a> {
    /// Creates a new flow constructor.
    pub fn new<'a>(layout_context: &'a LayoutContext<'a>)
               -> FlowConstructor<'a> {
        FlowConstructor {
            layout_context: layout_context,
        }
    }

    /// Builds the `ImageFragmentInfo` for the given image. This is out of line to guide inlining.
    fn build_fragment_info_for_image(&mut self, node: &ThreadSafeLayoutNode, url: Option<Url>)
                                -> SpecificFragmentInfo {
        match url {
            None => GenericFragment,
            Some(url) => {
                // FIXME(pcwalton): The fact that image fragments store the cache within them makes
                // little sense to me.
                ImageFragment(ImageFragmentInfo::new(node,
                                                     url,
                                                     self.layout_context
                                                         .shared
                                                         .image_cache
                                                         .clone()))
            }
        }
    }

    fn build_fragment_info_for_input(&mut self, node: &ThreadSafeLayoutNode) -> SpecificFragmentInfo {
        //FIXME: would it make more sense to use HTMLInputElement::input_type instead of the raw
        //       value? definitely for string comparisons.
        let elem = node.as_element();
        let data = match elem.get_attr(&ns!(""), "type") {
            Some("checkbox") | Some("radio") => None,
            Some("button") | Some("submit") | Some("reset") =>
                Some(node.get_input_value().len() as u32),
            Some("file") => Some(node.get_input_size()),
            _ => Some(node.get_input_size()),
        };
        data.map(|size| InputFragment(InputFragmentInfo { size: size }))
            .unwrap_or(GenericFragment)
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
            Some(ElementNodeTypeId(HTMLIFrameElementTypeId)) => {
                IframeFragment(IframeFragmentInfo::new(node))
            }
            Some(ElementNodeTypeId(HTMLImageElementTypeId)) => {
                self.build_fragment_info_for_image(node, node.image_url())
            }
            Some(ElementNodeTypeId(HTMLInputElementTypeId)) => {
                self.build_fragment_info_for_input(node)
            }
            Some(ElementNodeTypeId(HTMLObjectElementTypeId)) => {
                let data = node.get_object_data();
                self.build_fragment_info_for_image(node, data)
            }
            Some(ElementNodeTypeId(HTMLTableElementTypeId)) => TableWrapperFragment,
            Some(ElementNodeTypeId(HTMLTableColElementTypeId)) => {
                TableColumnFragment(TableColumnFragmentInfo::new(node))
            }
            Some(ElementNodeTypeId(HTMLTableDataCellElementTypeId)) |
            Some(ElementNodeTypeId(HTMLTableHeaderCellElementTypeId)) => TableCellFragment,
            Some(ElementNodeTypeId(HTMLTableRowElementTypeId)) |
            Some(ElementNodeTypeId(HTMLTableSectionElementTypeId)) => TableRowFragment,
            Some(TextNodeTypeId) => UnscannedTextFragment(UnscannedTextFragmentInfo::new(node)),
            _ => {
                // This includes pseudo-elements.
                GenericFragment
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
        let mut fragments = fragment_accumulator.finish();
        if fragments.is_empty() {
            return
        };

        match whitespace_stripping {
            NoWhitespaceStripping => {}
            StripWhitespaceFromStart => {
                fragments.strip_ignorable_whitespace_from_start();
                if fragments.is_empty() {
                    return
                };
            }
            StripWhitespaceFromEnd => {
                fragments.strip_ignorable_whitespace_from_end();
                if fragments.is_empty() {
                    return
                };
            }
        }

        // Build a list of all the inline-block fragments before fragments is moved.
        let mut inline_block_flows = vec!();
        for f in fragments.fragments.iter() {
            match f.specific {
                InlineBlockFragment(ref info) => {
                    inline_block_flows.push(info.flow_ref.clone());
                },
                _ => {}
            }
        }

        let mut inline_flow_ref = FlowRef::new(box InlineFlow::from_fragments((*node).clone(), fragments));

        // Add all the inline-block fragments as children of the inline flow.
        for inline_block_flow in inline_block_flows.iter() {
            inline_flow_ref.add_new_child(inline_block_flow.clone());
        }

        {
            let inline_flow = inline_flow_ref.get_mut().as_inline();
            let (ascent, descent) = inline_flow.compute_minimum_ascent_and_descent(self.layout_context.font_context(), &**node.style());
            inline_flow.minimum_block_size_above_baseline = ascent;
            inline_flow.minimum_depth_below_baseline = descent;
            TextRunScanner::new().scan_for_runs(self.layout_context.font_context(), inline_flow);
        }

        inline_flow_ref.finish(self.layout_context);

        if flow.get().need_anonymous_flow(inline_flow_ref.get()) {
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
            NoConstructionResult => {}
            FlowConstructionResult(kid_flow, kid_abs_descendants) => {
                // If kid_flow is TableCaptionFlow, kid_flow should be added under
                // TableWrapperFlow.
                if flow.get().is_table() && kid_flow.get().is_table_caption() {
                    kid.set_flow_construction_result(FlowConstructionResult(kid_flow,
                                                                            Descendants::new()))
                } else if flow.get().need_anonymous_flow(kid_flow.get()) {
                    consecutive_siblings.push(kid_flow)
                } else {
                    // Flush any inline fragments that we were gathering up. This allows us to handle
                    // {ib} splits.
                    debug!("flushing {} inline box(es) to flow A",
                           inline_fragment_accumulator.fragments.len());
                    self.flush_inline_fragments_to_flow_or_list(
                        mem::replace(inline_fragment_accumulator, InlineFragmentsAccumulator::new()),
                        flow,
                        consecutive_siblings,
                        StripWhitespaceFromStart,
                        node);
                    if !consecutive_siblings.is_empty() {
                        let consecutive_siblings = mem::replace(consecutive_siblings, vec!());
                        self.generate_anonymous_missing_child(consecutive_siblings,
                                                              flow,
                                                              node);
                    }
                    flow.add_new_child(kid_flow);
                }
                abs_descendants.push_descendants(kid_abs_descendants);
            }
            ConstructionItemConstructionResult(InlineFragmentsConstructionItem(
                    InlineFragmentsConstructionResult {
                        splits: splits,
                        fragments: successor_fragments,
                        abs_descendants: kid_abs_descendants,
                    })) => {
                // Add any {ib} splits.
                for split in splits.into_iter() {
                    // Pull apart the {ib} split object and push its predecessor fragments
                    // onto the list.
                    let InlineBlockSplit {
                        predecessors: predecessors,
                        flow: kid_flow
                    } = split;
                    inline_fragment_accumulator.fragments.push_all(predecessors);

                    // If this is the first fragment in flow, then strip ignorable
                    // whitespace per CSS 2.1 § 9.2.1.1.
                    let whitespace_stripping = if *first_fragment {
                        *first_fragment = false;
                        StripWhitespaceFromStart
                    } else {
                        NoWhitespaceStripping
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
                    if flow.get().need_anonymous_flow(kid_flow.get()) {
                        consecutive_siblings.push(kid_flow)
                    } else {
                        flow.add_new_child(kid_flow)
                    }
                }

                // Add the fragments to the list we're maintaining.
                inline_fragment_accumulator.fragments.push_all(successor_fragments);
                abs_descendants.push_descendants(kid_abs_descendants);
            }
            ConstructionItemConstructionResult(WhitespaceConstructionItem(whitespace_node, whitespace_style)) => {
                // Add whitespace results. They will be stripped out later on when
                // between block elements, and retained when between inline elements.
                let fragment_info = UnscannedTextFragment(UnscannedTextFragmentInfo::from_text(" ".to_string()));
                let mut fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                    whitespace_style,
                                                                    fragment_info);
                inline_fragment_accumulator.fragments.push(&mut fragment);
            }
            ConstructionItemConstructionResult(TableColumnFragmentConstructionItem(_)) => {
                // TODO: Implement anonymous table objects for missing parents
                // CSS 2.1 § 17.2.1, step 3-2
            }
        }
    }

    /// Build block flow for current node using information from children nodes.
    ///
    /// Consume results from children and combine them, handling {ib} splits.
    /// Block flows and inline flows thus created will become the children of
    /// this block flow.
    /// Also, deal with the absolute and fixed descendants bubbled up by
    /// children nodes.
    fn build_flow_for_block(&mut self, mut flow: FlowRef, node: &ThreadSafeLayoutNode)
                            -> ConstructionResult {
        // Gather up fragments for the inline flows we might need to create.
        let mut inline_fragment_accumulator = InlineFragmentsAccumulator::new();
        let mut consecutive_siblings = vec!();
        let mut first_fragment = true;

        // Special case: If this is generated content, then we need to initialize the accumulator
        // with the fragment corresponding to that content.
        if node.get_pseudo_element_type() != Normal ||
           node.type_id() == Some(ElementNodeTypeId(HTMLInputElementTypeId)) {
            let fragment_info = UnscannedTextFragment(UnscannedTextFragmentInfo::new(node));
            let mut fragment = Fragment::new_from_specific_info(node, fragment_info);
            inline_fragment_accumulator.fragments.push(&mut fragment);
            first_fragment = false;
        }

        // List of absolute descendants, in tree order.
        let mut abs_descendants = Descendants::new();
        for kid in node.children() {
            if kid.get_pseudo_element_type() != Normal {
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
                                                    StripWhitespaceFromEnd,
                                                    node);
        if !consecutive_siblings.is_empty() {
            self.generate_anonymous_missing_child(consecutive_siblings, &mut flow, node);
        }

        // The flow is done.
        flow.finish(self.layout_context);
        let is_positioned = flow.get_mut().as_block().is_positioned();
        let is_fixed_positioned = flow.get_mut().as_block().is_fixed();
        let is_absolutely_positioned = flow.get_mut().as_block().is_absolutely_positioned();
        if is_positioned {
            // This is the CB for all the absolute descendants.
            flow.set_absolute_descendants(abs_descendants);

            abs_descendants = Descendants::new();

            if is_fixed_positioned || is_absolutely_positioned {
                // This is now the only absolute flow in the subtree which hasn't yet
                // reached its CB.
                abs_descendants.push(flow.clone());
            }
        }
        FlowConstructionResult(flow, abs_descendants)
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
        let mut opt_inline_block_splits: Vec<InlineBlockSplit> = Vec::new();
        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        let mut abs_descendants = Descendants::new();

        // Concatenate all the fragments of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            if kid.get_pseudo_element_type() != Normal {
                self.process(&kid);
            }
            match kid.swap_out_construction_result() {
                NoConstructionResult => {}
                FlowConstructionResult(flow, kid_abs_descendants) => {
                    // {ib} split. Flush the accumulator to our new split and make a new
                    // accumulator to hold any subsequent fragments we come across.
                    let split = InlineBlockSplit {
                        predecessors:
                            mem::replace(
                                &mut fragment_accumulator,
                                InlineFragmentsAccumulator::from_inline_node(node)).finish(),
                        flow: flow,
                    };
                    opt_inline_block_splits.push(split);
                    abs_descendants.push_descendants(kid_abs_descendants);
                }
                ConstructionItemConstructionResult(InlineFragmentsConstructionItem(
                        InlineFragmentsConstructionResult {
                            splits: splits,
                            fragments: successors,
                            abs_descendants: kid_abs_descendants,
                        })) => {

                    // Bubble up {ib} splits.
                    for split in splits.into_iter() {
                        let InlineBlockSplit {
                            predecessors: predecessors,
                            flow: kid_flow
                        } = split;
                        fragment_accumulator.fragments.push_all(predecessors);

                        let split = InlineBlockSplit {
                            predecessors:
                                mem::replace(&mut fragment_accumulator,
                                             InlineFragmentsAccumulator::from_inline_node(node))
                                    .finish(),
                            flow: kid_flow,
                        };
                        opt_inline_block_splits.push(split)
                    }

                    // Push residual fragments.
                    fragment_accumulator.fragments.push_all(successors);
                    abs_descendants.push_descendants(kid_abs_descendants);
                }
                ConstructionItemConstructionResult(WhitespaceConstructionItem(whitespace_node,
                                                                              whitespace_style))
                        => {
                    // Instantiate the whitespace fragment.
                    let fragment_info = UnscannedTextFragment(UnscannedTextFragmentInfo::from_text(" ".to_string()));
                    let mut fragment = Fragment::from_opaque_node_and_style(whitespace_node,
                                                                        whitespace_style,
                                                                        fragment_info);
                    fragment_accumulator.fragments.push(&mut fragment)
                }
                ConstructionItemConstructionResult(TableColumnFragmentConstructionItem(_)) => {
                    // TODO: Implement anonymous table objects for missing parents
                    // CSS 2.1 § 17.2.1, step 3-2
                }
            }
        }

        // Finally, make a new construction result.
        if opt_inline_block_splits.len() > 0 || fragment_accumulator.fragments.len() > 0
                || abs_descendants.len() > 0 {
            let construction_item = InlineFragmentsConstructionItem(
                    InlineFragmentsConstructionResult {
                splits: opt_inline_block_splits,
                fragments: fragment_accumulator.finish(),
                abs_descendants: abs_descendants,
            });
            ConstructionItemConstructionResult(construction_item)
        } else {
            NoConstructionResult
        }
    }

    /// Creates an `InlineFragmentsConstructionResult` for replaced content. Replaced content
    /// doesn't render its children, so this just nukes a child's fragments and creates a
    /// `Fragment`.
    fn build_fragments_for_replaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                               -> ConstructionResult {
        for kid in node.children() {
            kid.set_flow_construction_result(NoConstructionResult)
        }

        // If this node is ignorable whitespace, bail out now.
        //
        // FIXME(#2001, pcwalton): Don't do this if there's padding or borders.
        if node.is_ignorable_whitespace() {
            let opaque_node = OpaqueNodeMethods::from_thread_safe_layout_node(node);
            return ConstructionItemConstructionResult(WhitespaceConstructionItem(
                opaque_node,
                node.style().clone()))
        }

        // If this is generated content, then we need to initialize the accumulator with the
        // fragment corresponding to that content. Otherwise, just initialize with the ordinary
        // fragment that needs to be generated for this inline node.
        let mut fragment = if node.get_pseudo_element_type() != Normal {
            let fragment_info = UnscannedTextFragment(UnscannedTextFragmentInfo::new(node));
            Fragment::new_from_specific_info(node, fragment_info)
        } else {
            Fragment::new(self, node)
        };

        let mut fragments = InlineFragments::new();
        fragments.push(&mut fragment);

        let construction_item = InlineFragmentsConstructionItem(InlineFragmentsConstructionResult {
            splits: Vec::new(),
            fragments: fragments,
            abs_descendants: Descendants::new(),
        });
        ConstructionItemConstructionResult(construction_item)
    }

    fn build_fragment_for_inline_block(&mut self, node: &ThreadSafeLayoutNode)
                                       -> ConstructionResult {
        let block_flow_result = self.build_flow_for_nonfloated_block(node);
        let (block_flow, abs_descendants) = match block_flow_result {
            FlowConstructionResult(block_flow, abs_descendants) => (block_flow, abs_descendants),
            _ => unreachable!()
        };

        let fragment_info = InlineBlockFragment(InlineBlockFragmentInfo::new(block_flow));
        let mut fragment = Fragment::new_from_specific_info(node, fragment_info);

        let mut fragment_accumulator = InlineFragmentsAccumulator::from_inline_node(node);
        fragment_accumulator.fragments.push(&mut fragment);

        let construction_item = InlineFragmentsConstructionItem(InlineFragmentsConstructionResult {
            splits: Vec::new(),
            fragments: fragment_accumulator.finish(),
            abs_descendants: abs_descendants,
        });
        ConstructionItemConstructionResult(construction_item)
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

    /// TableCaptionFlow is populated underneath TableWrapperFlow
    fn place_table_caption_under_table_wrapper(&mut self,
                                               table_wrapper_flow: &mut FlowRef,
                                               node: &ThreadSafeLayoutNode) {
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult | ConstructionItemConstructionResult(_) => {}
                FlowConstructionResult(kid_flow, _) => {
                    // Only kid flows with table-caption are matched here.
                    assert!(kid_flow.get().is_table_caption());
                    table_wrapper_flow.add_new_child(kid_flow);
                }
            }
        }
    }

    /// Generates an anonymous table flow according to CSS 2.1 § 17.2.1, step 2.
    /// If necessary, generate recursively another anonymous table flow.
    fn generate_anonymous_missing_child(&mut self,
                                        child_flows: Vec<FlowRef>,
                                        flow: &mut FlowRef,
                                        node: &ThreadSafeLayoutNode) {
        let mut anonymous_flow = flow.get().generate_missing_child_flow(node);
        let mut consecutive_siblings = vec!();
        for kid_flow in child_flows.into_iter() {
            if anonymous_flow.get().need_anonymous_flow(kid_flow.get()) {
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
        anonymous_flow.finish(self.layout_context);
        flow.add_new_child(anonymous_flow);
    }

    /// Builds a flow for a node with `display: table`. This yields a `TableWrapperFlow` with possibly
    /// other `TableCaptionFlow`s or `TableFlow`s underneath it.
    fn build_flow_for_table_wrapper(&mut self, node: &ThreadSafeLayoutNode,
                                    float_value: float::T) -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, TableWrapperFragment);
        let wrapper_flow = match float_value {
            float::none => box TableWrapperFlow::from_node_and_fragment(node, fragment),
            _ => {
                let float_kind = FloatKind::from_property(float_value);
                box TableWrapperFlow::float_from_node_and_fragment(node, fragment, float_kind)
            }
        };
        let mut wrapper_flow = FlowRef::new(wrapper_flow as Box<Flow>);

        let table_fragment = Fragment::new_from_specific_info(node, TableFragment);
        let table_flow = box TableFlow::from_node_and_fragment(node, table_fragment);
        let table_flow = FlowRef::new(table_flow as Box<Flow>);

        // We first populate the TableFlow with other flows than TableCaptionFlow.
        // We then populate the TableWrapperFlow with TableCaptionFlow, and attach
        // the TableFlow to the TableWrapperFlow
        let construction_result = self.build_flow_for_block(table_flow, node);
        self.place_table_caption_under_table_wrapper(&mut wrapper_flow, node);

        let mut abs_descendants = Descendants::new();
        let mut fixed_descendants = Descendants::new();

        // NOTE: The order of captions and table are not the same order as in the DOM tree.
        // All caption blocks are placed before the table flow
        match construction_result {
            FlowConstructionResult(table_flow, table_abs_descendants) => {
                wrapper_flow.add_new_child(table_flow);
                abs_descendants.push_descendants(table_abs_descendants);
            }
            _ => {}
        }

        // The flow is done.
        wrapper_flow.finish(self.layout_context);
        let is_positioned = wrapper_flow.get_mut().as_block().is_positioned();
        let is_fixed_positioned = wrapper_flow.get_mut().as_block().is_fixed();
        let is_absolutely_positioned = wrapper_flow.get_mut()
                                                   .as_block()
                                                   .is_absolutely_positioned();
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

        FlowConstructionResult(wrapper_flow, abs_descendants)
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
        let fragment = Fragment::new_from_specific_info(node, TableRowFragment);
        let flow = box TableRowGroupFlow::from_node_and_fragment(node, fragment);
        let flow = flow as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-row`. This yields a `TableRowFlow` with
    /// possibly other `TableCellFlow`s underneath it.
    fn build_flow_for_table_row(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, TableRowFragment);
        let flow = box TableRowFlow::from_node_and_fragment(node, fragment) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Builds a flow for a node with `display: table-cell`. This yields a `TableCellFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_cell(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(node, TableCellFragment);
        let flow = box TableCellFlow::from_node_and_fragment(node, fragment) as Box<Flow>;
        self.build_flow_for_block(FlowRef::new(flow), node)
    }

    /// Creates a fragment for a node with `display: table-column`.
    fn build_fragments_for_table_column(&mut self, node: &ThreadSafeLayoutNode)
                                        -> ConstructionResult {
        // CSS 2.1 § 17.2.1. Treat all child fragments of a `table-column` as `display: none`.
        for kid in node.children() {
            kid.set_flow_construction_result(NoConstructionResult)
        }

        let specific = TableColumnFragment(TableColumnFragmentInfo::new(node));
        let construction_item = TableColumnFragmentConstructionItem(
            Fragment::new_from_specific_info(node, specific)
        );
        ConstructionItemConstructionResult(construction_item)
    }

    /// Builds a flow for a node with `display: table-column-group`.
    /// This yields a `TableColGroupFlow`.
    fn build_flow_for_table_colgroup(&mut self, node: &ThreadSafeLayoutNode)
                                     -> ConstructionResult {
        let fragment = Fragment::new_from_specific_info(
            node,
            TableColumnFragment(TableColumnFragmentInfo::new(node)));
        let mut col_fragments = vec!();
        for kid in node.children() {
            // CSS 2.1 § 17.2.1. Treat all non-column child fragments of `table-column-group`
            // as `display: none`.
            match kid.swap_out_construction_result() {
                ConstructionItemConstructionResult(TableColumnFragmentConstructionItem(
                        fragment)) => {
                    col_fragments.push(fragment);
                }
                _ => {}
            }
        }
        if col_fragments.is_empty() {
            debug!("add TableColumnFragment for empty colgroup");
            let specific = TableColumnFragment(TableColumnFragmentInfo::new(node));
            col_fragments.push(Fragment::new_from_specific_info(node, specific));
        }
        let flow = box TableColGroupFlow::from_node_and_fragments(node, fragment, col_fragments);
        let mut flow = FlowRef::new(flow as Box<Flow>);
        flow.finish(self.layout_context);

        FlowConstructionResult(flow, Descendants::new())
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
                    Normal => display::inline,
                    Before(display) => display,
                    After(display) => display,
                };
                (display, style.get_box().float, style.get_box().position)
            }
            Some(ElementNodeTypeId(_)) => {
                let style = node.style();
                (style.get_box().display, style.get_box().float, style.get_box().position)
            }
            Some(TextNodeTypeId) => (display::inline, float::none, position::static_),
            Some(CommentNodeTypeId) |
            Some(DoctypeNodeTypeId) |
            Some(DocumentFragmentNodeTypeId) |
            Some(DocumentNodeTypeId) |
            Some(ProcessingInstructionNodeTypeId) => {
                (display::none, float::none, position::static_)
            }
        };

        debug!("building flow for node: {:?} {:?}", display, float);

        // Switch on display and floatedness.
        match (display, float, positioning) {
            // `display: none` contributes no flow construction result. Nuke the flow construction
            // results of children.
            (display::none, _, _) => {
                for child in node.children() {
                    drop(child.swap_out_construction_result())
                }
            }

            // Table items contribute table flow construction results.
            (display::table, float_value, _) => {
                let construction_result = self.build_flow_for_table_wrapper(node, float_value);
                node.set_flow_construction_result(construction_result)
            }

            // Absolutely positioned elements will have computed value of
            // `float` as 'none' and `display` as per the table.
            // Only match here for block items. If an item is absolutely
            // positioned, but inline we shouldn't try to construct a block
            // flow here - instead, let it match the inline case
            // below.
            (display::block, _, position::absolute) | (_, _, position::fixed) => {
                node.set_flow_construction_result(self.build_flow_for_nonfloated_block(node))
            }

            // Inline items contribute inline fragment construction results.
            //
            // FIXME(pcwalton, #3307): This is not sufficient to handle floated generated content.
            (display::inline, _, _) => {
                let construction_result = self.build_fragments_for_inline(node);
                node.set_flow_construction_result(construction_result)
            }

            // Inline-block items contribute inline fragment construction results.
            (display::inline_block, float::none, _) => {
                let construction_result = self.build_fragment_for_inline_block(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_caption, _, _) => {
                let construction_result = self.build_flow_for_table_caption(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_column_group, _, _) => {
                let construction_result = self.build_flow_for_table_colgroup(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_column, _, _) => {
                let construction_result = self.build_fragments_for_table_column(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_row_group, _, _) | (display::table_header_group, _, _) |
            (display::table_footer_group, _, _) => {
                let construction_result = self.build_flow_for_table_rowgroup(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_row, _, _) => {
                let construction_result = self.build_flow_for_table_row(node);
                node.set_flow_construction_result(construction_result)
            }

            // Table items contribute table flow construction results.
            (display::table_cell, _, _) => {
                let construction_result = self.build_flow_for_table_cell(node);
                node.set_flow_construction_result(construction_result)
            }

            // Block flows that are not floated contribute block flow construction results.
            //
            // TODO(pcwalton): Make this only trigger for blocks and handle the other `display`
            // properties separately.

            (_, float::none, _) => {
                node.set_flow_construction_result(self.build_flow_for_nonfloated_block(node))
            }

            // Floated flows contribute float flow construction results.
            (_, float_value, _) => {
                let float_kind = FloatKind::from_property(float_value);
                node.set_flow_construction_result(
                    self.build_flow_for_floated_block(node, float_kind))
            }
        }

        true
    }
}

/// A utility trait with some useful methods for node queries.
trait NodeUtils {
    /// Returns true if this node doesn't render its kids and false otherwise.
    fn is_replaced_content(&self) -> bool;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(&self, result: ConstructionResult);

    /// Replaces the flow construction result in a node with `NoConstructionResult` and returns the
    /// old value.
    fn swap_out_construction_result(&self) -> ConstructionResult;
}

impl<'ln> NodeUtils for ThreadSafeLayoutNode<'ln> {
    fn is_replaced_content(&self) -> bool {
        match self.type_id() {
            Some(TextNodeTypeId) |
            Some(ProcessingInstructionNodeTypeId) |
            Some(CommentNodeTypeId) |
            Some(DoctypeNodeTypeId) |
            Some(DocumentFragmentNodeTypeId) |
            Some(DocumentNodeTypeId) |
            None |
            Some(ElementNodeTypeId(HTMLImageElementTypeId)) => true,
            Some(ElementNodeTypeId(HTMLObjectElementTypeId)) => self.has_object_data(),
            Some(ElementNodeTypeId(_)) => false,
        }
    }

    #[inline(always)]
    fn set_flow_construction_result(&self, result: ConstructionResult) {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &Some(ref mut layout_data) =>{
                match self.get_pseudo_element_type() {
                    Before(_) => layout_data.data.before_flow_construction_result = result,
                    After(_) => layout_data.data.after_flow_construction_result = result,
                    Normal => layout_data.data.flow_construction_result = result,
                }
            },
            &None => fail!("no layout data"),
        }
    }

    #[inline(always)]
    fn swap_out_construction_result(&self) -> ConstructionResult {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &Some(ref mut layout_data) => {
                match self.get_pseudo_element_type() {
                    Before(_) => {
                        mem::replace(&mut layout_data.data.before_flow_construction_result,
                                     NoConstructionResult)
                    }
                    After(_) => {
                        mem::replace(&mut layout_data.data.after_flow_construction_result,
                                     NoConstructionResult)
                    }
                    Normal => {
                        mem::replace(&mut layout_data.data.flow_construction_result,
                                     NoConstructionResult)
                    }
                }
            }
            &None => fail!("no layout data"),
        }
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
        (elem.get_attr(&ns!(""), "type"), elem.get_attr(&ns!(""), "data"))
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
    /// This will normally run the bubble-inline-sizes (minimum and preferred -- i.e. intrinsic -- inline-size)
    /// calculation, unless the global `bubble_inline-sizes_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic inline-sizes
    /// properly computed. (This is not, however, a memory safety problem.)
    fn finish(&mut self, context: &LayoutContext);
}

impl FlowConstructionUtils for FlowRef {
    /// Adds a new flow as a child of this flow. Fails if this flow is marked as a leaf.
    ///
    /// This must not be public because only the layout constructor can do this.
    fn add_new_child(&mut self, mut new_child: FlowRef) {
        {
            let kid_base = flow::mut_base(new_child.get_mut());
            kid_base.parallel.parent = parallel::mut_owned_flow_to_unsafe_flow(self);
        }

        let base = flow::mut_base(self.get_mut());
        base.children.push_back(new_child);
        let _ = base.parallel.children_count.fetch_add(1, Relaxed);
        let _ = base.parallel.children_and_absolute_descendant_count.fetch_add(1, Relaxed);
    }

    /// Finishes a flow. Once a flow is finished, no more child flows or fragments may be added to
    /// it. This will normally run the bubble-inline-sizes (minimum and preferred -- i.e. intrinsic --
    /// inline-size) calculation, unless the global `bubble_inline-sizes_separately` flag is on.
    ///
    /// All flows must be finished at some point, or they will not have their intrinsic inline-sizes
    /// properly computed. (This is not, however, a memory safety problem.)
    ///
    /// This must not be public because only the layout constructor can do this.
    fn finish(&mut self, context: &LayoutContext) {
        if !context.shared.opts.bubble_inline_sizes_separately {
            self.get_mut().bubble_inline_sizes(context)
        }
    }
}
