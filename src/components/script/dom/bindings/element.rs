/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::types::*;
use dom::bindings::utils::{Reflectable, Reflector};

// generate_cacheable_wrapper
macro_rules! generate_cacheable_wrapper(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, element)
    )
)

macro_rules! generate_cacheable_wrapper_characterdata(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, characterdata)
    )
)

macro_rules! generate_cacheable_wrapper_htmlelement(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, htmlelement)
    )
)

macro_rules! generate_cacheable_wrapper_htmlmediaelement(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, htmlmediaelement)
    )
)

macro_rules! generate_cacheable_wrapper_htmltablecellelement(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, htmltablecellelement)
    )
)

macro_rules! generate_cacheable_wrapper_node(
    ($name: path) => (
        generate_cacheable_wrapper_base!($name, node)
    )
)

macro_rules! generate_cacheable_wrapper_base(
    ($name: path, $parent: ident) => (
        impl Reflectable for $name {
            fn reflector<'a>(&'a self) -> &'a Reflector {
                self.$parent.reflector()
            }
        }
    )
)

generate_cacheable_wrapper_characterdata!(Comment)

generate_cacheable_wrapper_node!(DocumentFragment)

generate_cacheable_wrapper_node!(DocumentType)

generate_cacheable_wrapper_characterdata!(Text)

generate_cacheable_wrapper_characterdata!(ProcessingInstruction)

generate_cacheable_wrapper_htmlelement!(HTMLHeadElement)

generate_cacheable_wrapper_htmlelement!(HTMLAnchorElement)

generate_cacheable_wrapper_htmlelement!(HTMLAppletElement)

generate_cacheable_wrapper_htmlelement!(HTMLAreaElement)

generate_cacheable_wrapper_htmlmediaelement!(HTMLAudioElement)

generate_cacheable_wrapper_htmlelement!(HTMLBaseElement)

generate_cacheable_wrapper_htmlelement!(HTMLBodyElement)

generate_cacheable_wrapper_htmlelement!(HTMLButtonElement)

generate_cacheable_wrapper_htmlelement!(HTMLCanvasElement)

generate_cacheable_wrapper_htmlelement!(HTMLDataListElement)

generate_cacheable_wrapper_htmlelement!(HTMLDListElement)

generate_cacheable_wrapper_htmlelement!(HTMLFormElement)

generate_cacheable_wrapper_htmlelement!(HTMLFrameElement)

generate_cacheable_wrapper_htmlelement!(HTMLFrameSetElement)

generate_cacheable_wrapper_htmlelement!(HTMLBRElement)

generate_cacheable_wrapper_htmlelement!(HTMLHRElement)

generate_cacheable_wrapper_htmlelement!(HTMLHtmlElement)

generate_cacheable_wrapper_htmlelement!(HTMLDataElement)

generate_cacheable_wrapper_htmlelement!(HTMLDirectoryElement)
generate_cacheable_wrapper_htmlelement!(HTMLDivElement)

generate_cacheable_wrapper_htmlelement!(HTMLEmbedElement)

generate_cacheable_wrapper_htmlelement!(HTMLFieldSetElement)

generate_cacheable_wrapper_htmlelement!(HTMLFontElement)

generate_cacheable_wrapper_htmlelement!(HTMLHeadingElement)

generate_cacheable_wrapper_htmlelement!(HTMLIFrameElement)

generate_cacheable_wrapper_htmlelement!(HTMLImageElement)

generate_cacheable_wrapper_htmlelement!(HTMLInputElement)

generate_cacheable_wrapper_htmlelement!(HTMLLabelElement)

generate_cacheable_wrapper_htmlelement!(HTMLLegendElement)

generate_cacheable_wrapper_htmlelement!(HTMLLIElement)

generate_cacheable_wrapper_htmlelement!(HTMLLinkElement)

generate_cacheable_wrapper_htmlelement!(HTMLMapElement)

generate_cacheable_wrapper_htmlelement!(HTMLMediaElement)

generate_cacheable_wrapper_htmlelement!(HTMLMetaElement)

generate_cacheable_wrapper_htmlelement!(HTMLMeterElement)

generate_cacheable_wrapper_htmlelement!(HTMLModElement)

generate_cacheable_wrapper_htmlelement!(HTMLObjectElement)

generate_cacheable_wrapper_htmlelement!(HTMLOListElement)

generate_cacheable_wrapper_htmlelement!(HTMLOptGroupElement)

generate_cacheable_wrapper_htmlelement!(HTMLOptionElement)

generate_cacheable_wrapper_htmlelement!(HTMLOutputElement)

generate_cacheable_wrapper_htmlelement!(HTMLParagraphElement)

generate_cacheable_wrapper_htmlelement!(HTMLParamElement)

generate_cacheable_wrapper_htmlelement!(HTMLPreElement)

generate_cacheable_wrapper_htmlelement!(HTMLProgressElement)

generate_cacheable_wrapper_htmlelement!(HTMLQuoteElement)

generate_cacheable_wrapper_htmlelement!(HTMLScriptElement)

generate_cacheable_wrapper_htmlelement!(HTMLSelectElement)

generate_cacheable_wrapper_htmlelement!(HTMLSourceElement)

generate_cacheable_wrapper_htmlelement!(HTMLSpanElement)

generate_cacheable_wrapper_htmlelement!(HTMLStyleElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableCaptionElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableCellElement)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableDataCellElement)

generate_cacheable_wrapper_htmltablecellelement!(HTMLTableHeaderCellElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableColElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableRowElement)

generate_cacheable_wrapper_htmlelement!(HTMLTableSectionElement)

generate_cacheable_wrapper_htmlelement!(HTMLTemplateElement)

generate_cacheable_wrapper_htmlelement!(HTMLTextAreaElement)

generate_cacheable_wrapper_htmlelement!(HTMLTitleElement)

generate_cacheable_wrapper_htmlelement!(HTMLTimeElement)

generate_cacheable_wrapper_htmlelement!(HTMLTrackElement)

generate_cacheable_wrapper_htmlelement!(HTMLUListElement)

generate_cacheable_wrapper_htmlelement!(HTMLUnknownElement)

generate_cacheable_wrapper_htmlmediaelement!(HTMLVideoElement)

generate_cacheable_wrapper!(HTMLElement)
