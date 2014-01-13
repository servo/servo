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
use layout::box_::{Box, GenericBox, IframeBox, IframeBoxInfo, ImageBox, ImageBoxInfo};
use layout::box_::{UnscannedTextBox, UnscannedTextBoxInfo};
use layout::context::LayoutContext;
use layout::float_context::FloatType;
use layout::flow::{BaseFlow, Flow, MutableFlowUtils};
use layout::inline::InlineFlow;
use layout::text::TextRunScanner;
use layout::util::LayoutDataAccess;
use layout::wrapper::{LayoutNode, PostorderNodeMutTraversal};

use script::dom::element::{HTMLIframeElementTypeId, HTMLImageElementTypeId};
use script::dom::node::{CommentNodeTypeId, DoctypeNodeTypeId, DocumentFragmentNodeTypeId};
use script::dom::node::{DocumentNodeTypeId, ElementNodeTypeId, TextNodeTypeId};
use style::computed_values::{display, float};

use std::cell::RefCell;
use std::util;

/// The results of flow construction for a DOM node.
pub enum ConstructionResult {
    /// This node contributes nothing at all (`display: none`). Alternately, this is what newly
    /// created nodes have their `ConstructionResult` set to.
    NoConstructionResult,

    /// This node contributed a flow at the proper position in the tree. Nothing more needs to be
    /// done for this node.
    FlowConstructionResult(~Flow),

    /// This node contributed some object or objects that will be needed to construct a proper flow
    /// later up the tree, but these objects have not yet found their home.
    ConstructionItemConstructionResult(ConstructionItem),
}

/// Represents the output of flow construction for a DOM node that has not yet resulted in a
/// complete flow. Construction items bubble up the tree until they find a `Flow` to be
/// attached to.
enum ConstructionItem {
    /// Inline boxes and associated {ib} splits that have not yet found flows.
    InlineBoxesConstructionItem(InlineBoxesConstructionResult),
}

/// Represents inline boxes and {ib} splits that are bubbling up from an inline.
struct InlineBoxesConstructionResult {
    /// Any {ib} splits that we're bubbling up.
    ///
    /// TODO(pcwalton): Small vector optimization.
    splits: Option<~[InlineBlockSplit]>,

    /// Any boxes that succeed the {ib} splits.
    boxes: ~[Box],
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

/// Methods on optional vectors.
///
/// TODO(pcwalton): I think this will no longer be necessary once Rust #8981 lands.
trait OptVector<T> {
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
    ///
    /// FIXME(pcwalton): Why does this contain `@`??? That destroys parallelism!!!
    layout_context: &'a mut LayoutContext,

