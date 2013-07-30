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
use std::uint;
use std::io::stderr;
use std::cast::transmute;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::{TreeNode, TreeNodeRef, TreeUtils};

// Phantom types to describe different views of the flow tree.
// Sequential view is used by misc. layout operations, VisitView
// is used when visiting a node during a traversal, and
// VisitChildView is for any node in the subtree of the visited
// node during a traversal.
// VisitOrChildView allows us to quantify over the two visit views.
pub struct SequentialView;
pub struct VisitView;
pub struct VisitChildView;
pub trait VisitOrChildView {}
impl VisitOrChildView for VisitView {}
impl VisitOrChildView for VisitChildView {}

/// The type of the formatting context and data specific to each context, such as line box
/// structures or float lists. The first type parameter is the view of this node, and the
/// second parameter is the view of all child nodes.
#[deriving(Clone)]
pub enum FlowContext<View,ChildView> {
    AbsoluteFlow(@mut FlowData<View, ChildView>), 
    BlockFlow(@mut BlockFlowData<View, ChildView>),
    FloatFlow(@mut FloatFlowData<View, ChildView>),
    InlineBlockFlow(@mut FlowData<View, ChildView>),
    InlineFlow(@mut InlineFlowData<View, ChildView>),
    TableFlow(@mut FlowData<View, ChildView>),
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

impl<V,CV> FlowContext<V,CV> {
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
    //
    // FIXME: Unify this with traverse_preorder_prune, which takes a separate
    // 'prune' function.
}

impl FlowContext<SequentialView,SequentialView> {
    pub unsafe fn decode(compressed_ptr: uint) -> Self {
        let new_ptr: *FlowContext<SequentialView,SequentialView> = transmute(compressed_ptr);
        *new_ptr
    }

    pub unsafe fn encode(&self) -> uint {
        let new_ptr: *FlowContext<SequentialView,SequentialView> = self;
        transmute(new_ptr)
    }

    pub unsafe fn restrict_view(&self) -> FlowContext<VisitView,VisitChildView> {
        transmute(*self)
    }

    pub unsafe fn get_traversal(&self) -> uint {
        self.with_base |base| {
            base.cur_traversal
        }
    }

    pub unsafe fn set_traversal(&self, traversal: uint) {
        self.with_mut_base |base| {
            base.cur_traversal = traversal;
        }
    }

