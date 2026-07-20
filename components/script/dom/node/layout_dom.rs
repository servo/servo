/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Methods for layout of node

use std::borrow::Cow;

use layout_api::{
    GenericLayoutData, HTMLCanvasData, HTMLMediaData, LayoutElementType, LayoutNodeType,
    SVGElementData, SharedSelection,
};
use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use script_bindings::codegen::InheritTypes::{
    ElementTypeId, HTMLElementTypeId, SVGElementTypeId, SVGGraphicsElementTypeId,
};
use servo_base::id::{BrowsingContextId, PipelineId};
use servo_url::ServoUrl;
use style::dom::OpaqueNode;
use style::selector_parser::PseudoElement;

use crate::dom::bindings::inheritance::{CharacterDataTypeId, NodeTypeId};
use crate::dom::bindings::root::{LayoutDom, ToLayout, ToLayoutOptional};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlslotelement::HTMLSlotElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::svg::svgsvgelement::SVGSVGElement;
use crate::dom::text::Text;
use crate::dom::{Node, NodeFlags};

impl<'dom> LayoutDom<'dom, Node> {
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().parent_node().to_layout() }
    }

    #[inline]
    pub(crate) fn type_id_for_layout(self) -> NodeTypeId {
        self.unsafe_get().type_id()
    }

    #[inline]
    pub(crate) fn is_element_for_layout(&self) -> bool {
        (*self).is::<Element>()
    }

    pub(crate) fn is_text_node_for_layout(&self) -> bool {
        matches!(
            self.type_id_for_layout(),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(..))
        )
    }

    #[inline]
    pub(crate) fn composed_parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        let parent = self.parent_node_ref();
        if let Some(parent) = parent &&
            let Some(shadow_root) = parent.downcast::<ShadowRoot>()
        {
            return Some(shadow_root.get_host_for_layout().upcast());
        }
        parent
    }

    #[inline]
    pub(crate) fn traversal_parent(self) -> Option<LayoutDom<'dom, Element>> {
        if let Some(assigned_slot) = self.assigned_slot_for_layout() {
            return Some(assigned_slot.upcast());
        }
        let parent = self.parent_node_ref()?;
        if let Some(shadow) = parent.downcast::<ShadowRoot>() {
            return Some(shadow.get_host_for_layout());
        };
        parent.downcast()
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn first_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().first_child().to_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn last_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().last_child().to_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn prev_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().prev_sibling().to_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn next_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().next_sibling().to_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn owner_doc_for_layout(self) -> LayoutDom<'dom, Document> {
        unsafe { self.unsafe_get().get_owner_doc().to_layout().unwrap() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn containing_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>> {
        unsafe {
            self.unsafe_get()
                .get_rare_data()
                .borrow_for_layout()
                .as_ref()?
                .containing_shadow_root
                .as_ref()
                .map(|sr| sr.to_layout())
        }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn assigned_slot_for_layout(self) -> Option<LayoutDom<'dom, HTMLSlotElement>> {
        unsafe {
            self.unsafe_get()
                .get_rare_data()
                .borrow_for_layout()
                .as_ref()?
                .slottable_data
                .assigned_slot
                .as_ref()
                .map(|assigned_slot| assigned_slot.to_layout())
        }
    }

    // FIXME(nox): get_flag/set_flag (especially the latter) are not safe because
    // they mutate stuff while values of this type can be used from multiple
    // threads at once, this should be revisited.

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn get_flag(self, flag: NodeFlags) -> bool {
        (self.unsafe_get()).flags().get().contains(flag)
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn set_flag(self, flag: NodeFlags, value: bool) {
        let this = self.unsafe_get();
        let mut flags = (this).flags().get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        (this).flags().set(flags);
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn layout_data(self) -> Option<&'dom GenericLayoutData> {
        unsafe {
            self.unsafe_get()
                .layout_data()
                .borrow_for_layout()
                .as_deref()
        }
    }

    /// Initialize the style data of this node.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn initialize_layout_data(self, new_data: Box<GenericLayoutData>) {
        let data = unsafe { self.unsafe_get().layout_data().borrow_mut_for_layout() };
        debug_assert!(data.is_none());
        *data = Some(new_data);
    }

    /// Clear the style and opaque layout data of this node.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn clear_layout_data(self) {
        unsafe {
            self.unsafe_get()
                .layout_data()
                .borrow_mut_for_layout()
                .take();
        }
    }

    /// Whether this element serve as a container of editable text for a text input
    /// that is implemented as an UA widget.
    pub(crate) fn is_single_line_text_inner_editor(&self) -> bool {
        matches!(
            self.implemented_pseudo_element(),
            Some(PseudoElement::ServoTextControlInnerEditor)
        )
    }

    /// Whether this element serve as a container of any text inside a text input
    /// that is implemented as an UA widget.
    pub(crate) fn is_text_container_of_single_line_input(&self) -> bool {
        let is_single_line_text_inner_placeholder = matches!(
            self.implemented_pseudo_element(),
            Some(PseudoElement::Placeholder)
        );
        // Currently `::placeholder` is only implemented for single line text input element.
        debug_assert!(
            !is_single_line_text_inner_placeholder ||
                self.containing_shadow_root_for_layout()
                    .map(|root| root.get_host_for_layout())
                    .map(|host| host.downcast::<HTMLInputElement>())
                    .is_some()
        );

        self.is_single_line_text_inner_editor() || is_single_line_text_inner_placeholder
    }

    pub(crate) fn text_content(self) -> Cow<'dom, str> {
        self.downcast::<Text>()
            .expect("Called LayoutDom::text_content on non-Text node!")
            .upcast()
            .data_for_layout()
            .into()
    }

    /// Get the selection for the given node. This only works for text nodes that are in
    /// the shadow DOM of user agent widgets for form controls, specifically for `<input>`
    /// and `<textarea>`.
    ///
    /// As we want to expose the selection on the inner text node of the widget's shadow
    /// DOM, we must find the shadow root and then access the containing element itself.
    pub(crate) fn selection(self) -> Option<SharedSelection> {
        if let Some(input) = self.downcast::<HTMLInputElement>() {
            return input.selection_for_layout();
        }
        if let Some(textarea) = self.downcast::<HTMLTextAreaElement>() {
            return Some(textarea.selection_for_layout());
        }

        let shadow_root = self
            .containing_shadow_root_for_layout()?
            .get_host_for_layout();
        if let Some(input) = shadow_root.downcast::<HTMLInputElement>() {
            return input.selection_for_layout();
        }
        shadow_root
            .downcast::<HTMLTextAreaElement>()
            .map(|textarea| textarea.selection_for_layout())
    }

    pub(crate) fn image_url(self) -> Option<ServoUrl> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_url()
    }

    pub(crate) fn image_data(self) -> Option<(Option<Image>, Option<ImageMetadata>)> {
        self.downcast::<HTMLImageElement>().map(|e| e.image_data())
    }

    pub(crate) fn image_density(self) -> Option<f64> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_density()
    }

    pub(crate) fn showing_broken_image_icon(self) -> bool {
        self.downcast::<HTMLImageElement>()
            .map(|image_element| image_element.showing_broken_image_icon())
            .unwrap_or_default()
    }

    pub(crate) fn canvas_data(self) -> Option<HTMLCanvasData> {
        self.downcast::<HTMLCanvasElement>()
            .map(|canvas| canvas.data())
    }

    pub(crate) fn media_data(self) -> Option<HTMLMediaData> {
        self.downcast::<HTMLVideoElement>()
            .map(|media| media.data())
    }

    pub(crate) fn svg_data(self) -> Option<SVGElementData<'dom>> {
        self.downcast::<SVGSVGElement>().map(|svg| svg.data())
    }

    pub(crate) fn iframe_browsing_context_id(self) -> Option<BrowsingContextId> {
        self.downcast::<HTMLIFrameElement>()
            .and_then(|iframe_element| iframe_element.browsing_context_id())
    }

    pub(crate) fn iframe_pipeline_id(self) -> Option<PipelineId> {
        self.downcast::<HTMLIFrameElement>()
            .and_then(|iframe_element| iframe_element.pipeline_id())
    }

    #[expect(unsafe_code)]
    pub(crate) fn opaque(self) -> OpaqueNode {
        unsafe { OpaqueNode(self.get_jsobject() as usize) }
    }

    #[expect(unsafe_code)]
    pub(crate) fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        unsafe {
            self.unsafe_get()
                .get_rare_data()
                .borrow_for_layout()
                .as_ref()
                .and_then(|rare_data| rare_data.implemented_pseudo_element)
        }
    }

    pub(crate) fn is_in_ua_widget(&self) -> bool {
        self.unsafe_get().is_in_ua_widget()
    }

    pub(crate) fn is_root_of_user_agent_widget(&self) -> bool {
        self.downcast::<Element>().is_some_and(|element| {
            element
                .get_shadow_root_for_layout()
                .is_some_and(|shadow_root| shadow_root.is_user_agent_widget())
        })
    }

    pub(crate) fn children_count(&self) -> u32 {
        self.unsafe_get().children_count()
    }
}

