/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::types::*;
use dom::bindings::utils::{Reflectable, Reflector, Traceable};

use js::jsapi::JSTracer;

// generate_cacheable_wrapper
macro_rules! generate_cacheable_wrapper(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, element)
    )
)

macro_rules! generate_cacheable_wrapper_htmlelement(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, htmlelement)
    )
)

macro_rules! generate_cacheable_wrapper_htmlmediaelement(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, htmlmediaelement)
    )
)

macro_rules! generate_cacheable_wrapper_htmltablecellelement(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, htmltablecellelement)
    )
)

macro_rules! generate_cacheable_wrapper_node(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, node)
    )
)

macro_rules! generate_cacheable_wrapper_base(
    ($name: path, $wrap: path, $parent: ident) => (
        impl Reflectable for $name {
            fn reflector<'a>(&'a self) -> &'a Reflector {
                self.$parent.reflector()
            }

            fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
                self.$parent.mut_reflector()
            }
        }
    )
)


// generate_traceable
macro_rules! generate_traceable(
    ($name: path) => (
        generate_traceable_base!($name, element)
    )
)

macro_rules! generate_traceable_htmlelement(
    ($name: path) => (
        generate_traceable_base!($name, htmlelement)
    )
)

macro_rules! generate_traceable_htmlmediaelement(
    ($name: path) => (
        generate_traceable_base!($name, htmlmediaelement)
    )
)

macro_rules! generate_traceable_htmltablecellelement(
    ($name: path) => (
        generate_traceable_base!($name, htmltablecellelement)
    )
)

macro_rules! generate_traceable_node(
    ($name: path) => (
        generate_traceable_base!($name, node)
    )
)

macro_rules! generate_traceable_base(
    ($name: path, $parent: ident) => (
        impl Traceable for $name {
            fn trace(&self, trc: *mut JSTracer) {
                self.$parent.trace(trc);
            }
        }
    )
)


generate_cacheable_wrapper!(Comment, CommentBinding::Wrap)
generate_traceable!(Comment)

generate_cacheable_wrapper_node!(DocumentFragment, DocumentFragmentBinding::Wrap)
generate_traceable_node!(DocumentFragment)

generate_cacheable_wrapper_node!(DocumentType, DocumentTypeBinding::Wrap)
generate_traceable_node!(DocumentType)

generate_cacheable_wrapper!(Text, TextBinding::Wrap)
generate_traceable!(Text)

generate_cacheable_wrapper!(ProcessingInstruction, ProcessingInstruction::Wrap)
generate_traceable!(ProcessingInstruction)

generate_cacheable_wrapper_htmlelement!(HTMLHeadElement, HTMLHeadElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLHeadElement)

generate_cacheable_wrapper_htmlelement!(HTMLAnchorElement, HTMLAnchorElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLAnchorElement)

generate_cacheable_wrapper_htmlelement!(HTMLAppletElement, HTMLAppletElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLAppletElement)

generate_cacheable_wrapper_htmlelement!(HTMLAreaElement, HTMLAreaElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLAreaElement)

generate_cacheable_wrapper_htmlmediaelement!(HTMLAudioElement, HTMLAudioElementBinding::Wrap)
generate_traceable_htmlmediaelement!(HTMLAudioElement)

generate_cacheable_wrapper_htmlelement!(HTMLBaseElement, HTMLBaseElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLBaseElement)

generate_cacheable_wrapper_htmlelement!(HTMLBodyElement, HTMLBodyElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLBodyElement)

generate_cacheable_wrapper_htmlelement!(HTMLButtonElement, HTMLButtonElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLButtonElement)

generate_cacheable_wrapper_htmlelement!(HTMLCanvasElement, HTMLCanvasElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLCanvasElement)

generate_cacheable_wrapper_htmlelement!(HTMLDataListElement, HTMLDataListElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLDataListElement)

generate_cacheable_wrapper_htmlelement!(HTMLDListElement, HTMLDListElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLDListElement)

generate_cacheable_wrapper_htmlelement!(HTMLFormElement, HTMLFormElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLFormElement)

generate_cacheable_wrapper_htmlelement!(HTMLFrameElement, HTMLFrameElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLFrameElement)

generate_cacheable_wrapper_htmlelement!(HTMLFrameSetElement, HTMLFrameSetElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLFrameSetElement)

generate_cacheable_wrapper_htmlelement!(HTMLBRElement, HTMLBRElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLBRElement)

generate_cacheable_wrapper_htmlelement!(HTMLHRElement, HTMLHRElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLHRElement)

generate_cacheable_wrapper_htmlelement!(HTMLHtmlElement, HTMLHtmlElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLHtmlElement)

generate_cacheable_wrapper_htmlelement!(HTMLDataElement, HTMLDataElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLDataElement)

generate_cacheable_wrapper_htmlelement!(HTMLDirectoryElement, HTMLDirectoryElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLDirectoryElement)

generate_cacheable_wrapper_htmlelement!(HTMLDivElement, HTMLDivElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLDivElement)

generate_cacheable_wrapper_htmlelement!(HTMLEmbedElement, HTMLEmbedElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLEmbedElement)

generate_cacheable_wrapper_htmlelement!(HTMLFieldSetElement, HTMLFieldSetElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLFieldSetElement)

generate_cacheable_wrapper_htmlelement!(HTMLFontElement, HTMLFontElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLFontElement)

