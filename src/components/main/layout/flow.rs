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
use extra::dlist::{DList,MutDListIterator};
use extra::container::Deque;

use std::cell::Cell;
use std::io::stderr;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use script::dom::node::{AbstractNode, LayoutView};

/// The type of the formatting context and data specific to each context, such as line box
/// structures or float lists.
pub enum FlowContext {
    AbsoluteFlow(~FlowData), 
    BlockFlow(~BlockFlowData),
    FloatFlow(~FloatFlowData),
    InlineBlockFlow(~FlowData),
    InlineFlow(~InlineFlowData),
    TableFlow(~FlowData),
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
    pub fn each_bu_sub_inorder (&mut self, callback: &fn(&mut FlowContext) -> bool) -> bool {
        for kid in self.child_iter() {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !kid.each_bu_sub_inorder(|a| callback(a)) {
                return false;
            }
        }

        if !self.is_inorder() {
            callback(self)
        } else {
            true
        }
    }

    pub fn each_preorder_prune(&mut self, prune: &fn(&mut FlowContext) -> bool, 
                               callback: &fn(&mut FlowContext) -> bool) 
                               -> bool {
        if prune(self) {
            return true;
        }

        if !callback(self) {
            return false;
        }

        for kid in self.child_iter() {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !kid.each_preorder_prune(|a| prune(a), |a| callback(a)) {
                return false;
            }
        }

        true
    }

    pub fn each_postorder_prune(&mut self, prune: &fn(&mut FlowContext) -> bool, 
                                callback: &fn(&mut FlowContext) -> bool) 
                                -> bool {
        if prune(self) {
            return true;
        }

        for kid in self.child_iter() {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !kid.each_postorder_prune(|a| prune(a), |a| callback(a)) {
                return false;
            }
        }

        callback(self)
    }

    pub fn each_preorder(&mut self, callback: &fn(&mut FlowContext) -> bool) -> bool {
        self.each_preorder_prune(|_| false, callback)
    }

    pub fn each_postorder(&mut self, callback: &fn(&mut FlowContext) -> bool) -> bool {
        self.each_postorder_prune(|_| false, callback)
    }
}

impl<'self> FlowContext {
    pub fn is_block_like(&self) -> bool {
        match *self {
            BlockFlow(*) | FloatFlow(*) => true,
            _ => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        do self.with_base |base| {
            base.children.len() == 0
        }
    }

    pub fn add_new_child(&mut self, new_child: FlowContext) {
        let cell = Cell::new(new_child);
        do self.with_mut_base |base| {
            base.children.push_back(cell.take());
        }
    }

    pub fn with_first_child<R>(&mut self, cb: &fn(Option<&mut FlowContext>) -> R) -> R {
        do self.with_mut_base |base| {
            cb(base.children.front_mut())
        }
    }

    pub fn with_last_child<R>(&mut self, cb: &fn(Option<&mut FlowContext>) -> R) -> R {
        do self.with_mut_base |base| {
            cb(base.children.back_mut())
        }
    }

    pub fn last_child(&'self mut self) -> Option<&'self mut FlowContext> {
        self.mut_base().children.back_mut()
    }

    pub fn remove_first(&mut self) {
        do self.with_mut_base |base| {
            base.children.pop_front();
        }
    }

    pub fn remove_last(&mut self) {
        do self.with_mut_base |base| {
            base.children.pop_back();
        }
    }

    pub fn child_iter<'a>(&'a mut self) -> MutDListIterator<'a, FlowContext> {
        self.mut_base().children.mut_iter()
    }

}

