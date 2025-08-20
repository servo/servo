/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use base::id::{BrowsingContextId, PipelineId};
use html5ever::{local_name, ns};
use layout_api::wrapper_traits::{LayoutDataTrait, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use layout_api::{
    GenericLayoutDataTrait, LayoutDamage, LayoutElementType,
    LayoutNodeType as ScriptLayoutNodeType, SVGElementData,
};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::image_cache::Image;
use script::layout_dom::ServoThreadSafeLayoutNode;
use servo_arc::Arc as ServoArc;
use smallvec::SmallVec;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use style::selector_parser::{PseudoElement, RestyleDamage};

use crate::cell::ArcRefCell;
use crate::flexbox::FlexLevelBox;
use crate::flow::BlockLevelBox;
use crate::flow::inline::{InlineItem, SharedInlineStyles};
use crate::fragment_tree::Fragment;
use crate::geom::PhysicalSize;
use crate::layout_box_base::LayoutBoxBase;
use crate::replaced::CanvasInfo;
use crate::table::TableLevelBox;
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
            .map(|layout_box| layout_box.with_base_flat(LayoutBoxBase::fragments))
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

    fn clear_fragment_layout_cache(&self) {
        if let Some(data) = self.self_box.borrow().as_ref() {
            data.clear_fragment_layout_cache();
        }
        for pseudo_layout_data in self.pseudo_boxes.iter() {
            pseudo_layout_data
                .data
                .borrow()
                .clear_fragment_layout_cache();
        }
    }
}

/// A box that is stored in one of the `DOMLayoutData` slots.
#[derive(MallocSizeOf)]
pub(super) enum LayoutBox {
    DisplayContents(SharedInlineStyles),
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(Vec<ArcRefCell<InlineItem>>),
    FlexLevel(ArcRefCell<FlexLevelBox>),
    TableLevelBox(TableLevelBox),
    TaffyItemBox(ArcRefCell<TaffyItemBox>),
}

impl LayoutBox {
    fn clear_fragment_layout_cache(&self) {
        match self {
            LayoutBox::DisplayContents(..) => {},
            LayoutBox::BlockLevel(block_level_box) => {
                block_level_box.borrow().clear_fragment_layout_cache()
            },
            LayoutBox::InlineLevel(inline_items) => {
                for inline_item in inline_items.iter() {
                    inline_item.borrow().clear_fragment_layout_cache()
                }
            },
            LayoutBox::FlexLevel(flex_level_box) => {
                flex_level_box.borrow().clear_fragment_layout_cache()
            },
            LayoutBox::TaffyItemBox(taffy_item_box) => {
                taffy_item_box.borrow_mut().clear_fragment_layout_cache()
            },
            LayoutBox::TableLevelBox(table_box) => table_box.clear_fragment_layout_cache(),
        }
    }

    pub(crate) fn with_base_flat<T>(&self, callback: impl Fn(&LayoutBoxBase) -> Vec<T>) -> Vec<T> {
        match self {
            LayoutBox::DisplayContents(..) => vec![],
            LayoutBox::BlockLevel(block_level_box) => block_level_box.borrow().with_base(callback),
            LayoutBox::InlineLevel(inline_items) => inline_items
                .iter()
                .flat_map(|inline_item| inline_item.borrow().with_base(&callback))
                .collect(),
            LayoutBox::FlexLevel(flex_level_box) => flex_level_box.borrow().with_base(callback),
            LayoutBox::TaffyItemBox(taffy_item_box) => taffy_item_box.borrow().with_base(callback),
            LayoutBox::TableLevelBox(table_box) => table_box.with_base(callback),
        }
    }

    pub(crate) fn with_base_mut(&mut self, callback: impl Fn(&mut LayoutBoxBase)) {
        match self {
            LayoutBox::DisplayContents(..) => {},
            LayoutBox::BlockLevel(block_level_box) => {
                block_level_box.borrow_mut().with_base_mut(callback);
            },
            LayoutBox::InlineLevel(inline_items) => {
                for inline_item in inline_items {
                    inline_item.borrow_mut().with_base_mut(&callback);
                }
            },
            LayoutBox::FlexLevel(flex_level_box) => {
                flex_level_box.borrow_mut().with_base_mut(callback)
            },
            LayoutBox::TableLevelBox(table_level_box) => table_level_box.with_base_mut(callback),
            LayoutBox::TaffyItemBox(taffy_item_box) => {
                taffy_item_box.borrow_mut().with_base_mut(callback)
            },
        }
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
                *inline_shared_styles.selected.borrow_mut() = node.selected_style();
            },
            LayoutBox::BlockLevel(block_level_box) => {
                block_level_box
                    .borrow_mut()
                    .repair_style(context, node, new_style);
            },
            LayoutBox::InlineLevel(inline_items) => {
                for inline_item in inline_items {
                    inline_item
                        .borrow_mut()
                        .repair_style(context, node, new_style);
                }
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

    /// If this [`LayoutBox`] represents an unsplit (due to inline-block splits) inline
    /// level item, unwrap and return it. If not, return `None`.
    pub(crate) fn unsplit_inline_level_layout_box(self) -> Option<ArcRefCell<InlineItem>> {
        let LayoutBox::InlineLevel(inline_level_boxes) = self else {
            return None;
        };
        // If this element box has been subject to inline-block splitting, ignore it. It's
        // not useful currently for incremental box tree construction.
        if inline_level_boxes.len() != 1 {
            return None;
        }
        inline_level_boxes.into_iter().next()
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
    pub(crate) slot: Option<ArcRefCell<Option<LayoutBox>>>,
    pub(crate) marker: PhantomData<&'dom ()>,
}

impl From<ArcRefCell<Option<LayoutBox>>> for BoxSlot<'_> {
    fn from(layout_box_slot: ArcRefCell<Option<LayoutBox>>) -> Self {
        let slot = Some(layout_box_slot);
        Self {
            slot,
            marker: PhantomData,
        }
    }
}

/// A mutable reference to a `LayoutBox` stored in a DOM element.
impl BoxSlot<'_> {
    pub(crate) fn set(mut self, box_: LayoutBox) {
        if let Some(slot) = &mut self.slot {
            *slot.borrow_mut() = Some(box_);
        }
    }