pub(crate) struct NodeTypeIdWrapper(pub(crate) NodeTypeId);

impl From<NodeTypeIdWrapper> for LayoutNodeType {
    #[inline(always)]
    fn from(node_type: NodeTypeIdWrapper) -> LayoutNodeType {
        match node_type.0 {
            NodeTypeId::Element(e) => LayoutNodeType::Element(ElementTypeIdWrapper(e).into()),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => LayoutNodeType::Text,
            x => unreachable!("Layout should not traverse nodes of type {:?}", x),
        }
    }
}

struct ElementTypeIdWrapper(ElementTypeId);

impl From<ElementTypeIdWrapper> for LayoutElementType {
    #[inline(always)]
    fn from(element_type: ElementTypeIdWrapper) -> LayoutElementType {
        match element_type.0 {
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement) => {
                LayoutElementType::HTMLBodyElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement) => {
                LayoutElementType::HTMLButtonElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBRElement) => {
                LayoutElementType::HTMLBRElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement) => {
                LayoutElementType::HTMLCanvasElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHtmlElement) => {
                LayoutElementType::HTMLHtmlElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement) => {
                LayoutElementType::HTMLIFrameElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement) => {
                LayoutElementType::HTMLImageElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(_)) => {
                LayoutElementType::HTMLMediaElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement) => {
                LayoutElementType::HTMLInputElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement) => {
                LayoutElementType::HTMLOptGroupElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement) => {
                LayoutElementType::HTMLOptionElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement) => {
                LayoutElementType::HTMLObjectElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLParagraphElement) => {
                LayoutElementType::HTMLParagraphElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLPreElement) => {
                LayoutElementType::HTMLPreElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement) => {
                LayoutElementType::HTMLSelectElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement) => {
                LayoutElementType::HTMLTableCellElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableColElement) => {
                LayoutElementType::HTMLTableColElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement) => {
                LayoutElementType::HTMLTableElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement) => {
                LayoutElementType::HTMLTableRowElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement) => {
                LayoutElementType::HTMLTableSectionElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement) => {
                LayoutElementType::HTMLTextAreaElement
            },
            ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
                SVGGraphicsElementTypeId::SVGImageElement,
            )) => LayoutElementType::SVGImageElement,
            ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
                SVGGraphicsElementTypeId::SVGSVGElement,
            )) => LayoutElementType::SVGSVGElement,
            _ => LayoutElementType::Element,
        }
    }
}
