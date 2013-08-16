/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's experimental layout system builds a tree of `FlowContext` and `RenderBox` objects and
/// solves layout constraints to obtain positions and display attributes of tree nodes. Positions
/// are computed in several tree traversals driven by the fundamental data dependencies required by
/// inline and block layout.
/// 
/// Flows are interior nodes in the layout tree and correspond closely to *flow contexts* in the
/// CSS specification. Flows are responsible for positioning their child flow contexts and render
/// boxes. Flows have purpose-specific fields, such as auxiliary line box structs, out-of-flow
/// child lists, and so on.
///
/// Currently, the important types of flows are:
/// 
/// * `BlockFlow`: A flow that establishes a block context. It has several child flows, each of
///   which are positioned according to block formatting context rules (CSS block boxes). Block
///   flows also contain a single `GenericBox` to represent their rendered borders, padding, etc.
///   (In the future, this render box may be folded into `BlockFlow` to save space.) The BlockFlow
///   at the root of the tree has special behavior: it stretches to the boundaries of the viewport.
///   
/// * `InlineFlow`: A flow that establishes an inline context. It has a flat list of child
///   boxes/flows that are subject to inline layout and line breaking and structs to represent
///   line breaks and mapping to CSS boxes, for the purpose of handling `getClientRects()` and
///   similar methods.

use layout::block::BlockFlowData;
use layout::float::FloatFlowData;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::inline::{InlineFlowData};
use layout::float_context::{FloatContext, Invalid, FloatType};
use layout::incremental::RestyleDamage;
use css::node_style::StyledNode;

use std::cell::Cell;
use std::io::stderr;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::{TreeNode, TreeNodeRef};

/// The type of the formatting context and data specific to each context, such as line box
/// structures or float lists.
#[deriving(Clone)]
pub enum FlowContext {
    AbsoluteFlow(@mut FlowData), 
    BlockFlow(@mut BlockFlowData),
    FloatFlow(@mut FloatFlowData),
    InlineBlockFlow(@mut FlowData),
    InlineFlow(@mut InlineFlowData),
    TableFlow(@mut FlowData),
}

pub enum FlowContextType {
    Flow_Absolute, 
    Flow_Block,
    Flow_Float(FloatType),
    Flow_InlineBlock,
    Flow_Inline,
    Flow_Root,
    Flow_Table
}

impl FlowContext {
    pub fn teardown(&self) {
        match *self {
          AbsoluteFlow(data) |
          InlineBlockFlow(data) |
          TableFlow(data) => data.teardown(),
          BlockFlow(data) => data.teardown(),
          FloatFlow(data) => data.teardown(),
          InlineFlow(data) => data.teardown()
        }
    }

    /// Like traverse_preorder, but don't end the whole traversal if the callback
    /// returns false.
    //
    // FIXME: Unify this with traverse_preorder_prune, which takes a separate
    // 'prune' function.
    pub fn partially_traverse_preorder(&self, callback: &fn(FlowContext) -> bool) {
        if !callback((*self).clone()) {
            return;
        }

        for kid in self.children() {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            kid.partially_traverse_preorder(|a| callback(a));
        }
    }

    pub fn traverse_bu_sub_inorder (&self, callback: &fn(FlowContext)) {
        for kid in self.children() {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            kid.traverse_bu_sub_inorder(|a| callback(a));
        }

        if !self.is_inorder() {
            callback((*self).clone())
        }
    }
}

impl FlowData {
    pub fn teardown(&mut self) {
        // Under the assumption that all flows exist in a tree,
        // we must restrict ourselves to finalizing flows that
        // are descendents and subsequent siblings to ourselves,
        // or we risk dynamic borrow failures.
        self.parent = None;

        for flow in self.first_child.iter() {
            flow.teardown();
        }
        self.first_child = None;

        self.last_child = None;

        for flow in self.next_sibling.iter() {
            flow.teardown();
        }
        self.next_sibling = None;

        self.prev_sibling = None;
    }
}