    pub(crate) fn take_layout_box_if_undamaged(&self, damage: LayoutDamage) -> Option<LayoutBox> {
        if damage.has_box_damage() {
            return None;
        }
        self.slot.as_ref().and_then(|slot| slot.borrow_mut().take())
    }
}

impl Drop for BoxSlot<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            if let Some(slot) = &mut self.slot {
                assert!(slot.borrow().is_some(), "failed to set a layout box");
            }
        }
    }
}

pub(crate) trait NodeExt<'dom> {
    /// Returns the image if it’s loaded, and its size in image pixels
    /// adjusted for `image_density`.
    fn as_image(&self) -> Option<(Option<Image>, PhysicalSize<f64>)>;
    fn as_canvas(&self) -> Option<(CanvasInfo, PhysicalSize<f64>)>;
    fn as_iframe(&self) -> Option<(PipelineId, BrowsingContextId)>;
    fn as_video(&self) -> Option<(Option<webrender_api::ImageKey>, Option<PhysicalSize<f64>>)>;
    fn as_svg(&self) -> Option<SVGElementData>;
    fn as_typeless_object_with_data_attribute(&self) -> Option<String>;

    fn ensure_inner_layout_data(&self) -> AtomicRefMut<'dom, InnerDOMLayoutData>;
    fn inner_layout_data(&self) -> Option<AtomicRef<'dom, InnerDOMLayoutData>>;
    fn box_slot(&self) -> BoxSlot<'dom>;

    /// Remove boxes for the element itself, and all of its pseudo-element boxes.
    fn unset_all_boxes(&self);

    /// Remove all pseudo-element boxes for this element.
    fn unset_all_pseudo_boxes(&self);

    fn fragments_for_pseudo(&self, pseudo_element: Option<PseudoElement>) -> Vec<Fragment>;
    fn clear_fragment_layout_cache(&self);

    fn repair_style(&self, context: &SharedStyleContext);
    fn take_restyle_damage(&self) -> LayoutDamage;
}

impl<'dom> NodeExt<'dom> for ServoThreadSafeLayoutNode<'dom> {
    fn as_image(&self) -> Option<(Option<Image>, PhysicalSize<f64>)> {
        let (resource, metadata) = self.image_data()?;
        let (width, height) = resource
            .as_ref()
            .map(|image| {
                let image_metadata = image.metadata();
                (image_metadata.width, image_metadata.height)
            })
            .or_else(|| metadata.map(|metadata| (metadata.width, metadata.height)))
            .unwrap_or((0, 0));
        let (mut width, mut height) = (width as f64, height as f64);
        if let Some(density) = self.image_density().filter(|density| *density != 1.) {
            width /= density;
            height /= density;
        }
        Some((resource, PhysicalSize::new(width, height)))
    }

    fn as_svg(&self) -> Option<SVGElementData> {
        self.svg_data()
    }

    fn as_video(&self) -> Option<(Option<webrender_api::ImageKey>, Option<PhysicalSize<f64>>)> {
        let data = self.media_data()?;
        let natural_size = if let Some(frame) = data.current_frame {
            Some(PhysicalSize::new(frame.width.into(), frame.height.into()))
        } else {
            data.metadata
                .map(|meta| PhysicalSize::new(meta.width.into(), meta.height.into()))
        };
        Some((
            data.current_frame.map(|frame| frame.image_key),
            natural_size,
        ))
    }

    fn as_canvas(&self) -> Option<(CanvasInfo, PhysicalSize<f64>)> {
        let canvas_data = self.canvas_data()?;
        let source = canvas_data.source;
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

    fn unset_all_pseudo_boxes(&self) {
        self.ensure_inner_layout_data().pseudo_boxes.clear();
    }

    fn clear_fragment_layout_cache(&self) {
        if let Some(inner_layout_data) = self.inner_layout_data() {
            inner_layout_data.clear_fragment_layout_cache();
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

    fn take_restyle_damage(&self) -> LayoutDamage {
        let damage = self
            .style_data()
            .map(|style_data| std::mem::take(&mut style_data.element_data.borrow_mut().damage))
            .unwrap_or_else(RestyleDamage::reconstruct);
        LayoutDamage::from_bits_retain(damage.bits())
    }
}