    /// The next flow ID to assign.
    ///
    /// FIXME(pcwalton): This is going to have to be atomic; can't we do something better?
    next_flow_id: RefCell<int>,
}

impl<'fc> FlowConstructor<'fc> {
    /// Creates a new flow constructor.
    pub fn init<'a>(layout_context: &'a mut LayoutContext) -> FlowConstructor<'a> {
        FlowConstructor {
            layout_context: layout_context,
            next_flow_id: RefCell::new(0),
        }
    }

    /// Returns the next flow ID and bumps the internal counter.
    fn next_flow_id(&self) -> int {
        let id = self.next_flow_id.get();
        self.next_flow_id.set(id + 1);
        id
    }

    /// Builds the `ImageBoxInfo` for the given image. This is out of line to guide inlining.
    fn build_box_info_for_image(&mut self, node: LayoutNode) -> Option<ImageBoxInfo> {
        // FIXME(pcwalton): Don't copy URLs.
        match node.image_url() {
            None => None,
            Some(url) => {
                // FIXME(pcwalton): The fact that image boxes store the cache within them makes
                // little sense to me.
                Some(ImageBoxInfo::new(&node, url, self.layout_context.image_cache.clone()))
            }
        }
    }

    /// Builds a `Box` for the given node.
    fn build_box_for_node(&mut self, node: LayoutNode) -> Box {
        let specific = match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                match self.build_box_info_for_image(node) {
                    None => GenericBox,
                    Some(image_box_info) => ImageBox(image_box_info),
                }
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => IframeBox(IframeBoxInfo::new(&node)),
            TextNodeTypeId => UnscannedTextBox(UnscannedTextBoxInfo::new(&node)),
            _ => GenericBox,
        };
        Box::new(node, specific)
    }

    /// Creates an inline flow from a set of inline boxes and adds it as a child of the given flow.
    ///
    /// `#[inline(always)]` because this is performance critical and LLVM will not inline it
    /// otherwise.
    #[inline(always)]
    fn flush_inline_boxes_to_flow(&mut self, boxes: ~[Box], flow: &mut ~Flow, node: LayoutNode) {
        if boxes.len() > 0 {
            let inline_base = BaseFlow::new(self.next_flow_id(), node);
            let mut inline_flow = ~InlineFlow::from_boxes(inline_base, boxes) as ~Flow;
            TextRunScanner::new().scan_for_runs(self.layout_context, inline_flow);
            flow.add_new_child(inline_flow)
        }
    }

    /// Creates an inline flow from a set of inline boxes, if present, and adds it as a child of
    /// the given flow.
    fn flush_inline_boxes_to_flow_if_necessary(&mut self,
                                               opt_boxes: &mut Option<~[Box]>,
                                               flow: &mut ~Flow,
                                               node: LayoutNode) {
        let opt_boxes = util::replace(opt_boxes, None);
        if opt_boxes.len() > 0 {
            self.flush_inline_boxes_to_flow(opt_boxes.to_vec(), flow, node)
        }
    }

    /// Builds the children flows underneath a node with `display: block`. After this call,
    /// other `BlockFlow`s or `InlineFlow`s will be populated underneath this node, depending on
    /// whether {ib} splits needed to happen.
    fn build_children_of_block_flow(&mut self,
                                    flow: &mut ~Flow,
                                    node: LayoutNode) {
        // Gather up boxes for the inline flows we might need to create.
        let mut opt_boxes_for_inline_flow = None;
        let mut first_box = true;
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult => {}
                FlowConstructionResult(kid_flow) => {
                    // Strip ignorable whitespace from the start of this flow per CSS 2.1 §
                    // 9.2.1.1.
                    if first_box {
                        strip_ignorable_whitespace_from_start(&mut opt_boxes_for_inline_flow);
                        first_box = false
                    }

                    // Flush any inline boxes that we were gathering up. This allows us to handle
                    // {ib} splits.
                    debug!("flushing {} inline box(es) to flow A",
                           opt_boxes_for_inline_flow.as_ref()
                                                    .map_default(0, |boxes| boxes.len()));
                    self.flush_inline_boxes_to_flow_if_necessary(&mut opt_boxes_for_inline_flow,
                                                                 flow,
                                                                 node);
                    flow.add_new_child(kid_flow);
                }
                ConstructionItemConstructionResult(InlineBoxesConstructionItem(
                        InlineBoxesConstructionResult {
                            splits: opt_splits,
                            boxes: boxes
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
                                                                .map_default(0,
                                                                             |boxes| boxes.len()));
                                self.flush_inline_boxes_to_flow_if_necessary(
                                        &mut opt_boxes_for_inline_flow,
                                        flow,
                                        node);

                                // Push the flow generated by the {ib} split onto our list of
                                // flows.
                                flow.add_new_child(kid_flow);
                            }
                        }
                    }

                    // Add the boxes to the list we're maintaining.
                    opt_boxes_for_inline_flow.push_all_move(boxes)
                }
            }
        }

        // Perform a final flush of any inline boxes that we were gathering up to handle {ib}
        // splits, after stripping ignorable whitespace.
        strip_ignorable_whitespace_from_end(&mut opt_boxes_for_inline_flow);
        self.flush_inline_boxes_to_flow_if_necessary(&mut opt_boxes_for_inline_flow,
                                                     flow,
                                                     node);
    }

    /// Builds a flow for a node with `display: block`. This yields a `BlockFlow` with possibly
    /// other `BlockFlow`s or `InlineFlow`s underneath it, depending on whether {ib} splits needed
    /// to happen.
    fn build_flow_for_block(&mut self, node: LayoutNode) -> ~Flow {
        let base = BaseFlow::new(self.next_flow_id(), node);
        let box_ = self.build_box_for_node(node);
        let mut flow = ~BlockFlow::from_box(base, box_) as ~Flow;
        self.build_children_of_block_flow(&mut flow, node);
        flow
    }

    /// Builds the flow for a node with `float: {left|right}`. This yields a float `BlockFlow` with
    /// a `BlockFlow` underneath it.
    fn build_flow_for_floated_block(&mut self, node: LayoutNode, float_type: FloatType)
                                    -> ~Flow {
        let base = BaseFlow::new(self.next_flow_id(), node);
        let box_ = self.build_box_for_node(node);
        let mut flow = ~BlockFlow::float_from_box(base, float_type, box_) as ~Flow;
        self.build_children_of_block_flow(&mut flow, node);
        flow
    }

    /// Concatenates the boxes of kids, adding in our own borders/padding/margins if necessary.
    /// Returns the `InlineBoxesConstructionResult`, if any. There will be no
    /// `InlineBoxesConstructionResult` if this node consisted entirely of ignorable whitespace.
    fn build_boxes_for_nonreplaced_inline_content(&mut self, node: LayoutNode)
                                                  -> ConstructionResult {
        let mut opt_inline_block_splits = None;
        let mut opt_box_accumulator = None;

        // Concatenate all the boxes of our kids, creating {ib} splits as necessary.
        for kid in node.children() {
            match kid.swap_out_construction_result() {
                NoConstructionResult => {}
                FlowConstructionResult(flow) => {
                    // {ib} split. Flush the accumulator to our new split and make a new
                    // accumulator to hold any subsequent boxes we come across.
                    let split = InlineBlockSplit {
                        predecessor_boxes: util::replace(&mut opt_box_accumulator, None).to_vec(),
                        flow: flow,
                    };
                    opt_inline_block_splits.push(split)
                }
                ConstructionItemConstructionResult(InlineBoxesConstructionItem(
                        InlineBoxesConstructionResult {
                            splits: opt_splits,
                            boxes: boxes
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
                                    predecessor_boxes: util::replace(&mut opt_box_accumulator,
                                                                     None).to_vec(),
                                    flow: kid_flow,
                                };
                                opt_inline_block_splits.push(split)
                            }
                        }
                    }

                    // Push residual boxes.
                    opt_box_accumulator.push_all_move(boxes)
                }
            }
        }

        // TODO(pcwalton): Add in our own borders/padding/margins if necessary.

        // Finally, make a new construction result.
        if opt_inline_block_splits.len() > 0 || opt_box_accumulator.len() > 0 {
            let construction_item = InlineBoxesConstructionItem(InlineBoxesConstructionResult {
                splits: opt_inline_block_splits,
                boxes: opt_box_accumulator.to_vec(),
            });
            ConstructionItemConstructionResult(construction_item)
        } else {
            NoConstructionResult
        }
    }

    /// Creates an `InlineBoxesConstructionResult` for replaced content. Replaced content doesn't
    /// render its children, so this just nukes a child's boxes and creates a `Box`.
    fn build_boxes_for_replaced_inline_content(&mut self, node: LayoutNode) -> ConstructionResult {
        for kid in node.children() {
            kid.set_flow_construction_result(NoConstructionResult)
        }

        let construction_item = InlineBoxesConstructionItem(InlineBoxesConstructionResult {
            splits: None,
            boxes: ~[
                self.build_box_for_node(node)
            ],
        });
        ConstructionItemConstructionResult(construction_item)
    }

    /// Builds one or more boxes for a node with `display: inline`. This yields an
    /// `InlineBoxesConstructionResult`.
    fn build_boxes_for_inline(&mut self, node: LayoutNode) -> ConstructionResult {
        // Is this node replaced content?
        if !node.is_replaced_content() {
            // Go to a path that concatenates our kids' boxes.
            self.build_boxes_for_nonreplaced_inline_content(node)
        } else {
            // Otherwise, just nuke our kids' boxes, create our box if any, and be done with it.
            self.build_boxes_for_replaced_inline_content(node)
        }
    }
}