    pub unsafe fn update_child_counter(&mut self) {
        do self.with_mut_base |base| {
            // increment count
            base.count.fetch_add(1);

            // TODO(eatkinson): num_children is slow, we should fix
            // this.
            let children = self.num_children();

            // if the count is num_children, replace it with 0 and return true
            base.count.compare_and_swap(children, 0) == children
        }

    }
}

impl<V,CV> FlowData<V,CV> {
    pub fn teardown(&mut self) {
        // Under the assumption that all flows exist in a tree,
        // we must restrict ourselves to finalizing flows that
        // are descendents and subsequent siblings to ourselves,
        // or we risk dynamic borrow failures.
        self.parent = None;

        for self.first_child.iter().advance |flow| {
            flow.teardown();
        }
        self.first_child = None;

        self.last_child = None;

        for self.next_sibling.iter().advance |flow| {
            flow.teardown();
        }
        self.next_sibling = None;

        self.prev_sibling = None;
    }
}

impl<V,CV> TreeNodeRef<FlowData<V,CV>> for FlowContext<V,CV> {
    fn with_base<R>(&self, callback: &fn(&FlowData<V,CV>) -> R) -> R {
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

    fn with_mut_base<R>(&self, callback: &fn(&mut FlowData<V,CV>) -> R) -> R {
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
}

impl <V,CV> FlowContext<V,CV> {
    fn with_base<R>(&self, callback: &fn(&FlowData<V,CV>) -> R) -> R {
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

    fn with_mut_base<R>(&self, callback: &fn(&mut FlowData<V,CV>) -> R) -> R {
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
}

/// Data common to all flows.
///
/// FIXME: We need a naming convention for pseudo-inheritance like this. How about
/// `CommonFlowInfo`?
pub struct FlowData<View,ChildView> {
    priv node: AbstractNode<LayoutView>,
    restyle_damage: RestyleDamage,

    priv parent: Option<FlowContext<View,View>>,
    priv first_child: Option<FlowContext<ChildView,ChildView>>,
    priv last_child: Option<FlowContext<ChildView,ChildView>>,
    priv prev_sibling: Option<FlowContext<View,View>>,
    priv next_sibling: Option<FlowContext<View,View>>,

    /* TODO (Issue #87): debug only */
    id: int,
    priv current_traversal: uint,
    priv child_counter: AtomicUint,

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
    is_inorder: bool
}

// SequentialView flows can perform arbitrary tree operations.
impl TreeNode<FlowContext<SequentialView, SequentialView>> 
for  FlowData<SequentialView, SequentialView> {
    fn parent_node(&self) -> Option<FlowContext<SequentialView,SequentialView>> {
        self.parent
    }

    fn first_child(&self) -> Option<FlowContext<SequentialView,SequentialView>> {
        self.first_child
    }

    fn last_child(&self) -> Option<FlowContext<SequentialView,SequentialView>> {
        self.last_child
    }

    fn prev_sibling(&self) -> Option<FlowContext<SequentialView,SequentialView>> {
        self.prev_sibling
    }

    fn next_sibling(&self) -> Option<FlowContext<SequentialView,SequentialView>> {
        self.next_sibling
    }

    fn set_parent_node(&mut self, 
                       new_parent_node: Option<FlowContext<SequentialView,SequentialView>>) {
        self.parent = new_parent_node
    }

    fn set_first_child(&mut self, 
                       new_first_child: Option<FlowContext<SequentialView,SequentialView>>) {
        self.first_child = new_first_child
    }

    fn set_last_child(&mut self, 
                      new_last_child: Option<FlowContext<SequentialView,SequentialView>>) {
        self.last_child = new_last_child
    }

    fn set_prev_sibling(&mut self, 
                        new_prev_sibling: Option<FlowContext<SequentialView,SequentialView>>) {
        self.prev_sibling = new_prev_sibling
    }

    fn set_next_sibling(&mut self, 
                        new_next_sibling: Option<FlowContext<SequentialView,SequentialView>>) {
        self.next_sibling = new_next_sibling
    }
}

impl FlowData<SequentialView,SequentialView> {
    pub fn node(&self) -> AbstractNode<LayoutView> {
        self.node
    }
}

// Visitors can only read the subtree rooted at this node.
impl<V:VisitOrChildView> FlowContext<V,VisitChildView> {
    fn first_child(&self) -> Option<FlowContext<VisitChildView,VisitChildView>> {
        do self.with_base |base| {
            base.first_child
        }
    }

    fn last_child(&self) -> Option<FlowContext<VisitChildView,VisitChildView>> {
        do self.with_base |base| {
            base.last_child
        }
    }
}

impl FlowContext<VisitChildView,VisitChildView> {
    fn prev_sibling(&self) -> Option<FlowContext<VisitChildView,VisitChildView>> {
        do self.with_base |base| {
            base.prev_sibling
        }
    }

    fn next_sibling(&self) -> Option<FlowContext<VisitChildView,VisitChildView>> {
        do self.with_base |base| {
            base.next_sibling
        }
    }
}

impl<V,CV> FlowData<V,CV> {
    pub fn new(id: int, node: AbstractNode<LayoutView>) -> FlowData<V,CV> {
        FlowData {
            node: node,
            restyle_damage: node.restyle_damage(),

            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,

            id: id,
            current_traversal: 0,
            child_counter: AtomicUint(0),

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

impl<V:VisitOrChildView> FlowContext<V,VisitChildView> {
    /// A convenience method to return the restyle damage of this flow. Fails if the flow is
    /// currently being borrowed mutably.
    #[inline(always)]
    pub fn restyle_damage(&self) -> RestyleDamage {
        do self.with_base |info| {
            info.restyle_damage
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
            FloatFlow(info)  => info.assign_widths_float(ctx),
            _ => fail!(fmt!("Tried to assign_widths of flow: f%d", self.id()))
        }
    }

    pub fn assign_height_inorder(&self, ctx: &mut LayoutContext) {
        match *self {
            BlockFlow(info)  => info.assign_height_inorder_block(ctx),
            InlineFlow(info) => info.assign_height_inorder_inline(ctx),
            FloatFlow(info)  => info.assign_height_inorder_float(ctx),
            _ => fail!(fmt!("Tried to assign_height of flow: f%d", self.id()))
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
}

impl<V,CV> FlowContext<V,CV> {
    /// A convenience method to return the position of this flow. Fails if the flow is currently
    /// being borrowed mutably.
    #[inline(always)]
    pub fn position(&self) -> Rect<Au> {
        do self.with_base |common_info| {
            common_info.position
        }
    }

    /// Convenience method to return whether this flow should be inorder or bottom-up on the assign-
    /// heights traversal.
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

    

    pub fn inline(&self) -> @mut InlineFlowData<V,CV> {
        match *self {
            InlineFlow(info) => info,
            _ => fail!(fmt!("Tried to access inline data of non-inline: f%d", self.id()))
        }
    }

    pub fn block(&self) -> @mut BlockFlowData<V,CV> {
        match *self {
            BlockFlow(info) => info,
            _ => fail!(fmt!("Tried to access block data of non-block: f%d", self.id()))
        }
    }

    pub fn root(&self) -> @mut BlockFlowData<V,CV> {
        match *self {
            BlockFlow(info) if info.is_root => info,
            _ => fail!(fmt!("Tried to access root block data of non-root: f%d", self.id()))
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

    pub fn iter_all_boxes(&self, cb: &fn(RenderBox) -> bool) -> bool {
        match *self {
            BlockFlow(block) => {
                let block = &mut *block;
                for block.box.iter().advance |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            InlineFlow(inline) => {
                let inline = &mut *inline;
                for inline.boxes.iter().advance |box| {
                    if !cb(*box) {
                        break;
                    }
                }
            }
            _ => fail!(fmt!("Don't know how to iterate node's RenderBoxes for %?", self))
        }

        true
    }

    pub fn iter_boxes_for_node(&self,
                               node: AbstractNode<LayoutView>,
                               callback: &fn(RenderBox) -> bool)
                               -> bool {
        for self.iter_all_boxes |box| {
            if box.node() == node {
                if !callback(box) {
                    break;
                }
            }
        }

        true
    }
}

impl FlowContext<SequentialView,SequentialView> {
    /// Dumps the flow tree for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the flow tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: uint) {
        let mut s = ~"|";
        for uint::range(0, indent) |_i| {
            s.push_str("---- ");
        }

        s.push_str(self.debug_str());
        stderr().write_line(s);

        // FIXME: this should have a pure/const version?
        for self.each_child |child| {
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

