/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use base::id::{BrowsingContextId, PipelineId};
use html5ever::{local_name, ns};
use layout_api::wrapper_traits::{LayoutDataTrait, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use layout_api::{
    GenericLayoutDataTrait, LayoutElementType, LayoutNodeType as ScriptLayoutNodeType,
    SVGElementData,
};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::image_cache::Image;
use script::layout_dom::ServoThreadSafeLayoutNode;
use servo_arc::Arc as ServoArc;
use smallvec::SmallVec;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::specified::box_::DisplayOutside as StyloDisplayOutside;

use crate::cell::{ArcRefCell, WeakRefCell};
use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexLevelBox;
use crate::flow::inline::{InlineItem, SharedInlineStyles, WeakInlineItem};
use crate::flow::{BlockLevelBox, BlockLevelCreator};
use crate::fragment_tree::{Fragment, FragmentFlags};
use crate::geom::PhysicalSize;
use crate::layout_box_base::LayoutBoxBase;
use crate::replaced::{CanvasInfo, VideoInfo};
use crate::style_ext::{
    ComputedValuesExt, Display, DisplayGeneratingBox, DisplayLayoutInternal, DisplayOutside,
};
use crate::table::{TableLevelBox, WeakTableLevelBox};
use crate::taffy::TaffyItemBox;

#[derive(MallocSizeOf)]
pub struct PseudoLayoutData {
    pseudo: PseudoElement,
    data: ArcRefCell<InnerDOMLayoutData>,
}

/// The data that is stored in each DOM node that is used by layout.
#[derive(Default, MallocSizeOf)]
pub struct InnerDOMLayoutData {
    pub(super) self_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_boxes: SmallVec<[PseudoLayoutData; 2]>,
}

impl InnerDOMLayoutData {
    fn pseudo_layout_data(
        &self,
        pseudo_element: PseudoElement,
    ) -> Option<ArcRefCell<InnerDOMLayoutData>> {
        for pseudo_layout_data in self.pseudo_boxes.iter() {
            if pseudo_element == pseudo_layout_data.pseudo {
                return Some(pseudo_layout_data.data.clone());
            }
        }
        None
    }

    fn create_pseudo_layout_data(
        &mut self,
        pseudo_element: PseudoElement,
    ) -> ArcRefCell<InnerDOMLayoutData> {
        let data: ArcRefCell<InnerDOMLayoutData> = Default::default();
        self.pseudo_boxes.push(PseudoLayoutData {
            pseudo: pseudo_element,
            data: data.clone(),
        });
        data
    }

    fn fragments(&self) -> Vec<Fragment> {
        self.self_box
            .borrow()
            .as_ref()
            .and_then(|layout_box| layout_box.with_base(LayoutBoxBase::fragments))
            .unwrap_or_default()
    }

    fn repair_style(&self, node: &ServoThreadSafeLayoutNode, context: &SharedStyleContext) {
        if let Some(layout_object) = &*self.self_box.borrow() {
            layout_object.repair_style(context, node, &node.style(context));
        }

        for pseudo_layout_data in self.pseudo_boxes.iter() {
            let Some(node_with_pseudo) = node.with_pseudo(pseudo_layout_data.pseudo) else {
                continue;
            };
            pseudo_layout_data
                .data
                .borrow()
                .repair_style(&node_with_pseudo, context);
        }
    }

    fn with_layout_box_base(&self, callback: impl Fn(&LayoutBoxBase)) {
        if let Some(data) = self.self_box.borrow().as_ref() {
            data.with_base(callback);
        }
    }

    fn with_layout_box_base_including_pseudos(&self, callback: impl Fn(&LayoutBoxBase)) {
        self.with_layout_box_base(&callback);
        for pseudo_layout_data in self.pseudo_boxes.iter() {
            pseudo_layout_data
                .data
                .borrow()
                .with_layout_box_base(&callback);
        }
    }
}

/// A box that is stored in one of the `DOMLayoutData` slots.
#[derive(Debug, MallocSizeOf)]
pub(super) enum LayoutBox {
    DisplayContents(SharedInlineStyles),
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(InlineItem),
    FlexLevel(ArcRefCell<FlexLevelBox>),
    TableLevelBox(TableLevelBox),
    TaffyItemBox(ArcRefCell<TaffyItemBox>),
}

impl LayoutBox {
    pub(crate) fn with_base<T>(&self, callback: impl FnOnce(&LayoutBoxBase) -> T) -> Option<T> {
        Some(match self {
            LayoutBox::DisplayContents(..) => return None,
            LayoutBox::BlockLevel(block_level_box) => block_level_box.borrow().with_base(callback),
            LayoutBox::InlineLevel(inline_item) => inline_item.with_base(callback),
            LayoutBox::FlexLevel(flex_level_box) => flex_level_box.borrow().with_base(callback),
            LayoutBox::TaffyItemBox(taffy_item_box) => taffy_item_box.borrow().with_base(callback),
            LayoutBox::TableLevelBox(table_box) => table_box.with_base(callback),
        })
    }

    pub(crate) fn with_base_mut<T>(
        &mut self,
        callback: impl FnOnce(&mut LayoutBoxBase) -> T,
    ) -> Option<T> {
        Some(match self {
            LayoutBox::DisplayContents(..) => return None,
            LayoutBox::BlockLevel(block_level_box) => {
                block_level_box.borrow_mut().with_base_mut(callback)
            },
            LayoutBox::InlineLevel(inline_item) => inline_item.with_base_mut(callback),
            LayoutBox::FlexLevel(flex_level_box) => {
                flex_level_box.borrow_mut().with_base_mut(callback)
            },
            LayoutBox::TaffyItemBox(taffy_item_box) => {
                taffy_item_box.borrow_mut().with_base_mut(callback)
            },
            LayoutBox::TableLevelBox(table_box) => table_box.with_base_mut(callback),
        })
    }

    fn repair_style(
        &self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
        new_style: &ServoArc<ComputedValues>,
    ) {
        match self {
            LayoutBox::DisplayContents(inline_shared_styles) => {
                *inline_shared_styles.style.borrow_mut() = new_style.clone();
                *inline_shared_styles.selected.borrow_mut() = node.selected_style(context);
            },
            LayoutBox::BlockLevel(block_level_box) => {
                block_level_box
                    .borrow_mut()
                    .repair_style(context, node, new_style);
            },
            LayoutBox::InlineLevel(inline_item) => {
                inline_item.repair_style(context, node, new_style);
            },
            LayoutBox::FlexLevel(flex_level_box) => flex_level_box
                .borrow_mut()
                .repair_style(context, node, new_style),
            LayoutBox::TableLevelBox(table_level_box) => {
                table_level_box.repair_style(context, node, new_style)
            },
            LayoutBox::TaffyItemBox(taffy_item_box) => taffy_item_box
                .borrow_mut()
                .repair_style(context, node, new_style),
        }
    }

    fn attached_to_tree(&self, layout_box: WeakLayoutBox) {
        match self {
            Self::DisplayContents(_) => {
                // This box can't have children, its contents get reparented to its parent.
                // Therefore, no need to do anything.
            },
            Self::BlockLevel(block_level_box) => {
                block_level_box.borrow().attached_to_tree(layout_box)
            },
            Self::InlineLevel(inline_item) => inline_item.attached_to_tree(layout_box),
            Self::FlexLevel(flex_level_box) => flex_level_box.borrow().attached_to_tree(layout_box),
            Self::TableLevelBox(table_level_box) => table_level_box.attached_to_tree(layout_box),
            Self::TaffyItemBox(taffy_item_box) => {
                taffy_item_box.borrow().attached_to_tree(layout_box)
            },
        }
    }

    fn downgrade(&self) -> WeakLayoutBox {
        match self {
            Self::DisplayContents(inline_shared_styles) => {
                WeakLayoutBox::DisplayContents(inline_shared_styles.clone())
            },
            Self::BlockLevel(block_level_box) => {
                WeakLayoutBox::BlockLevel(block_level_box.downgrade())
            },
            Self::InlineLevel(inline_item) => WeakLayoutBox::InlineLevel(inline_item.downgrade()),
            Self::FlexLevel(flex_level_box) => WeakLayoutBox::FlexLevel(flex_level_box.downgrade()),
            Self::TableLevelBox(table_level_box) => {
                WeakLayoutBox::TableLevelBox(table_level_box.downgrade())
            },
            Self::TaffyItemBox(taffy_item_box) => {
                WeakLayoutBox::TaffyItemBox(taffy_item_box.downgrade())
            },
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(super) enum WeakLayoutBox {
    DisplayContents(SharedInlineStyles),
    BlockLevel(WeakRefCell<BlockLevelBox>),
    InlineLevel(WeakInlineItem),
    FlexLevel(WeakRefCell<FlexLevelBox>),
    TableLevelBox(WeakTableLevelBox),
    TaffyItemBox(WeakRefCell<TaffyItemBox>),
}

impl WeakLayoutBox {
    pub(crate) fn upgrade(&self) -> Option<LayoutBox> {
        Some(match self {
            Self::DisplayContents(inline_shared_styles) => {
                LayoutBox::DisplayContents(inline_shared_styles.clone())
            },
            Self::BlockLevel(block_level_box) => LayoutBox::BlockLevel(block_level_box.upgrade()?),
            Self::InlineLevel(inline_item) => LayoutBox::InlineLevel(inline_item.upgrade()?),
            Self::FlexLevel(flex_level_box) => LayoutBox::FlexLevel(flex_level_box.upgrade()?),
            Self::TableLevelBox(table_level_box) => {
                LayoutBox::TableLevelBox(table_level_box.upgrade()?)
            },
            Self::TaffyItemBox(taffy_item_box) => {
                LayoutBox::TaffyItemBox(taffy_item_box.upgrade()?)
            },
        })
    }
}

/// A wrapper for [`InnerDOMLayoutData`]. This is necessary to give the entire data
/// structure interior mutability, as we will need to mutate the layout data of
/// non-mutable DOM nodes.
#[derive(Default, MallocSizeOf)]
pub struct DOMLayoutData(AtomicRefCell<InnerDOMLayoutData>);

// The implementation of this trait allows the data to be stored in the DOM.
impl LayoutDataTrait for DOMLayoutData {}
impl GenericLayoutDataTrait for DOMLayoutData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct BoxSlot<'dom> {
    pub(crate) slot: ArcRefCell<Option<LayoutBox>>,
    pub(crate) marker: PhantomData<&'dom ()>,
}

impl From<ArcRefCell<Option<LayoutBox>>> for BoxSlot<'_> {
    fn from(slot: ArcRefCell<Option<LayoutBox>>) -> Self {
        Self {
            slot,
            marker: PhantomData,
        }
    }
}

/// A mutable reference to a `LayoutBox` stored in a DOM element.
impl BoxSlot<'_> {
    pub(crate) fn set(self, layout_box: LayoutBox) {
        layout_box.attached_to_tree(layout_box.downgrade());
        *self.slot.borrow_mut() = Some(layout_box);
    }

    pub(crate) fn take_layout_box(&self) -> Option<LayoutBox> {
        self.slot.borrow_mut().take()
    }
}

impl Drop for BoxSlot<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.slot.borrow().is_some(), "failed to set a layout box");
        }
    }
}

