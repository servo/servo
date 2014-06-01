/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::types::*;
use dom::bindings::utils::{Reflectable, Reflector};

// generate_cacheable_wrapper
macro_rules! generate_cacheable_wrapper(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, element)
    )
)

macro_rules! generate_cacheable_wrapper_characterdata(
    ($name: path, $wrap: path) => (
        generate_cacheable_wrapper_base!($name, $wrap, characterdata)
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

generate_cacheable_wrapper_characterdata!(Comment, CommentBinding::Wrap)

generate_cacheable_wrapper_node!(DocumentFragment, DocumentFragmentBinding::Wrap)

generate_cacheable_wrapper_node!(DocumentType, DocumentTypeBinding::Wrap)

generate_cacheable_wrapper_characterdata!(Text, TextBinding::Wrap)

generate_cacheable_wrapper_characterdata!(ProcessingInstruction, ProcessingInstruction::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLHeadElement, HTMLHeadElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLAnchorElement, HTMLAnchorElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLAppletElement, HTMLAppletElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLAreaElement, HTMLAreaElementBinding::Wrap)

generate_cacheable_wrapper_htmlmediaelement!(HTMLAudioElement, HTMLAudioElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLBaseElement, HTMLBaseElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLBodyElement, HTMLBodyElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLButtonElement, HTMLButtonElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLCanvasElement, HTMLCanvasElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLDataListElement, HTMLDataListElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLDListElement, HTMLDListElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLFormElement, HTMLFormElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLFrameElement, HTMLFrameElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLFrameSetElement, HTMLFrameSetElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLBRElement, HTMLBRElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLHRElement, HTMLHRElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLHtmlElement, HTMLHtmlElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLDataElement, HTMLDataElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLDirectoryElement, HTMLDirectoryElementBinding::Wrap)
generate_cacheable_wrapper_htmlelement!(HTMLDivElement, HTMLDivElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLEmbedElement, HTMLEmbedElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLFieldSetElement, HTMLFieldSetElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLFontElement, HTMLFontElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLHeadingElement, HTMLHeadingElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLIFrameElement, HTMLIFrameElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLImageElement, HTMLImageElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLInputElement, HTMLInputElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLLabelElement, HTMLLabelElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLLegendElement, HTMLLegendElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLLIElement, HTMLLIElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLLinkElement, HTMLLinkElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLMapElement, HTMLMapElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLMediaElement, HTMLMediaElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLMetaElement, HTMLMetaElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLMeterElement, HTMLMeterElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLModElement, HTMLModElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLObjectElement, HTMLObjectElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLOListElement, HTMLOListElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLOptGroupElement, HTMLOptGroupElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLOptionElement, HTMLOptionElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLOutputElement, HTMLOutputElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLParagraphElement, HTMLParagraphElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLParamElement, HTMLParamElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLPreElement, HTMLPreElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLProgressElement, HTMLProgressElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLQuoteElement, HTMLQuoteElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLScriptElement, HTMLScriptElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLSelectElement, HTMLSelectElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLSourceElement, HTMLSourceElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLSpanElement, HTMLSpanElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLStyleElement, HTMLStyleElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableElement, HTMLTableElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableCaptionElement, HTMLTableCaptionElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableCellElement, HTMLTableCellElementBinding::Wrap)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableDataCellElement, HTMLTableDataCellElementBinding::Wrap)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableHeaderCellElement, HTMLTableHeaderCellElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableColElement, HTMLTableColElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableRowElement, HTMLTableRowElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTableSectionElement, HTMLTableSectionElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTemplateElement, HTMLTemplateElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTextAreaElement, HTMLTextAreaElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTitleElement, HTMLTitleElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTimeElement, HTMLTimeElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLTrackElement, HTMLTrackElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLUListElement, HTMLUListElementBinding::Wrap)

generate_cacheable_wrapper_htmlelement!(HTMLUnknownElement, HTMLUnknownElementBinding::Wrap)

generate_cacheable_wrapper_htmlmediaelement!(HTMLVideoElement, HTMLVideoElementBinding::Wrap)

generate_cacheable_wrapper!(HTMLElement, HTMLElementBinding::Wrap)