impl TreeNodeRef<FlowData> for FlowContext {
    fn with_base<R>(&self, callback: &fn(&FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(info) => callback(info),
            BlockFlow(info) => {
                callback(&info.common)
            }
            FloatFlow(info) => callback(&info.common),
            InlineBlockFlow(info) => callback(info),
            InlineFlow(info) => {
                callback(&info.common)
            }
            TableFlow(info) => callback(info)
        }
    }
    fn with_mut_base<R>(&self, callback: &fn(&mut FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(info) => callback(info),
            BlockFlow(info) => {
                callback(&mut info.common)
            }
            FloatFlow(info) => callback(&mut info.common),
            InlineBlockFlow(info) => callback(info),
            InlineFlow(info) => {
                callback(&mut info.common)
            }
            TableFlow(info) => callback(info),
        }
    }

    fn parent_node(node: &FlowData) -> Option<FlowContext> {
        node.parent
    }

    fn first_child(node: &FlowData) -> Option<FlowContext> {
        node.first_child
    }

    fn last_child(node: &FlowData) -> Option<FlowContext> {
        node.last_child
    }

    fn prev_sibling(node: &FlowData) -> Option<FlowContext> {
        node.prev_sibling
    }

    fn next_sibling(node: &FlowData) -> Option<FlowContext> {
        node.next_sibling
    }

    fn set_parent_node(node: &mut FlowData, new_parent_node: Option<FlowContext>) {
        node.parent = new_parent_node
    }

    fn set_first_child(node: &mut FlowData, new_first_child: Option<FlowContext>) {
        node.first_child = new_first_child
    }

    fn set_last_child(node: &mut FlowData, new_last_child: Option<FlowContext>) {
        node.last_child = new_last_child
    }

    fn set_prev_sibling(node: &mut FlowData, new_prev_sibling: Option<FlowContext>) {
        node.prev_sibling = new_prev_sibling
    }

    fn set_next_sibling(node: &mut FlowData, new_next_sibling: Option<FlowContext>) {
        node.next_sibling = new_next_sibling
    }
}

impl TreeNode<FlowContext> for FlowData { }

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData {
    node: AbstractNode<LayoutView>,
    restyle_damage: RestyleDamage,

    parent: Option<FlowContext>,
    first_child: Option<FlowContext>,
    last_child: Option<FlowContext>,
    prev_sibling: Option<FlowContext>,
    next_sibling: Option<FlowContext>,

    /* TODO (Issue #87): debug only */
    id: int,

    /* layout computations */
    // TODO: min/pref and position are used during disjoint phases of
    // layout; maybe combine into a single enum to save space.
    min_width: Au,
    pref_width: Au,
    position: Rect<Au>,
    floats_in: FloatContext,
    floats_out: FloatContext,
    num_floats: uint,
    abs_position: Point2D<Au>,
    is_inorder: bool,
}

pub struct BoxIterator {
    priv boxes: ~[RenderBox],
    priv index: uint,
}

impl Iterator<RenderBox> for BoxIterator {
    fn next(&mut self) -> Option<RenderBox> {
        if self.index >= self.boxes.len() {
            None
        } else {
            let v = self.boxes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

impl FlowData {
    pub fn new(id: int, node: AbstractNode<LayoutView>) -> FlowData {
        FlowData {
            node: node,
            restyle_damage: node.restyle_damage(),

            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,

            id: id,

            min_width: Au(0),
            pref_width: Au(0),
            position: Au::zero_rect(),
            floats_in: Invalid,
            floats_out: Invalid,
            num_floats: 0,
            abs_position: Point2D(Au(0), Au(0)),
            is_inorder: false
        }
    }
}

impl<'self> FlowContext {
    /// A convenience method to return the position of this flow. Fails if the flow is currently
    /// being borrowed mutably.
    #[inline(always)]
    pub fn position(&self) -> Rect<Au> {
        do self.with_base |common_info| {
            common_info.position
        }
    }

    #[inline(always)]
    pub fn is_inorder(&self) -> bool {
        do self.with_base |common_info| {
            common_info.is_inorder
        }
    }

    /// A convenience method to return the ID of this flow. Fails if the flow is currently being
    /// borrowed mutably.
    #[inline(always)]
    pub fn id(&self) -> int {
        do self.with_base |info| {
            info.id
        }
    }


    pub fn inline(&self) -> @mut InlineFlowData {
        match *self {
            InlineFlow(info) => info,
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.id()))
        }
    }