pub(crate) trait NodeExt<'dom> {
    /// Returns the image if it’s loaded, and its size in image pixels
    /// adjusted for `image_density`.
    fn as_image(&self) -> Option<(Option<Image>, PhysicalSize<f64>)>;
    fn as_canvas(&self) -> Option<(CanvasInfo, PhysicalSize<f64>)>;
    fn as_iframe(&self) -> Option<(PipelineId, BrowsingContextId)>;
    fn as_video(&self) -> Option<(VideoInfo, Option<PhysicalSize<f64>>)>;
    fn as_svg(&self) -> Option<SVGElementData<'dom>>;
    fn as_typeless_object_with_data_attribute(&self) -> Option<String>;

    fn ensure_inner_layout_data(&self) -> AtomicRefMut<'dom, InnerDOMLayoutData>;
    fn inner_layout_data(&self) -> Option<AtomicRef<'dom, InnerDOMLayoutData>>;
    fn inner_layout_data_mut(&self) -> Option<AtomicRefMut<'dom, InnerDOMLayoutData>>;
    fn box_slot(&self) -> BoxSlot<'dom>;

    /// Remove boxes for the element itself, and all of its pseudo-element boxes.
    fn unset_all_boxes(&self);

    fn fragments_for_pseudo(&self, pseudo_element: Option<PseudoElement>) -> Vec<Fragment>;
    fn with_layout_box_base_including_pseudos(&self, callback: impl Fn(&LayoutBoxBase));

    fn repair_style(&self, context: &SharedStyleContext);

    /// Whether or not this node isolates downward flowing box tree rebuild damage and
    /// fragment tree layout cache damage. Roughly, this corresponds to independent
    /// formatting context boundaries.
    ///
    /// - The node's boxes themselves will be rebuilt, but not the descendant node's
    ///   boxes.
    /// - The node's fragment tree layout will be rebuilt, not the descendent node's
    ///   fragment tree layout cache.
    ///
    /// When this node has no box yet, `false` is returned.
    fn isolates_damage_for_damage_propagation(&self) -> bool;

    /// Try to re-run box tree reconstruction from this point. This can succeed if the
    /// node itself is still valid and isolates box tree damage from ancestors (for
    /// instance, if it starts an independent formatting context). **Note:** This assumes
    /// that no ancestors have box damage.
    ///
    /// Returns `true` if box tree reconstruction was sucessful and `false` otherwise.
    fn rebuild_box_tree_from_independent_formatting_context(
        &self,
        layout_context: &LayoutContext,
    ) -> bool;
}