impl<'a> PostorderNodeMutTraversal for FlowConstructor<'a> {
    // `#[inline(always)]` because this is always called from the traversal function and for some
    // reason LLVM's inlining heuristics go awry here.
    #[inline(always)]
    fn process(&mut self, node: LayoutNode) -> bool {
        // Get the `display` property for this node, and determine whether this node is floated.
        let (display, float) = match node.type_id() {
            ElementNodeTypeId(_) => {
                let style = node.style().get();
                (style.Box.display, style.Box.float)
            }
            TextNodeTypeId => (display::inline, float::none),
            CommentNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId |
            DocumentNodeTypeId(_) => (display::none, float::none),
        };

        debug!("building flow for node: {:?} {:?}", display, float);

        // Switch on display and floatedness.
        match (display, float) {
            // `display: none` contributes no flow construction result. Nuke the flow construction
            // results of children.
            (display::none, _) => {
                for child in node.children() {
                    child.set_flow_construction_result(NoConstructionResult)
                }
            }

            // Inline items contribute inline box construction results.
            (display::inline, float::none) => {
                let construction_result = self.build_boxes_for_inline(node);
                node.set_flow_construction_result(construction_result)
            }

            // Block flows that are not floated contribute block flow construction results.
            //
            // TODO(pcwalton): Make this only trigger for blocks and handle the other `display`
            // properties separately.
            (_, float::none) => {
                let flow = self.build_flow_for_block(node);
                node.set_flow_construction_result(FlowConstructionResult(flow))
            }

            // Floated flows contribute float flow construction results.
            (_, float_value) => {
                let float_type = FloatType::from_property(float_value);
                let flow = self.build_flow_for_floated_block(node, float_type);
                node.set_flow_construction_result(FlowConstructionResult(flow))
            }
        }

        true
    }
}