generate_cacheable_wrapper_htmlelement!(HTMLHeadingElement, HTMLHeadingElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLHeadingElement)

generate_cacheable_wrapper_htmlelement!(HTMLIFrameElement, HTMLIFrameElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLIFrameElement)

generate_cacheable_wrapper_htmlelement!(HTMLImageElement, HTMLImageElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLImageElement)

generate_cacheable_wrapper_htmlelement!(HTMLInputElement, HTMLInputElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLInputElement)

generate_cacheable_wrapper_htmlelement!(HTMLLabelElement, HTMLLabelElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLLabelElement)

generate_cacheable_wrapper_htmlelement!(HTMLLegendElement, HTMLLegendElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLLegendElement)

generate_cacheable_wrapper_htmlelement!(HTMLLIElement, HTMLLIElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLLIElement)

generate_cacheable_wrapper_htmlelement!(HTMLLinkElement, HTMLLinkElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLLinkElement)

generate_cacheable_wrapper_htmlelement!(HTMLMainElement, HTMLMainElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLMainElement)

generate_cacheable_wrapper_htmlelement!(HTMLMapElement, HTMLMapElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLMapElement)

generate_cacheable_wrapper_htmlelement!(HTMLMediaElement, HTMLMediaElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLMediaElement)

generate_cacheable_wrapper_htmlelement!(HTMLMetaElement, HTMLMetaElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLMetaElement)

generate_cacheable_wrapper_htmlelement!(HTMLMeterElement, HTMLMeterElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLMeterElement)

generate_cacheable_wrapper_htmlelement!(HTMLModElement, HTMLModElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLModElement)

generate_cacheable_wrapper_htmlelement!(HTMLObjectElement, HTMLObjectElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLObjectElement)

generate_cacheable_wrapper_htmlelement!(HTMLOListElement, HTMLOListElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLOListElement)

generate_cacheable_wrapper_htmlelement!(HTMLOptGroupElement, HTMLOptGroupElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLOptGroupElement)

generate_cacheable_wrapper_htmlelement!(HTMLOptionElement, HTMLOptionElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLOptionElement)

generate_cacheable_wrapper_htmlelement!(HTMLOutputElement, HTMLOutputElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLOutputElement)

generate_cacheable_wrapper_htmlelement!(HTMLParagraphElement, HTMLParagraphElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLParagraphElement)

generate_cacheable_wrapper_htmlelement!(HTMLParamElement, HTMLParamElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLParamElement)

generate_cacheable_wrapper_htmlelement!(HTMLPreElement, HTMLPreElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLPreElement)

generate_cacheable_wrapper_htmlelement!(HTMLProgressElement, HTMLProgressElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLProgressElement)

generate_cacheable_wrapper_htmlelement!(HTMLQuoteElement, HTMLQuoteElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLQuoteElement)

generate_cacheable_wrapper_htmlelement!(HTMLScriptElement, HTMLScriptElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLScriptElement)

generate_cacheable_wrapper_htmlelement!(HTMLSelectElement, HTMLSelectElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLSelectElement)

generate_cacheable_wrapper_htmlelement!(HTMLSourceElement, HTMLSourceElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLSourceElement)

generate_cacheable_wrapper_htmlelement!(HTMLSpanElement, HTMLSpanElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLSpanElement)

generate_cacheable_wrapper_htmlelement!(HTMLStyleElement, HTMLStyleElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLStyleElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableElement, HTMLTableElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableCaptionElement, HTMLTableCaptionElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableCaptionElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableCellElement, HTMLTableCellElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableCellElement)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableDataCellElement, HTMLTableDataCellElementBinding::Wrap)
generate_traceable_htmltablecellelement!(HTMLTableDataCellElement)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableHeaderCellElement, HTMLTableHeaderCellElementBinding::Wrap)
generate_traceable_htmltablecellelement!(HTMLTableHeaderCellElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableColElement, HTMLTableColElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableColElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableRowElement, HTMLTableRowElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableRowElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableSectionElement, HTMLTableSectionElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTableSectionElement)

generate_cacheable_wrapper_htmlelement!(HTMLTemplateElement, HTMLTemplateElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTemplateElement)

generate_cacheable_wrapper_htmlelement!(HTMLTextAreaElement, HTMLTextAreaElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTextAreaElement)

generate_cacheable_wrapper_htmlelement!(HTMLTitleElement, HTMLTitleElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTitleElement)

generate_cacheable_wrapper_htmlelement!(HTMLTimeElement, HTMLTimeElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTimeElement)

generate_cacheable_wrapper_htmlelement!(HTMLTrackElement, HTMLTrackElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLTrackElement)

generate_cacheable_wrapper_htmlelement!(HTMLUListElement, HTMLUListElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLUListElement)

generate_cacheable_wrapper_htmlelement!(HTMLUnknownElement, HTMLUnknownElementBinding::Wrap)
generate_traceable_htmlelement!(HTMLUnknownElement)

generate_cacheable_wrapper_htmlmediaelement!(HTMLVideoElement, HTMLVideoElementBinding::Wrap)
generate_traceable_htmlmediaelement!(HTMLVideoElement)

generate_cacheable_wrapper!(HTMLElement, HTMLElementBinding::Wrap)
generate_traceable!(HTMLElement)

generate_traceable_node!(Element)

generate_traceable_node!(CharacterData)