impl<'self> FlowContext {
    pub fn with_base<R>(&self, callback: &fn(&FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(ref info) => callback(&**info),
            BlockFlow(ref info) => {
                callback(&info.common)
            }
            FloatFlow(ref info) => callback(&info.common),
            InlineBlockFlow(ref info) => callback(&**info),
            InlineFlow(ref info) => {
                callback(&info.common)
            }
            TableFlow(ref info) => callback(&**info)
        }
    }
    pub fn with_mut_base<R>(&mut self, callback: &fn(&mut FlowData) -> R) -> R {
        match *self {
            AbsoluteFlow(ref mut info) => callback(&mut **info),
            BlockFlow(ref mut info) => {
                callback(&mut info.common)
            }
            FloatFlow(ref mut info) => callback(&mut info.common),
            InlineBlockFlow(ref mut info) => callback(&mut **info),
            InlineFlow(ref mut info) => {
                callback(&mut info.common)
            }
            TableFlow(ref mut info) => callback(&mut **info),
        }
    }
    pub fn mut_base(&'self mut self) -> &'self mut FlowData {
        match *self {
            AbsoluteFlow(ref mut info) => &mut(**info),
            BlockFlow(ref mut info) => {
                &mut info.common
            }
            FloatFlow(ref mut info) => &mut info.common,
            InlineBlockFlow(ref mut info) => &mut(**info),
            InlineFlow(ref mut info) => {
                &mut info.common
            }
            TableFlow(ref mut info) => &mut(**info),
        }
    }
}

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData {
    node: AbstractNode<LayoutView>,
    restyle_damage: RestyleDamage,

    children: DList<FlowContext>,

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

            children: DList::new(),

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

    pub fn child_iter<'a>(&'a mut self) -> MutDListIterator<'a, FlowContext> {
        self.children.mut_iter()
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

    pub fn inline(&'self mut self) -> &'self mut InlineFlowData {
        match *self {
            InlineFlow(ref mut info) => &mut (**info),
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.id()))
        }
    }

    pub fn imm_inline(&'self self) -> &'self InlineFlowData {
        match *self {
            InlineFlow(ref info) => &**info,
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.id()))
        }
    }

    pub fn block(&'self mut self) -> &'self mut BlockFlowData {
        match *self {
            BlockFlow(ref mut info) => &mut (**info),
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.id()))
        }
    }

    pub fn root(&'self mut self) -> &'self mut BlockFlowData {
        match *self {
            BlockFlow(ref mut info) if info.is_root => &mut (**info),
            _ => fail!(fmt!("Tried to access root block data of non-root: f%d", self.id()))
        }
    }

    pub fn bubble_widths(&mut self, ctx: &mut LayoutContext) {

        debug!("FlowContext: bubbling widths for f%?", self.id());
        match *self {
            BlockFlow(ref mut info)  => info.bubble_widths_block(ctx),
            InlineFlow(ref mut info) => info.bubble_widths_inline(ctx),
            FloatFlow(ref mut info)  => info.bubble_widths_float(ctx),
            _ => fail!(fmt!("Tried to bubble_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_widths(&mut self, ctx: &mut LayoutContext) {

        debug!("FlowContext: assigning widths for f%?", self.id());
        match *self {
            BlockFlow(ref mut info)  => info.assign_widths_block(ctx),
            InlineFlow(ref mut info) => info.assign_widths_inline(ctx),
            FloatFlow(ref mut info)  => info.assign_widths_float(),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_height(&mut self, ctx: &mut LayoutContext) {

        debug!("FlowContext: assigning height for f%?", self.id());
        match *self {
            BlockFlow(ref mut info)  => info.assign_height_block(ctx),
            InlineFlow(ref mut info) => info.assign_height_inline(ctx),
            FloatFlow(ref mut info)  => info.assign_height_float(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(ref mut info)  => info.assign_height_inorder_block(ctx),
            InlineFlow(ref mut info) => info.assign_height_inorder_inline(ctx),
            FloatFlow(ref mut info)  => info.assign_height_inorder_float(),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
        }
    }

    pub fn build_display_list<E:ExtraDisplayListData>(&mut self,
                                                     builder: &DisplayListBuilder,
                                                     dirty: &Rect<Au>,
                                                     list: &Cell<DisplayList<E>>)
                                                     -> bool {

        
        debug!("FlowContext: building display list for f%?", self.id());
        match *self {
            BlockFlow(ref mut info)  => info.build_display_list_block(builder, dirty, list),
            InlineFlow(ref mut info) => info.build_display_list_inline(builder, dirty, list),
            FloatFlow(ref mut info)  => info.build_display_list_float(builder, dirty, list),
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
    pub fn foldl_all_boxes<B:Clone>(&mut self, seed: B, cb: &fn(a: B, b: RenderBox) -> B) -> B {
        match *self {
            BlockFlow(ref mut block) => {
                do block.box.map_default(seed.clone()) |box| {
                    cb(seed.clone(), *box)
                }
            }
            InlineFlow(ref mut inline) => {
                do inline.boxes.iter().fold(seed) |acc, box| {
                    cb(acc.clone(), *box)
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self)),
        }
    }

    pub fn foldl_boxes_for_node<B:Clone>(&mut self,
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

    pub fn iter_all_boxes(&mut self) -> BoxIterator {
        BoxIterator {
            boxes: match *self {
                BlockFlow (ref mut block)  => block.box.map_default(~[], |&x| ~[x]),
                InlineFlow(ref mut inline) => inline.boxes.clone(),
                _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
            },
            index: 0,
        }
    }

    /// Dumps the flow tree for debugging.
    pub fn dump(&mut self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    pub fn dump_indent(&mut self, indent: uint) {
        let mut s = ~"|";
        for _ in range(0, indent) {
            s.push_str("---- ");
        }

        s.push_str(self.debug_str());
        stderr().write_line(s);

        // FIXME: this should have a pure/const version?
        for child in self.child_iter() {
            child.dump_indent(indent + 1)
        }
    }
    
    pub fn debug_str(&self) -> ~str {
        let repr = match *self {
            InlineFlow(ref inline) => {
                let mut s = inline.boxes.iter().fold(~"InlineFlow(children=", |s, box| {
                    fmt!("%s b%d", s, box.id())
                });
                s.push_str(")");
                s
            },
            BlockFlow(ref block) => {
                match block.box {
                    Some(box) => fmt!("BlockFlow(box=b%d)", box.id()),
                    None => ~"BlockFlow",
                }
            },
            FloatFlow(ref float) => {
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