impl<'dom> NodeExt<'dom> for ServoThreadSafeLayoutNode<'dom> {
    fn as_image(&self) -> Option<(Option<Image>, PhysicalSize<f64>)> {
        let (resource, metadata) = self.image_data()?;
        let width = metadata.map(|metadata| metadata.width).unwrap_or_default();
        let height = metadata.map(|metadata| metadata.height).unwrap_or_default();
        let (mut width, mut height) = (width as f64, height as f64);
        if let Some(density) = self.image_density().filter(|density| *density != 1.) {
            width /= density;
            height /= density;
        }
        Some((resource, PhysicalSize::new(width, height)))
    }

    fn as_svg(&self) -> Option<SVGElementData<'dom>> {
        self.svg_data()
    }

    fn as_video(&self) -> Option<(VideoInfo, Option<PhysicalSize<f64>>)> {
        let data = self.media_data()?;
        let natural_size = if let Some(frame) = data.current_frame {
            Some(PhysicalSize::new(frame.width.into(), frame.height.into()))
        } else {
            data.metadata
                .map(|meta| PhysicalSize::new(meta.width.into(), meta.height.into()))
        };
        Some((
            VideoInfo {
                image_key: data.current_frame.map(|frame| frame.image_key),
            },
            natural_size,
        ))
    }

    fn as_canvas(&self) -> Option<(CanvasInfo, PhysicalSize<f64>)> {
        let canvas_data = self.canvas_data()?;
        let source = canvas_data.image_key;
        Some((
            CanvasInfo { source },
            PhysicalSize::new(canvas_data.width.into(), canvas_data.height.into()),
        ))
    }

    fn as_iframe(&self) -> Option<(PipelineId, BrowsingContextId)> {
        match (self.iframe_pipeline_id(), self.iframe_browsing_context_id()) {
            (Some(pipeline_id), Some(browsing_context_id)) => {
                Some((pipeline_id, browsing_context_id))
            },
            _ => None,
        }
    }

    fn as_typeless_object_with_data_attribute(&self) -> Option<String> {
        if self.type_id() !=
            Some(ScriptLayoutNodeType::Element(
                LayoutElementType::HTMLObjectElement,
            ))
        {
            return None;
        }

        // TODO: This is the what the legacy layout system did, but really if Servo
        // supports any `<object>` that's an image, it should support those with URLs
        // and `type` attributes with image mime types.
        let element = self.as_element()?;
        if element.get_attr(&ns!(), &local_name!("type")).is_some() {
            return None;
        }
        element
            .get_attr(&ns!(), &local_name!("data"))
            .map(|string| string.to_owned())
    }

    fn ensure_inner_layout_data(&self) -> AtomicRefMut<'dom, InnerDOMLayoutData> {
        if self.layout_data().is_none() {
            self.initialize_layout_data::<DOMLayoutData>();
        }
        self.layout_data()
            .unwrap()
            .as_any()
            .downcast_ref::<DOMLayoutData>()
            .unwrap()
            .0
            .borrow_mut()
    }

    fn inner_layout_data(&self) -> Option<AtomicRef<'dom, InnerDOMLayoutData>> {
        self.layout_data().map(|data| {
            data.as_any()
                .downcast_ref::<DOMLayoutData>()
                .unwrap()
                .0
                .borrow()
        })
    }

    fn inner_layout_data_mut(&self) -> Option<AtomicRefMut<'dom, InnerDOMLayoutData>> {
        self.layout_data().map(|data| {
            data.as_any()
                .downcast_ref::<DOMLayoutData>()
                .unwrap()
                .0
                .borrow_mut()
        })
    }

    fn box_slot(&self) -> BoxSlot<'dom> {
        let pseudo_element_chain = self.pseudo_element_chain();
        let Some(primary) = pseudo_element_chain.primary else {
            return self.ensure_inner_layout_data().self_box.clone().into();
        };

        let Some(secondary) = pseudo_element_chain.secondary else {
            let primary_layout_data = self
                .ensure_inner_layout_data()
                .create_pseudo_layout_data(primary);
            return primary_layout_data.borrow().self_box.clone().into();
        };

        // It's *very* important that this not borrow the element's main
        // `InnerLayoutData`. Primary pseudo-elements are processed at the same recursion
        // level as the main data, so the `BoxSlot` is created sequentially with other
        // primary pseudo-elements and the element itself. The secondary pseudo-element is
        // one level deep, so could be happening in parallel with the primary
        // pseudo-elements or main element layout.
        let primary_layout_data = self
            .inner_layout_data()
            .expect("Should already have element InnerLayoutData here.")
            .pseudo_layout_data(primary)
            .expect("Should already have primary pseudo-element InnerLayoutData here");
        let secondary_layout_data = primary_layout_data
            .borrow_mut()
            .create_pseudo_layout_data(secondary);
        secondary_layout_data.borrow().self_box.clone().into()
    }

    fn unset_all_boxes(&self) {
        let mut layout_data = self.ensure_inner_layout_data();
        *layout_data.self_box.borrow_mut() = None;
        layout_data.pseudo_boxes.clear();

        // Stylo already takes care of removing all layout data
        // for DOM descendants of elements with `display: none`.
    }

    fn with_layout_box_base_including_pseudos(&self, callback: impl Fn(&LayoutBoxBase)) {
        if let Some(inner_layout_data) = self.inner_layout_data() {
            inner_layout_data.with_layout_box_base_including_pseudos(callback);
        }
    }

    fn fragments_for_pseudo(&self, pseudo_element: Option<PseudoElement>) -> Vec<Fragment> {
        let Some(layout_data) = self.inner_layout_data() else {
            return vec![];
        };
        match pseudo_element {
            Some(pseudo_element) => layout_data
                .pseudo_layout_data(pseudo_element)
                .map(|pseudo_layout_data| pseudo_layout_data.borrow().fragments())
                .unwrap_or_default(),
            None => layout_data.fragments(),
        }
    }

    fn repair_style(&self, context: &SharedStyleContext) {
        if let Some(layout_data) = self.inner_layout_data() {
            layout_data.repair_style(self, context);
        }
    }

    fn isolates_damage_for_damage_propagation(&self) -> bool {
        // Do not run incremental box and fragment tree layout at the `<body>` or root element as
        // there is some special processing that must happen for these elements and it currently
        // only happens when doing a full box tree construction traversal.
        if self.as_element().is_some_and(|element| {
            element.is_body_element_of_html_element_root() || element.is_root()
        }) {
            return false;
        }

        let Some(inner_layout_data) = self.inner_layout_data() else {
            return false;
        };
        let self_box = inner_layout_data.self_box.borrow();
        let Some(self_box) = &*self_box else {
            return false;
        };

        match self_box {
            LayoutBox::DisplayContents(..) => false,
            LayoutBox::BlockLevel(block_level) => matches!(
                &*block_level.borrow(),
                BlockLevelBox::Independent(..) |
                    BlockLevelBox::OutOfFlowFloatBox(..) |
                    BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(..)
            ),
            LayoutBox::InlineLevel(inline_level) => matches!(
                inline_level,
                InlineItem::OutOfFlowAbsolutelyPositionedBox(..) | InlineItem::Atomic(..)
            ),
            LayoutBox::FlexLevel(..) => true,
            LayoutBox::TableLevelBox(table_level_box) => matches!(
                table_level_box,
                TableLevelBox::Cell(..) | TableLevelBox::Caption(..),
            ),
            LayoutBox::TaffyItemBox(..) => true,
        }
    }

    fn rebuild_box_tree_from_independent_formatting_context(
        &self,
        layout_context: &LayoutContext,
    ) -> bool {
        // Do not run incremental box tree layout at the `<body>` or root element as there
        // is some special processing that must happen for these elements and it currently
        // only happens when doing a full box tree construction traversal.
        if self.as_element().is_some_and(|element| {
            element.is_body_element_of_html_element_root() || element.is_root()
        }) {
            return false;
        }

        let layout_box = {
            let Some(mut inner_layout_data) = self.inner_layout_data_mut() else {
                return false;
            };
            inner_layout_data.pseudo_boxes.clear();
            inner_layout_data.self_box.clone()
        };

        let layout_box = layout_box.borrow();
        let Some(layout_box) = &*layout_box else {
            return false;
        };

        let info = NodeAndStyleInfo::new(*self, self.style(&layout_context.style_context));
        let box_style = info.style.get_box();
        let Display::GeneratingBox(display) = box_style.display.into() else {
            return false;
        };
        let contents = || {
            assert!(
                self.pseudo_element_chain().is_empty(),
                "Shouldn't try to rebuild box tree from a pseudo-element"
            );
            Contents::for_element(info.node, layout_context)
        };
        match layout_box {
            LayoutBox::DisplayContents(..) => false,
            LayoutBox::BlockLevel(block_level) => {
                let mut block_level = block_level.borrow_mut();
                match &mut *block_level {
                    BlockLevelBox::Independent(independent_formatting_context) => {
                        let DisplayGeneratingBox::OutsideInside {
                            outside: DisplayOutside::Block,
                            inside: display_inside,
                        } = display
                        else {
                            return false;
                        };
                        if !matches!(
                            BlockLevelCreator::new_for_inflow_block_level_element(
                                &info,
                                display_inside,
                                contents(),
                                independent_formatting_context.propagated_data,
                            ),
                            BlockLevelCreator::Independent { .. }
                        ) {
                            return false;
                        }
                        independent_formatting_context.rebuild(layout_context, &info);
                        true
                    },
                    BlockLevelBox::OutOfFlowFloatBox(float_box) => {
                        if !info.style.clone_float().is_floating() {
                            return false;
                        }
                        float_box.contents.rebuild(layout_context, &info);
                        true
                    },
                    BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                        // Even if absolute positioning blockifies the outer display type, if the
                        // original display was inline-level, then the box needs to be handled as
                        // an inline-level in order to compute the static position correctly.
                        // See `BlockContainerBuilder::handle_absolutely_positioned_element()`.
                        if !info.style.clone_position().is_absolutely_positioned() ||
                            box_style.original_display.outside() != StyloDisplayOutside::Block
                        {
                            return false;
                        }
                        positioned_box
                            .borrow_mut()
                            .context
                            .rebuild(layout_context, &info);
                        true
                    },
                    _ => false,
                }
            },
            LayoutBox::InlineLevel(inline_level) => match inline_level {
                InlineItem::OutOfFlowAbsolutelyPositionedBox(positioned_box, ..) => {
                    if !info.style.clone_position().is_absolutely_positioned() {
                        return false;
                    }
                    positioned_box
                        .borrow_mut()
                        .context
                        .rebuild(layout_context, &info);
                    true
                },
                InlineItem::Atomic(atomic_box, _, _) => {
                    let flags = match contents() {
                        Contents::NonReplaced(_) => FragmentFlags::empty(),
                        Contents::Replaced(_) => FragmentFlags::IS_REPLACED,
                        Contents::Widget(_) => FragmentFlags::IS_WIDGET,
                    };
                    if !info.style.is_atomic_inline_level(flags) {
                        return false;
                    }
                    atomic_box.borrow_mut().rebuild(layout_context, &info);
                    true
                },
                _ => false,
            },
            LayoutBox::FlexLevel(flex_level_box) => {
                let mut flex_level_box = flex_level_box.borrow_mut();
                match &mut *flex_level_box {
                    FlexLevelBox::FlexItem(flex_item_box) => {
                        if info.style.clone_position().is_absolutely_positioned() ||
                            flex_item_box.style().clone_order() != info.style.clone_order()
                        {
                            return false;
                        }
                        flex_item_box
                            .independent_formatting_context
                            .rebuild(layout_context, &info)
                    },
                    FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                        if !info.style.clone_position().is_absolutely_positioned() {
                            return false;
                        }
                        positioned_box
                            .borrow_mut()
                            .context
                            .rebuild(layout_context, &info);
                    },
                }
                true
            },
            LayoutBox::TableLevelBox(table_level_box) => match table_level_box {
                TableLevelBox::Caption(caption) => {
                    if display !=
                        DisplayGeneratingBox::LayoutInternal(DisplayLayoutInternal::TableCaption)
                    {
                        return false;
                    }
                    caption.borrow_mut().context.rebuild(layout_context, &info);
                    true
                },
                TableLevelBox::Cell(table_cell) => {
                    if display !=
                        DisplayGeneratingBox::LayoutInternal(DisplayLayoutInternal::TableCell)
                    {
                        return false;
                    }
                    table_cell
                        .borrow_mut()
                        .context
                        .rebuild(layout_context, &info);
                    true
                },
                _ => false,
            },
            LayoutBox::TaffyItemBox(..) => false,
        }
    }
}