/// A utility trait with some useful methods for node queries.
trait NodeUtils {
    /// Returns true if this node doesn't render its kids and false otherwise.
    fn is_replaced_content(self) -> bool;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(self, result: ConstructionResult);

    /// Replaces the flow construction result in a node with `NoConstructionResult` and returns the
    /// old value.
    fn swap_out_construction_result(self) -> ConstructionResult;
}

impl<'ln> NodeUtils for LayoutNode<'ln> {
    fn is_replaced_content(self) -> bool {
        match self.type_id() {
            TextNodeTypeId |
            CommentNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId |
            DocumentNodeTypeId(_) |
            ElementNodeTypeId(HTMLImageElementTypeId) => true,
            ElementNodeTypeId(_) => false,
        }
    }

    #[inline(always)]
    fn set_flow_construction_result(self, result: ConstructionResult) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            Some(ref mut layout_data) => layout_data.data.flow_construction_result = result,
            None => fail!("no layout data"),
        }
    }

    #[inline(always)]
    fn swap_out_construction_result(self) -> ConstructionResult {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            Some(ref mut layout_data) => {
                util::replace(&mut layout_data.data.flow_construction_result, NoConstructionResult)
            }
            None => fail!("no layout data"),
        }
    }
}

/// Strips ignorable whitespace from the start of a list of boxes.
fn strip_ignorable_whitespace_from_start(opt_boxes: &mut Option<~[Box]>) {
    match util::replace(opt_boxes, None) {
        None => return,
        Some(boxes) => {
            // FIXME(pcwalton): This is slow because vector shift is broken. :(
            let mut found_nonwhitespace = false;
            let mut result = ~[];
            for box_ in boxes.move_iter() {
                if !found_nonwhitespace && box_.is_whitespace_only() {
                    debug!("stripping ignorable whitespace from start");
                    continue
                }

                found_nonwhitespace = true;
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
            while boxes.len() > 0 && boxes.last().is_whitespace_only() {
                debug!("stripping ignorable whitespace from end");
                let _ = boxes.pop();
            }
        }
    }
    if opt_boxes.len() == 0 {
        *opt_boxes = None
    }
}

