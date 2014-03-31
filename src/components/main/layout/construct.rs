/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Creates flows and boxes from a DOM tree via a bottom-up, incremental traversal of the DOM.
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
//!
//! TODO(pcwalton): This scheme should be amenable to parallelization, but, of course, that's not
//! yet implemented.

use css::node_style::StyledNode;
use layout::block::BlockFlow;
use layout::box_::{Box, GenericBox, IframeBox, IframeBoxInfo, ImageBox, ImageBoxInfo, TableBox};
use layout::box_::{TableCellBox, TableColumnBox, TableColumnBoxInfo, TableRowBox, TableWrapperBox};
use layout::box_::{InlineInfo, InlineParentInfo, SpecificBoxInfo, UnscannedTextBox};
use layout::box_::{UnscannedTextBoxInfo};
use layout::context::LayoutContext;
use layout::floats::FloatKind;
use layout::flow::{Flow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow::{Descendants, AbsDescendants, FixedDescendants};
use layout::flow_list::{Rawlink};
use layout::inline::InlineFlow;
use layout::table_wrapper::TableWrapperFlow;
use layout::table::TableFlow;
use layout::table_caption::TableCaptionFlow;
use layout::table_colgroup::TableColGroupFlow;
use layout::table_rowgroup::TableRowGroupFlow;
use layout::table_row::TableRowFlow;
use layout::table_cell::TableCellFlow;
use layout::text::TextRunScanner;
use layout::util::{LayoutDataAccess, OpaqueNode};
use layout::wrapper::{PostorderNodeMutTraversal, TLayoutNode, ThreadSafeLayoutNode};

use gfx::font_context::FontContext;
use script::dom::bindings::codegen::InheritTypes::TextCast;
use script::dom::bindings::js::JS;
use script::dom::element::{HTMLIFrameElementTypeId, HTMLImageElementTypeId, HTMLObjectElementTypeId};
use script::dom::element::{HTMLTableElementTypeId, HTMLTableSectionElementTypeId};
use script::dom::element::{HTMLTableDataCellElementTypeId, HTMLTableHeaderCellElementTypeId};
use script::dom::element::{HTMLTableColElementTypeId, HTMLTableRowElementTypeId};
use script::dom::node::{CommentNodeTypeId, DoctypeNodeTypeId, DocumentFragmentNodeTypeId};
use script::dom::node::{DocumentNodeTypeId, ElementNodeTypeId, ProcessingInstructionNodeTypeId};
use script::dom::node::{TextNodeTypeId};
use script::dom::text::Text;
use style::computed_values::{display, position, float, white_space};
use style::ComputedValues;
use servo_util::namespace;
use servo_util::url::parse_url;
use servo_util::url::is_image_data;
use servo_util::str::is_whitespace;

use extra::url::Url;
use sync::Arc;
use std::mem;
use std::num::Zero;

/// The results of flow construction for a DOM node.
pub enum ConstructionResult {
    /// This node contributes nothing at all (`display: none`). Alternately, this is what newly
    /// created nodes have their `ConstructionResult` set to.
    NoConstructionResult,

    /// This node contributed a flow at the proper position in the tree.
    /// Nothing more needs to be done for this node. It has bubbled up fixed
    /// and absolute descendant flows that have a CB above it.
    FlowConstructionResult(~Flow, AbsDescendants, FixedDescendants),

    /// This node contributed some object or objects that will be needed to construct a proper flow
    /// later up the tree, but these objects have not yet found their home.
    ConstructionItemConstructionResult(ConstructionItem),
}

impl ConstructionResult {
    fn destroy(&mut self) {
        match *self {
            NoConstructionResult => {}
            FlowConstructionResult(ref mut flow, _, _) => flow.destroy(),
            ConstructionItemConstructionResult(ref mut item) => item.destroy(),
        }
    }
}

/// Represents the output of flow construction for a DOM node that has not yet resulted in a
/// complete flow. Construction items bubble up the tree until they find a `Flow` to be
/// attached to.
enum ConstructionItem {
    /// Inline boxes and associated {ib} splits that have not yet found flows.
    InlineBoxesConstructionItem(InlineBoxesConstructionResult),
    /// Potentially ignorable whitespace.
    WhitespaceConstructionItem(OpaqueNode, Arc<ComputedValues>),
    /// TableColumn Box
    TableColumnBoxConstructionItem(Box),
}

impl ConstructionItem {
    fn destroy(&mut self) {
        match *self {
            InlineBoxesConstructionItem(ref mut result) => {
                for splits in result.splits.mut_iter() {
                    for split in splits.mut_iter() {
                        split.destroy()
                    }
                }
            }
            WhitespaceConstructionItem(..) => {}
            TableColumnBoxConstructionItem(_) => {}
        }
    }
}

/// Represents inline boxes and {ib} splits that are bubbling up from an inline.
struct InlineBoxesConstructionResult {
    /// Any {ib} splits that we're bubbling up.
    ///
    /// TODO(pcwalton): Small vector optimization.
    splits: Option<~[InlineBlockSplit]>,

    /// Any boxes that succeed the {ib} splits.
    boxes: ~[Box],

    /// Any absolute descendants that we're bubbling up.
    abs_descendants: AbsDescendants,

    /// Any fixed descendants that we're bubbling up.
    fixed_descendants: FixedDescendants,
}

/// Represents an {ib} split that has not yet found the containing block that it belongs to. This
/// is somewhat tricky. An example may be helpful. For this DOM fragment:
///
///     <span>
///     A
///     <div>B</div>
///     C
///     </span>
///
/// The resulting `ConstructionItem` for the outer `span` will be:
///
///     InlineBoxesConstructionItem(Some(~[
///         InlineBlockSplit {
///             predecessor_boxes: ~[
///                 A
///             ],
///             block: ~BlockFlow {
///                 B
///             },
///         }),~[
///             C
///         ])
struct InlineBlockSplit {
    /// The inline boxes that precede the flow.
    ///
    /// TODO(pcwalton): Small vector optimization.
    predecessor_boxes: ~[Box],

    /// The flow that caused this {ib} split.
    flow: ~Flow,
}

impl InlineBlockSplit {
    fn destroy(&mut self) {
        self.flow.destroy()
    }
}

/// Methods on optional vectors.
///
/// TODO(pcwalton): I think this will no longer be necessary once Rust #8981 lands.
pub trait OptVector<T> {
    /// Turns this optional vector into an owned one. If the optional vector is `None`, then this
    /// simply returns an empty owned vector.
    fn to_vec(self) -> ~[T];

    /// Pushes a value onto this vector.
    fn push(&mut self, value: T);

    /// Pushes a vector onto this vector, consuming the original.
    fn push_all_move(&mut self, values: ~[T]);

    /// Pushes an optional vector onto this vector, consuming the original.
    fn push_opt_vec_move(&mut self, values: Self);

    /// Returns the length of this optional vector.
    fn len(&self) -> uint;
}

impl<T> OptVector<T> for Option<~[T]> {
    #[inline]
    fn to_vec(self) -> ~[T] {
        match self {
            None => ~[],
            Some(vector) => vector,
        }
    }

    #[inline]
    fn push(&mut self, value: T) {
        match *self {
            None => *self = Some(~[value]),
            Some(ref mut vector) => vector.push(value),
        }
    }

    #[inline]
    fn push_all_move(&mut self, values: ~[T]) {
        match *self {
            None => *self = Some(values),
            Some(ref mut vector) => vector.push_all_move(values),
        }
    }

    #[inline]
    fn push_opt_vec_move(&mut self, values: Option<~[T]>) {
        match values {
            None => {}
            Some(values) => self.push_all_move(values),
        }
    }

    #[inline]
    fn len(&self) -> uint {
        match *self {
            None => 0,
            Some(ref vector) => vector.len(),
        }
    }
}

/// An object that knows how to create flows.
pub struct FlowConstructor<'a> {
    /// The layout context.
    layout_context: &'a mut LayoutContext,

    /// An optional font context. If this is `None`, then we fetch the font context from the
    /// layout context.
    ///
    /// FIXME(pcwalton): This is pretty bogus and is basically just a workaround for libgreen
    /// having slow TLS.
    font_context: Option<~FontContext>,
}