    pub fn block(&self) -> @mut BlockFlowData {
        match *self {
            BlockFlow(info) => info,
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.id()))
        }
    }

    pub fn root(&self) -> @mut BlockFlowData {
        match *self {
            BlockFlow(info) if info.is_root => info,
            _ => fail!(fmt!("Tried to access root block data of non-root: f%d", self.id()))
        }
    }

    pub fn bubble_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.bubble_widths_block(ctx),
            InlineFlow(info) => info.bubble_widths_inline(ctx),
            FloatFlow(info)  => info.bubble_widths_float(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_widths(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_widths_block(ctx),
            InlineFlow(info) => info.assign_widths_inline(ctx),
            FloatFlow(info)  => info.assign_widths_float(),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_height(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_height_block(ctx),
            InlineFlow(info) => info.assign_height_inline(ctx),
            FloatFlow(info)  => info.assign_height_float(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn assign_height_inorder(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_height_inorder_block(ctx),
            InlineFlow(info) => info.assign_height_inorder_inline(ctx),
            FloatFlow(info)  => info.assign_height_inorder_float(),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn build_display_list<E:ExtraDisplayListData>(&self,
                                                     builder: &DisplayListBuilder,
                                                     dirty: &Rect<Au>,
                                                     list: &Cell<DisplayList<E>>)
                                                     -> bool {

        
        match *self {
            BlockFlow(info)  => info.build_display_list_block(builder, dirty, list),
            InlineFlow(info) => info.build_display_list_inline(builder, dirty, list),
            FloatFlow(info)  => info.build_display_list_float(builder, dirty, list),
            _ => {
                fail!("Tried to build_display_list_recurse of flow: %?", self)
            }
        }

    }
    /// A convenience method to return the restyle damage of this flow. Fails if the flow is
    /// currently being borrowed mutably.
    #[inline(always)]
    pub fn restyle_damage(&self) -> RestyleDamage {
        do self.with_base |info| {
            info.restyle_damage
        }
    }


    // Actual methods that do not require much flow-specific logic
    pub fn foldl_all_boxes<B:Clone>(&self, seed: B, cb: &fn(a: B, b: RenderBox) -> B) -> B {
        match *self {
            BlockFlow(block) => {
                let block = &mut *block;
                do block.box.map_default(seed.clone()) |box| {
                    cb(seed.clone(), *box)
                }
            }
            InlineFlow(inline) => {
                let inline = &mut *inline;
                do inline.boxes.iter().fold(seed) |acc, box| {
                    cb(acc.clone(), *box)
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self)),
        }
    }

    pub fn foldl_boxes_for_node<B:Clone>(&self,
                                        node: AbstractNode<LayoutView>,
                                        seed: B,
                                        callback: &fn(a: B, RenderBox) -> B)
                                        -> B {
        do self.foldl_all_boxes(seed) |acc, box| {
            if box.node() == node {
                callback(acc, box)
            } else {
                acc
            }
        }
    }

    pub fn iter_all_boxes(&self) -> BoxIterator {
        BoxIterator {
            boxes: match *self {
                BlockFlow (block)  => block.box.map_default(~[], |&x| ~[x]),
                InlineFlow(inline) => inline.boxes.clone(),
                _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
            },
            index: 0,
        }
    }

    /// Dumps the flow tree for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: uint) {
        let mut s = ~"|";
        for _ in range(0, indent) {
            s.push_str("---- ");
        }

        s.push_str(self.debug_str());
        stderr().write_line(s);

        // FIXME: this should have a pure/const version?
        for child in self.children() {
            child.dump_indent(indent + 1)
        }
    }
    
    pub fn debug_str(&self) -> ~str {
        let repr = match *self {
            InlineFlow(inline) => {
                let mut s = inline.boxes.iter().fold(~"InlineFlow(children=", |s, box| {
                    fmt!("%s b%d", s, box.id())
                });
                s.push_str(")");
                s
            },
            BlockFlow(block) => {
                match block.box {
                    Some(box) => fmt!("BlockFlow(box=b%d)", box.id()),
                    None => ~"BlockFlow",
                }
            },
            FloatFlow(float) => {
                match float.box {
                    Some(box) => fmt!("FloatFlow(box=b%d)", box.id()),
                    None => ~"FloatFlow",
                }
            },
            _ => ~"(Unknown flow)"
        };

        do self.with_base |base| {
            fmt!("f%? %? floats %? size %? damage %?", base.id, repr, base.num_floats,
                 base.position, base.restyle_damage)
        }
    }
}

