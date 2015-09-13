/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Routines for handling measuring the memory usage of arbitrary DOM nodes.

use dom::bindings::codegen::InheritTypes::*;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElementTypeId;
use dom::htmlmediaelement::HTMLMediaElementTypeId::HTMLAudioElement;
use dom::htmlmediaelement::HTMLMediaElementTypeId::HTMLVideoElement;
use dom::htmltablecellelement::HTMLTableCellElementTypeId::HTMLTableDataCellElement;
use dom::htmltablecellelement::HTMLTableCellElementTypeId::HTMLTableHeaderCellElement;
use dom::node::NodeTypeId;
use libc;
use util::mem::{HeapSizeOf, heap_size_of};

// This is equivalent to measuring a Box<T>, except that DOM objects lose their
// associated box in order to stash their pointers in a reserved slot of their
// JS reflector. It is assumed that the caller passes a pointer to the most-derived
// type that this pointer represents, or the actual heap usage of the pointee will
// be under-reported.
fn heap_size_of_self_and_children<T: HeapSizeOf>(obj: &T) -> usize {
    heap_size_of(obj as *const T as *const libc::c_void) + obj.heap_size_of_children()
}

pub fn heap_size_of_eventtarget(target: &EventTarget) -> usize {
    //TODO: add more specific matches for concrete element types as derive(HeapSizeOf) is
    //      added to each one.
    match target.type_id() {
        &EventTargetTypeId::Window =>
            heap_size_of_self_and_children(WindowCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::CharacterData(_)) =>
            heap_size_of_self_and_children(CharacterDataCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Document) =>
            heap_size_of_self_and_children(DocumentCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::Element)) =>
            heap_size_of_self_and_children(ElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLElement))) =>
            heap_size_of_self_and_children(HTMLElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement))) =>
            heap_size_of_self_and_children(HTMLAnchorElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement))) =>
            heap_size_of_self_and_children(HTMLAppletElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement))) =>
            heap_size_of_self_and_children(HTMLAreaElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBaseElement))) =>
            heap_size_of_self_and_children(HTMLBaseElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBRElement))) =>
            heap_size_of_self_and_children(HTMLBRElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement))) =>
            heap_size_of_self_and_children(HTMLBodyElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement))) =>
            heap_size_of_self_and_children(HTMLButtonElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement))) =>
            heap_size_of_self_and_children(HTMLCanvasElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDataElement))) =>
            heap_size_of_self_and_children(HTMLDataElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDataListElement))) =>
            heap_size_of_self_and_children(HTMLDataListElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDialogElement))) =>
            heap_size_of_self_and_children(HTMLDialogElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDirectoryElement))) =>
            heap_size_of_self_and_children(HTMLDirectoryElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDListElement))) =>
            heap_size_of_self_and_children(HTMLDListElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDivElement))) =>
            heap_size_of_self_and_children(HTMLDivElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLEmbedElement))) =>
            heap_size_of_self_and_children(HTMLEmbedElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement))) =>
            heap_size_of_self_and_children(HTMLFieldSetElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement))) =>
            heap_size_of_self_and_children(HTMLFontElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement))) =>
            heap_size_of_self_and_children(HTMLFormElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameElement))) =>
            heap_size_of_self_and_children(HTMLFrameElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement))) =>
            heap_size_of_self_and_children(HTMLFrameSetElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHRElement))) =>
            heap_size_of_self_and_children(HTMLHRElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement))) =>
            heap_size_of_self_and_children(HTMLHeadElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadingElement))) =>
            heap_size_of_self_and_children(HTMLHeadingElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHtmlElement))) =>
            heap_size_of_self_and_children(HTMLHtmlElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement))) =>
            heap_size_of_self_and_children(HTMLIFrameElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement))) =>
            heap_size_of_self_and_children(HTMLImageElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement))) =>
            heap_size_of_self_and_children(HTMLInputElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement))) =>
            heap_size_of_self_and_children(HTMLLabelElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLegendElement))) =>
            heap_size_of_self_and_children(HTMLLegendElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement))) =>
            heap_size_of_self_and_children(HTMLLinkElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLIElement))) =>
            heap_size_of_self_and_children(HTMLLIElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMapElement))) =>
            heap_size_of_self_and_children(HTMLMapElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(
        HTMLElementTypeId::HTMLMediaElement(HTMLAudioElement)))) =>
            heap_size_of_self_and_children(HTMLAudioElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(
        HTMLElementTypeId::HTMLMediaElement(HTMLVideoElement)))) =>
            heap_size_of_self_and_children(HTMLVideoElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMetaElement))) =>
            heap_size_of_self_and_children(HTMLMetaElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMeterElement))) =>
            heap_size_of_self_and_children(HTMLMeterElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLModElement))) =>
            heap_size_of_self_and_children(HTMLModElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement))) =>
            heap_size_of_self_and_children(HTMLObjectElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOListElement))) =>
            heap_size_of_self_and_children(HTMLOListElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement))) =>
            heap_size_of_self_and_children(HTMLOptGroupElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement))) =>
            heap_size_of_self_and_children(HTMLOptionElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement))) =>
            heap_size_of_self_and_children(HTMLOutputElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLParagraphElement))) =>
            heap_size_of_self_and_children(HTMLParagraphElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLParamElement))) =>
            heap_size_of_self_and_children(HTMLParamElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLPreElement))) =>
            heap_size_of_self_and_children(HTMLPreElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLProgressElement))) =>
            heap_size_of_self_and_children(HTMLProgressElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLQuoteElement))) =>
            heap_size_of_self_and_children(HTMLQuoteElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement))) =>
            heap_size_of_self_and_children(HTMLScriptElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement))) =>
            heap_size_of_self_and_children(HTMLSelectElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSourceElement))) =>
            heap_size_of_self_and_children(HTMLSourceElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSpanElement))) =>
            heap_size_of_self_and_children(HTMLSpanElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement))) =>
            heap_size_of_self_and_children(HTMLStyleElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement))) =>
            heap_size_of_self_and_children(HTMLTableElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCaptionElement))) =>
            heap_size_of_self_and_children(HTMLTableCaptionElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(
        HTMLElementTypeId::HTMLTableCellElement(HTMLTableDataCellElement)))) =>
            heap_size_of_self_and_children(HTMLTableDataCellElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(
        HTMLElementTypeId::HTMLTableCellElement(HTMLTableHeaderCellElement)))) =>
            heap_size_of_self_and_children(HTMLTableHeaderCellElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableColElement))) =>
            heap_size_of_self_and_children(HTMLTableColElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement))) =>
            heap_size_of_self_and_children(HTMLTableRowElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement))) =>
            heap_size_of_self_and_children(HTMLTableSectionElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement))) =>
            heap_size_of_self_and_children(HTMLTemplateElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))) =>
            heap_size_of_self_and_children(HTMLTextAreaElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTimeElement))) =>
            heap_size_of_self_and_children(HTMLTimeElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement))) =>
            heap_size_of_self_and_children(HTMLTitleElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTrackElement))) =>
            heap_size_of_self_and_children(HTMLTrackElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLUListElement))) =>
            heap_size_of_self_and_children(HTMLUListElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::Element(
        ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLUnknownElement))) =>
            heap_size_of_self_and_children(HTMLUnknownElementCast::to_ref(target).unwrap()),
        &EventTargetTypeId::WebSocket => 0,
        &EventTargetTypeId::Worker => 0,
        &EventTargetTypeId::FileReader => 0,
        &EventTargetTypeId::WorkerGlobalScope(_) => 0,
        &EventTargetTypeId::XMLHttpRequestEventTarget(_) => 0,
        &EventTargetTypeId::Node(NodeTypeId::DocumentType) =>
            heap_size_of_self_and_children(DocumentTypeCast::to_ref(target).unwrap()),
        &EventTargetTypeId::Node(NodeTypeId::DocumentFragment) =>
            heap_size_of_self_and_children(DocumentFragmentCast::to_ref(target).unwrap()),
    }
}