impl<'a> FlowConstructor<'a> {
    /// Creates a new flow constructor.
    pub fn new(layout_context: &'a mut LayoutContext, font_context: Option<~FontContext>)
               -> FlowConstructor<'a> {
        FlowConstructor {
            layout_context: layout_context,
            font_context: font_context,
        }
    }

    fn font_context<'a>(&'a mut self) -> &'a mut FontContext {
        match self.font_context {
            Some(ref mut font_context) => {
                let font_context: &mut FontContext = *font_context;
                font_context
            }
            None => self.layout_context.font_context(),
        }
    }

    /// Destroys this flow constructor and retrieves the font context.
    pub fn unwrap_font_context(self) -> Option<~FontContext> {
        let FlowConstructor {
            font_context,
            ..
        } = self;
        font_context
    }

    /// Builds the `ImageBoxInfo` for the given image. This is out of line to guide inlining.
    fn build_box_info_for_image(&mut self, node: &ThreadSafeLayoutNode, url: Option<Url>) -> SpecificBoxInfo {
        match url {
            None => GenericBox,
            Some(url) => {
                // FIXME(pcwalton): The fact that image boxes store the cache within them makes
                // little sense to me.
                ImageBox(ImageBoxInfo::new(node, url, self.layout_context.image_cache.clone()))
            }
        }
    }

    /// Builds specific `Box` info for the given node.
    pub fn build_specific_box_info_for_node(&mut self, node: &ThreadSafeLayoutNode)
                                            -> SpecificBoxInfo {
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => self.build_box_info_for_image(node, node.image_url()),
            ElementNodeTypeId(HTMLIFrameElementTypeId) => IframeBox(IframeBoxInfo::new(node)),
            ElementNodeTypeId(HTMLObjectElementTypeId) => {
                let data = node.get_object_data(&self.layout_context.url);
                self.build_box_info_for_image(node, data)
            }
            ElementNodeTypeId(HTMLTableElementTypeId) => TableWrapperBox,
            ElementNodeTypeId(HTMLTableColElementTypeId) => TableColumnBox(TableColumnBoxInfo::new(node)),
            ElementNodeTypeId(HTMLTableDataCellElementTypeId) |
            ElementNodeTypeId(HTMLTableHeaderCellElementTypeId) => TableCellBox,
            ElementNodeTypeId(HTMLTableRowElementTypeId) |
            ElementNodeTypeId(HTMLTableSectionElementTypeId) => TableRowBox,
            TextNodeTypeId => UnscannedTextBox(UnscannedTextBoxInfo::new(node)),
            _ => GenericBox,
        }
    }

    /// Creates an inline flow from a set of inline boxes, then adds it as a child of the given flow
    /// or pushes it onto the given flow list.
    ///
    /// `#[inline(always)]` because this is performance critical and LLVM will not inline it
    /// otherwise.
    #[inline(always)]
    fn flush_inline_boxes_to_flow_or_list(&mut self,
                                          boxes: ~[Box],
                                          flow: &mut ~Flow,
                                          flow_list: &mut ~[~Flow],
                                          node: &ThreadSafeLayoutNode) {
        if boxes.len() == 0 {
            return
        }

        let mut inline_flow = ~InlineFlow::from_boxes((*node).clone(), boxes) as ~Flow;
        TextRunScanner::new().scan_for_runs(self.font_context(), inline_flow);
        inline_flow.finish(self.layout_context);

        if flow.need_anonymous_flow(inline_flow) {
            flow_list.push(inline_flow)
        } else {
            flow.add_new_child(inline_flow)
        }
    }

    /// Creates an inline flow from a set of inline boxes, if present, and adds it as a child of
    /// the given flow or pushes it onto the given flow list.
    fn flush_inline_boxes_to_flow_or_list_if_necessary(&mut self,
                                                       opt_boxes: &mut Option<~[Box]>,
                                                       flow: &mut ~Flow,
                                                       flow_list: &mut ~[~Flow],
                                                       node: &ThreadSafeLayoutNode) {
        let opt_boxes = mem::replace(opt_boxes, None);
        if opt_boxes.len() > 0 {
            self.flush_inline_boxes_to_flow_or_list(opt_boxes.to_vec(), flow, flow_list, node)
        }
    }

    /// Build block flow for current node using information from children nodes.
    ///
    /// Consume results from children and combine them, handling {ib} splits.
    /// Block flows and inline flows thus created will become the children of
    /// this block flow.
    /// Also, deal with the absolute and fixed descendants bubbled up by
    /// children nodes.
    fn build_flow_using_children(&mut self,
                                 mut flow: ~Flow,
                                 node: &ThreadSafeLayoutNode)
                                 -> ConstructionResult {
        // Gather up boxes for the inline flows we might need to create.
        let mut opt_boxes_for_inline_flow = None;
        let mut consecutive_siblings = ~[];
        let mut first_box = true;
        // List of absolute descendants, in tree order.
        let mut abs_descendants = Descendants::new();
        let mut fixed_descendants = Descendants::new();
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult => {}
                FlowConstructionResult(kid_flow, kid_abs_descendants, kid_fixed_descendants) => {
                    // If kid_flow is TableCaptionFlow, kid_flow should be added under TableWrapperFlow.
                    if flow.is_table() && kid_flow.is_table_caption() {
                        kid.set_flow_construction_result(FlowConstructionResult(kid_flow,
                                                                                Descendants::new(),
                                                                                Descendants::new()))
                    } else if flow.need_anonymous_flow(kid_flow) {
                        consecutive_siblings.push(kid_flow)
                    } else {
                        // Strip ignorable whitespace from the start of this flow per CSS 2.1 §
                        // 9.2.1.1.
                        if flow.is_table_kind() || first_box {
                            strip_ignorable_whitespace_from_start(&mut opt_boxes_for_inline_flow);
                            first_box = false
                        }

                        // Flush any inline boxes that we were gathering up. This allows us to handle
                        // {ib} splits.
                        debug!("flushing {} inline box(es) to flow A",
                                opt_boxes_for_inline_flow.as_ref()
                                .map_or(0, |boxes| boxes.len()));
                        self.flush_inline_boxes_to_flow_or_list_if_necessary(&mut opt_boxes_for_inline_flow,
                                                                             &mut flow,
                                                                             &mut consecutive_siblings,
                                                                             node);
                        if !consecutive_siblings.is_empty() {
                            self.generate_anonymous_missing_child(consecutive_siblings, &mut flow, node);
                            consecutive_siblings = ~[];
                        }
                        flow.add_new_child(kid_flow);
                    }
                    abs_descendants.push_descendants(kid_abs_descendants);
                    fixed_descendants.push_descendants(kid_fixed_descendants);
                }
                ConstructionItemConstructionResult(InlineBoxesConstructionItem(
                        InlineBoxesConstructionResult {
                            splits: opt_splits,
                            boxes: boxes,
                            abs_descendants: kid_abs_descendants,
                            fixed_descendants: kid_fixed_descendants,
                        })) => {
                    // Add any {ib} splits.
                    match opt_splits {
                        None => {}
                        Some(splits) => {
                            for split in splits.move_iter() {
                                // Pull apart the {ib} split object and push its predecessor boxes
                                // onto the list.
                                let InlineBlockSplit {
                                    predecessor_boxes: predecessor_boxes,
                                    flow: kid_flow
                                } = split;
                                opt_boxes_for_inline_flow.push_all_move(predecessor_boxes);

                                // If this is the first box in flow, then strip ignorable
                                // whitespace per CSS 2.1 § 9.2.1.1.
                                if first_box {
                                    strip_ignorable_whitespace_from_start(
                                        &mut opt_boxes_for_inline_flow);
                                    first_box = false
                                }

                                // Flush any inline boxes that we were gathering up.
                                debug!("flushing {} inline box(es) to flow A",
                                       opt_boxes_for_inline_flow.as_ref()
                                                                .map_or(0,
                                                                        |boxes| boxes.len()));
                                self.flush_inline_boxes_to_flow_or_list_if_necessary(
                                        &mut opt_boxes_for_inline_flow,
                                        &mut flow,
                                        &mut consecutive_siblings,
                                        node);

                                // Push the flow generated by the {ib} split onto our list of
                                // flows.
                                if flow.need_anonymous_flow(kid_flow) {
                                    consecutive_siblings.push(kid_flow)
                                } else {
                                    flow.add_new_child(kid_flow)
                                }
                            }
                        }
                    }

                    // Add the boxes to the list we're maintaining.
                    opt_boxes_for_inline_flow.push_all_move(boxes);
                    abs_descendants.push_descendants(kid_abs_descendants);
                    fixed_descendants.push_descendants(kid_fixed_descendants);
                }
                ConstructionItemConstructionResult(WhitespaceConstructionItem(..)) => {
                    // Nothing to do here.
                }
                ConstructionItemConstructionResult(TableColumnBoxConstructionItem(_)) => {
                    // TODO: Implement anonymous table objects for missing parents
                    // CSS 2.1 § 17.2.1, step 3-2
                }
            }
        }

        // Perform a final flush of any inline boxes that we were gathering up to handle {ib}
        // splits, after stripping ignorable whitespace.
        strip_ignorable_whitespace_from_end(&mut opt_boxes_for_inline_flow);
        self.flush_inline_boxes_to_flow_or_list_if_necessary(&mut opt_boxes_for_inline_flow,
                                                             &mut flow,
                                                             &mut consecutive_siblings,
                                                             node);
        if !consecutive_siblings.is_empty() {
            self.generate_anonymous_missing_child(consecutive_siblings, &mut flow, node);
        }

        // The flow is done.
        flow.finish(self.layout_context);
        let is_positioned = flow.as_block().is_positioned();
        let is_fixed_positioned = flow.as_block().is_fixed();
        let is_absolutely_positioned = flow.as_block().is_absolutely_positioned();
        if is_positioned {
            // This is the CB for all the absolute descendants.
            flow.set_abs_descendants(abs_descendants);
            abs_descendants = Descendants::new();

            if is_fixed_positioned {
                // Send itself along with the other fixed descendants.
                fixed_descendants.push(Rawlink::some(flow));
            } else if is_absolutely_positioned {
                // This is now the only absolute flow in the subtree which hasn't yet
                // reached its CB.
                abs_descendants.push(Rawlink::some(flow));
            }
        }
        FlowConstructionResult(flow, abs_descendants, fixed_descendants)
    }

    /// Builds a flow for a node with `display: block`. This yields a `BlockFlow` with possibly
    /// other `BlockFlow`s or `InlineFlow`s underneath it, depending on whether {ib} splits needed
    /// to happen.
    fn build_flow_for_block(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let flow = ~BlockFlow::from_node(self, node) as ~Flow;
        self.build_flow_using_children(flow, node)
    }

    /// Builds the flow for a node with `float: {left|right}`. This yields a float `BlockFlow` with
    /// a `BlockFlow` underneath it.
    fn build_flow_for_floated_block(&mut self, node: &ThreadSafeLayoutNode, float_kind: FloatKind)
                                    -> ConstructionResult {
        let flow = ~BlockFlow::float_from_node(self, node, float_kind) as ~Flow;
        self.build_flow_using_children(flow, node)
    }


    /// Concatenates the boxes of kids, adding in our own borders/padding/margins if necessary.
    /// Returns the `InlineBoxesConstructionResult`, if any. There will be no
    /// `InlineBoxesConstructionResult` if this node consisted entirely of ignorable whitespace.
    fn build_boxes_for_nonreplaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                                  -> ConstructionResult {
        let mut opt_inline_block_splits = None;
        let mut opt_box_accumulator = None;
        let mut abs_descendants = Descendants::new();
        let mut fixed_descendants = Descendants::new();

        // Concatenate all the boxes of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult => {}
                FlowConstructionResult(flow, kid_abs_descendants, kid_fixed_descendants) => {
                    // {ib} split. Flush the accumulator to our new split and make a new
                    // accumulator to hold any subsequent boxes we come across.
                    let split = InlineBlockSplit {
                        predecessor_boxes: mem::replace(&mut opt_box_accumulator, None).to_vec(),
                        flow: flow,
                    };
                    opt_inline_block_splits.push(split);
                    abs_descendants.push_descendants(kid_abs_descendants);
                    fixed_descendants.push_descendants(kid_fixed_descendants);
                }
                ConstructionItemConstructionResult(InlineBoxesConstructionItem(
                        InlineBoxesConstructionResult {
                            splits: opt_splits,
                            boxes: boxes,
                            abs_descendants: kid_abs_descendants,
                            fixed_descendants: kid_fixed_descendants,
                        })) => {

                    // Bubble up {ib} splits.
                    match opt_splits {
                        None => {}
                        Some(splits) => {
                            for split in splits.move_iter() {
                                let InlineBlockSplit {
                                    predecessor_boxes: boxes,
                                    flow: kid_flow
                                } = split;
                                opt_box_accumulator.push_all_move(boxes);

                                let split = InlineBlockSplit {
                                    predecessor_boxes: mem::replace(&mut opt_box_accumulator,
                                                                     None).to_vec(),
                                    flow: kid_flow,
                                };
                                opt_inline_block_splits.push(split)
                            }
                        }
                    }

                    // Push residual boxes.
                    opt_box_accumulator.push_all_move(boxes);
                    abs_descendants.push_descendants(kid_abs_descendants);
                    fixed_descendants.push_descendants(kid_fixed_descendants);
                }
                ConstructionItemConstructionResult(WhitespaceConstructionItem(whitespace_node,
                                                                              whitespace_style))
                        => {
                    // Instantiate the whitespace box.
                    opt_box_accumulator.push(Box::from_opaque_node_and_style(
                            whitespace_node,
                            whitespace_style,
                            UnscannedTextBox(UnscannedTextBoxInfo::from_text(~" "))))
                }
                ConstructionItemConstructionResult(TableColumnBoxConstructionItem(_)) => {
                    // TODO: Implement anonymous table objects for missing parents
                    // CSS 2.1 § 17.2.1, step 3-2
                }
            }
        }

        // fill inline info
        match opt_inline_block_splits {
            Some(ref splits) => {
                match opt_box_accumulator {
                    Some(ref boxes) => {
                        // Both
                        let mut total: ~[&Box] = ~[];
                        for split in splits.iter() {
                            for box_ in split.predecessor_boxes.iter() {
                                total.push(box_);
                            }
                        }
                        for box_ in boxes.iter() {
                            total.push(box_);
                        }
                        self.set_inline_info_for_inline_child(&total, node);

                    },
                    None => {
                        let mut total: ~[&Box] = ~[];
                        for split in splits.iter() {
                            for box_ in split.predecessor_boxes.iter() {
                                total.push(box_);
                            }
                        }
                        self.set_inline_info_for_inline_child(&total, node);
                    }
                }
            },
            None => {
                match opt_box_accumulator {
                    Some(ref boxes) => {
                        let mut total: ~[&Box] = ~[];
                        for box_ in boxes.iter() {
                            total.push(box_);
                        }
                        self.set_inline_info_for_inline_child(&total, node);
                    },
                    None => {}
                }
            }
        }

        // Finally, make a new construction result.
        if opt_inline_block_splits.len() > 0 || opt_box_accumulator.len() > 0
            || abs_descendants.len() > 0 {

            let construction_item = InlineBoxesConstructionItem(InlineBoxesConstructionResult {
                splits: opt_inline_block_splits,
                boxes: opt_box_accumulator.to_vec(),
                abs_descendants: abs_descendants,
                fixed_descendants: fixed_descendants,
            });
            ConstructionItemConstructionResult(construction_item)
        } else {
            NoConstructionResult
        }
    }

    fn set_inline_info_for_inline_child(&mut self,
                                        boxes: &~[&Box],
                                        parent_node: &ThreadSafeLayoutNode) {
        let parent_box = Box::new(self, parent_node);
        let font_style = parent_box.font_style();
        let font_group = self.font_context().get_resolved_font_for_style(&font_style);
        let (font_ascent,font_descent) = font_group.borrow().with_mut( |fg| {
            fg.fonts[0].borrow().with_mut( |font| {
                (font.metrics.ascent,font.metrics.descent)
            })
        });

        let boxes_len = boxes.len();
        parent_box.compute_borders(parent_box.style());

        for (i, box_) in boxes.iter().enumerate() {
            if box_.inline_info.with( |data| data.is_none() ) {
                box_.inline_info.set(Some(InlineInfo::new()));
            }

            let mut border = parent_box.border.get();
            if i != 0 {
                border.left = Zero::zero();
            }
            if i != (boxes_len - 1) {
                border.right = Zero::zero();
            }

            let mut info = box_.inline_info.borrow_mut();
            match info.get() {
                &Some(ref mut info) => {
                    // TODO(ksh8281) compute margin,padding
                    info.parent_info.push(
                        InlineParentInfo {
                            padding: Zero::zero(),
                            border: border,
                            margin: Zero::zero(),
                            style: parent_box.style.clone(),
                            font_ascent: font_ascent,
                            font_descent: font_descent,
                            node: OpaqueNode::from_thread_safe_layout_node(parent_node),
                        });
                },
                &None => {}
            }
        }
    }
    /// Creates an `InlineBoxesConstructionResult` for replaced content. Replaced content doesn't
    /// render its children, so this just nukes a child's boxes and creates a `Box`.
    fn build_boxes_for_replaced_inline_content(&mut self, node: &ThreadSafeLayoutNode)
                                               -> ConstructionResult {
        for kid in node.children() {
            kid.set_flow_construction_result(NoConstructionResult)
        }

        // If this node is ignorable whitespace, bail out now.
        if node.is_ignorable_whitespace() {
            let opaque_node = OpaqueNode::from_thread_safe_layout_node(node);
            return ConstructionItemConstructionResult(WhitespaceConstructionItem(
                opaque_node,
                node.style().clone()))
        }

        let construction_item = InlineBoxesConstructionItem(InlineBoxesConstructionResult {
            splits: None,
            boxes: ~[
                Box::new(self, node)
            ],
            abs_descendants: Descendants::new(),
            fixed_descendants: Descendants::new(),
        });
        ConstructionItemConstructionResult(construction_item)
    }

    /// Builds one or more boxes for a node with `display: inline`. This yields an
    /// `InlineBoxesConstructionResult`.
    fn build_boxes_for_inline(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        // Is this node replaced content?
        if !node.is_replaced_content() {
            // Go to a path that concatenates our kids' boxes.
            self.build_boxes_for_nonreplaced_inline_content(node)
        } else {
            // Otherwise, just nuke our kids' boxes, create our box if any, and be done with it.
            self.build_boxes_for_replaced_inline_content(node)
        }
    }

    /// TableCaptionFlow is populated underneath TableWrapperFlow
    fn place_table_caption_under_table_wrapper(&mut self,
                                               table_wrapper_flow: &mut ~Flow,
                                               node: &ThreadSafeLayoutNode) {
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult | ConstructionItemConstructionResult(_) => {}
                FlowConstructionResult(kid_flow, _, _) => {
                    // Only kid flows with table-caption are matched here.
                    assert!(kid_flow.is_table_caption());
                    table_wrapper_flow.add_new_child(kid_flow);
                }
            }
        }
    }

    /// Generates an anonymous table flow according to CSS 2.1 § 17.2.1, step 2.
    /// If necessary, generate recursively another anonymous table flow.
    fn generate_anonymous_missing_child(&mut self, child_flows: ~[~Flow],
                                        flow: &mut ~Flow, node: &ThreadSafeLayoutNode) {
        let mut anonymous_flow = flow.generate_missing_child_flow(node);
        let mut consecutive_siblings = ~[];
        for kid_flow in child_flows.move_iter() {
            if anonymous_flow.need_anonymous_flow(kid_flow) {
                consecutive_siblings.push(kid_flow);
                continue;
            }
            if !consecutive_siblings.is_empty() {
                self.generate_anonymous_missing_child(consecutive_siblings, &mut anonymous_flow, node);
                consecutive_siblings = ~[];
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
    fn build_flow_for_table_wrapper(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let box_ = Box::new_from_specific_info(node, TableWrapperBox);
        let mut wrapper_flow = ~TableWrapperFlow::from_node_and_box(node, box_) as ~Flow;

        let table_box_ = Box::new_from_specific_info(node, TableBox);
        let table_flow = ~TableFlow::from_node_and_box(node, table_box_) as ~Flow;

        // We first populate the TableFlow with other flows than TableCaptionFlow.
        // We then populate the TableWrapperFlow with TableCaptionFlow, and attach
        // the TableFlow to the TableWrapperFlow
        let construction_result = self.build_flow_using_children(table_flow, node);
        self.place_table_caption_under_table_wrapper(&mut wrapper_flow, node);

        let mut abs_descendants = Descendants::new();
        let mut fixed_descendants = Descendants::new();

        // NOTE: The order of captions and table are not the same order as in the DOM tree.
        // All caption blocks are placed before the table flow
        match construction_result {
            FlowConstructionResult(table_flow, table_abs_descendants, table_fixed_descendants) => {
                wrapper_flow.add_new_child(table_flow);
                abs_descendants.push_descendants(table_abs_descendants);
                fixed_descendants.push_descendants(table_fixed_descendants);
            }
            _ => {}
        }

        // The flow is done.
        wrapper_flow.finish(self.layout_context);
        let is_positioned = wrapper_flow.as_block().is_positioned();
        let is_fixed_positioned = wrapper_flow.as_block().is_fixed();
        let is_absolutely_positioned = wrapper_flow.as_block().is_absolutely_positioned();
        if is_positioned {
            // This is the CB for all the absolute descendants.
            wrapper_flow.set_abs_descendants(abs_descendants);
            abs_descendants = Descendants::new();

            if is_fixed_positioned {
                // Send itself along with the other fixed descendants.
                fixed_descendants.push(Rawlink::some(wrapper_flow));
            } else if is_absolutely_positioned {
                // This is now the only absolute flow in the subtree which hasn't yet
                // reached its CB.
                abs_descendants.push(Rawlink::some(wrapper_flow));
            }
        }
        FlowConstructionResult(wrapper_flow, abs_descendants, fixed_descendants)
    }

    /// Builds a flow for a node with `display: table-caption`. This yields a `TableCaptionFlow`
    /// with possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_caption(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let flow = ~TableCaptionFlow::from_node(self, node) as ~Flow;
        self.build_flow_using_children(flow, node)
    }

    /// Builds a flow for a node with `display: table-row-group`. This yields a `TableRowGroupFlow`
    /// with possibly other `TableRowFlow`s underneath it.
    fn build_flow_for_table_rowgroup(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let box_ = Box::new_from_specific_info(node, TableRowBox);
        let flow = ~TableRowGroupFlow::from_node_and_box(node, box_) as ~Flow;
        self.build_flow_using_children(flow, node)
    }

    /// Builds a flow for a node with `display: table-row`. This yields a `TableRowFlow` with
    /// possibly other `TableCellFlow`s underneath it.
    fn build_flow_for_table_row(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let box_ = Box::new_from_specific_info(node, TableRowBox);
        let flow = ~TableRowFlow::from_node_and_box(node, box_) as ~Flow;
        self.build_flow_using_children(flow, node)
    }

    /// Builds a flow for a node with `display: table-cell`. This yields a `TableCellFlow` with
    /// possibly other `BlockFlow`s or `InlineFlow`s underneath it.
    fn build_flow_for_table_cell(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let box_ = Box::new_from_specific_info(node, TableCellBox);
        let flow = ~TableCellFlow::from_node_and_box(node, box_) as ~Flow;
        self.build_flow_using_children(flow, node)
    }

    /// Creates a box for a node with `display: table-column`.
    fn build_boxes_for_table_column(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        // CSS 2.1 § 17.2.1. Treat all child boxes of a `table-column` as `display: none`.
        for kid in node.children() {
            kid.set_flow_construction_result(NoConstructionResult)
        }

        let specific = TableColumnBox(TableColumnBoxInfo::new(node));
        let construction_item = TableColumnBoxConstructionItem(
            Box::new_from_specific_info(node, specific)
        );
        ConstructionItemConstructionResult(construction_item)
    }

    /// Builds a flow for a node with `display: table-column-group`.
    /// This yields a `TableColGroupFlow`.
    fn build_flow_for_table_colgroup(&mut self, node: &ThreadSafeLayoutNode) -> ConstructionResult {
        let box_ = Box::new_from_specific_info(node,
                                               TableColumnBox(TableColumnBoxInfo::new(node)));
        let mut col_boxes = ~[];
        for kid in node.children() {
            // CSS 2.1 § 17.2.1. Treat all non-column child boxes of `table-column-group`
            // as `display: none`.
            match kid.swap_out_construction_result() {
                ConstructionItemConstructionResult(TableColumnBoxConstructionItem(box_)) => {
                    col_boxes.push(box_);
                }
                _ => {}
            }
        }
        if col_boxes.is_empty() {
            debug!("add TableColumnBox for empty colgroup");
            let specific = TableColumnBox(TableColumnBoxInfo::new(node));
            col_boxes.push( Box::new_from_specific_info(node, specific) );
        }
        let mut flow = ~TableColGroupFlow::from_node_and_boxes(node, box_, col_boxes) as ~Flow;
        flow.finish(self.layout_context);

        FlowConstructionResult(flow, Descendants::new(), Descendants::new())
    }
}

impl<'a> PostorderNodeMutTraversal for FlowConstructor<'a> {
    // Construct Flow based on 'display', 'position', and 'float' values.
    //
    // CSS 2.1 Section 9.7
    //
    // TODO: This should actually consult the table in that section to get the
    // final computed value for 'display'.
    //
    // `#[inline(always)]` because this is always called from the traversal function and for some
    // reason LLVM's inlining heuristics go awry here.
    #[inline(always)]
    fn process(&mut self, node: &ThreadSafeLayoutNode) -> bool {
        // Get the `display` property for this node, and determine whether this node is floated.
        let (display, float, positioning) = match node.type_id() {
            ElementNodeTypeId(_) => {
                let style = node.style().get();
                (style.Box.get().display, style.Box.get().float, style.Box.get().position)
            }
            TextNodeTypeId => (display::inline, float::none, position::static_),
            CommentNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId |
            DocumentNodeTypeId |
            ProcessingInstructionNodeTypeId => (display::none, float::none, position::static_),
        };

        debug!("building flow for node: {:?} {:?}", display, float);

        // Switch on display and floatedness.
        match (display, float, positioning) {
            // `display: none` contributes no flow construction result. Nuke the flow construction
            // results of children.
            (display::none, _, _) => {
                for child in node.children() {
                    let mut old_result = child.swap_out_construction_result();
                    old_result.destroy()
                }
            }

            // Table items contribute table flow construction results.
            (display::table, _, _) => {
                let construction_result = self.build_flow_for_table_wrapper(node);
                node.set_flow_construction_result(construction_result)
            }

            // Absolutely positioned elements will have computed value of
            // `float` as 'none' and `display` as per the table.
            // Currently, for original `display` value of 'inline', the new
            // `display` value is 'block'.
            (_, _, position::absolute) | (_, _, position::fixed) => {
                node.set_flow_construction_result(self.build_flow_for_block(node))
            }

            // Inline items contribute inline box construction results.
            (display::inline, float::none, _) => {
                let construction_result = self.build_boxes_for_inline(node);
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
                let construction_result = self.build_boxes_for_table_column(node);
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
                node.set_flow_construction_result(self.build_flow_for_block(node))
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

    /// Returns true if this node is ignorable whitespace.
    fn is_ignorable_whitespace(&self) -> bool;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(&self, result: ConstructionResult);

    /// Replaces the flow construction result in a node with `NoConstructionResult` and returns the
    /// old value.
    fn swap_out_construction_result(&self) -> ConstructionResult;
}

impl<'ln> NodeUtils for ThreadSafeLayoutNode<'ln> {
    fn is_replaced_content(&self) -> bool {
        match self.type_id() {
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId |
            DocumentNodeTypeId |
            ElementNodeTypeId(HTMLImageElementTypeId) => true,
            ElementNodeTypeId(HTMLObjectElementTypeId) => self.has_object_data(),
            ElementNodeTypeId(_) => false,
        }
    }

    fn is_ignorable_whitespace(&self) -> bool {
        match self.type_id() {
            TextNodeTypeId => {
                unsafe {
                    let text: JS<Text> = TextCast::to(self.get_jsmanaged()).unwrap();
                    if !is_whitespace(text.get().characterdata.data) {
                        return false
                    }

                    // NB: See the rules for `white-space` here:
                    //
                    //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
                    //
                    // If you implement other values for this property, you will almost certainly
                    // want to update this check.
                    match self.style().get().InheritedText.get().white_space {
                        white_space::normal => true,
                        _ => false,
                    }
                }
            }
            _ => false
        }
    }

    #[inline(always)]
    fn set_flow_construction_result(&self, result: ConstructionResult) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            Some(ref mut layout_data) => layout_data.data.flow_construction_result = result,
            None => fail!("no layout data"),
        }
    }

    #[inline(always)]
    fn swap_out_construction_result(&self) -> ConstructionResult {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            Some(ref mut layout_data) => {
                mem::replace(&mut layout_data.data.flow_construction_result, NoConstructionResult)
            }
            None => fail!("no layout data"),
        }
    }
}

/// Methods for interacting with HTMLObjectElement nodes
trait ObjectElement {
    /// Returns None if this node is not matching attributes.
    fn get_type_and_data(&self) -> (Option<&'static str>, Option<&'static str>);

    /// Returns true if this node has object data that is correct uri.
    fn has_object_data(&self) -> bool;

    /// Returns the "data" attribute value parsed as a URL
    fn get_object_data(&self, base_url: &Url) -> Option<Url>;
}

impl<'ln> ObjectElement for ThreadSafeLayoutNode<'ln> {
    fn get_type_and_data(&self) -> (Option<&'static str>, Option<&'static str>) {
        let elem = self.as_element();
        (elem.get_attr(&namespace::Null, "type"), elem.get_attr(&namespace::Null, "data"))
    }

    fn has_object_data(&self) -> bool {
        match self.get_type_and_data() {
            (None, Some(uri)) => is_image_data(uri),
            _ => false
        }
    }

    fn get_object_data(&self, base_url: &Url) -> Option<Url> {
        match self.get_type_and_data() {
            (None, Some(uri)) if is_image_data(uri) => Some(parse_url(uri, Some(base_url.clone()))),
            _ => None
        }
    }
}

/// Strips ignorable whitespace from the start of a list of boxes.
fn strip_ignorable_whitespace_from_start(opt_boxes: &mut Option<~[Box]>) {
    match mem::replace(opt_boxes, None) {
        None => return,
        Some(boxes) => {
            // FIXME(pcwalton): This is slow because vector shift is broken. :(
            let mut found_nonwhitespace = false;
            let mut result = ~[];
            let mut last_removed_box: Option<Box> = None;
            for box_ in boxes.move_iter() {
                if !found_nonwhitespace && box_.is_whitespace_only() {
                    debug!("stripping ignorable whitespace from start");
                    last_removed_box = Some(box_);
                    continue
                }

                found_nonwhitespace = true;
                match last_removed_box {
                    Some(ref last_removed_box) => {
                        box_.merge_noncontent_inline_left(last_removed_box);
                    },
                    None => {}
                }
                last_removed_box = None;
                result.push(box_)
            }

            *opt_boxes = Some(result)
        }
    }
}

/// Strips ignorable whitespace from the end of a list of boxes.
fn strip_ignorable_whitespace_from_end(opt_boxes: &mut Option<~[Box]>) {
    match *opt_boxes {
        None => {}
        Some(ref mut boxes) => {
            while boxes.len() > 0 && boxes.last().get_ref().is_whitespace_only() {
                debug!("stripping ignorable whitespace from end");
                let box_ = boxes.pop().unwrap();
                if boxes.len() > 0 {
                    boxes[boxes.len() - 1].merge_noncontent_inline_right(&box_);
                }
            }
        }
    }
    if opt_boxes.len() == 0 {
        *opt_boxes = None
    }
}
