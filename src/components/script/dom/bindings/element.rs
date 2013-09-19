/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::types::*;
use dom::bindings::codegen::*;
use dom::bindings::utils::{BindingObject, WrapperCache, CacheableWrapper, Traceable};
use dom::node::ScriptView;

use js::jsapi::{JSContext, JSObject, JSTracer};

macro_rules! generate_cacheable_wrapper(
    ($name: path, $wrap: path) => (
        impl CacheableWrapper for $name {
            fn get_wrappercache(&mut self) -> &mut WrapperCache {
                self.element.get_wrappercache()
            }

            fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
                let mut unused = false;
                $wrap(cx, scope, self, &mut unused)
            }
        }
    )
)

macro_rules! generate_cacheable_wrapper_htmlelement(
    ($name: path, $wrap: path) => (
        impl CacheableWrapper for $name {
            fn get_wrappercache(&mut self) -> &mut WrapperCache {
                self.htmlelement.get_wrappercache()
            }

            fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
                let mut unused = false;
                $wrap(cx, scope, self, &mut unused)
            }
        }
    )
)

macro_rules! generate_cacheable_wrapper_node(
    ($name: path, $wrap: path) => (
        impl CacheableWrapper for $name {
            fn get_wrappercache(&mut self) -> &mut WrapperCache {
                self.node.get_wrappercache()
            }

            fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
                let mut unused = false;
                $wrap(cx, scope, self, &mut unused)
            }
        }
    )
)

macro_rules! generate_binding_object(
    ($name: path) => (
        impl BindingObject for $name {
            fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
                self.element.GetParentObject(cx)
            }
        }
    )
)

macro_rules! generate_binding_object_htmlelement(
    ($name: path) => (
        impl BindingObject for $name {
            fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
                self.htmlelement.GetParentObject(cx)
            }
        }
    )
)

macro_rules! generate_binding_object_node(
    ($name: path) => (
        impl BindingObject for $name {
            fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
                self.node.GetParentObject(cx)
            }
        }
    )
)

macro_rules! generate_traceable(
    ($name: path) => (
        impl Traceable for $name {
            fn trace(&self, trc: *mut JSTracer) {
                self.element.trace(trc);
            }
        }
    )
)

macro_rules! generate_traceable_htmlelement(
    ($name: path) => (
        impl Traceable for $name {
            fn trace(&self, trc: *mut JSTracer) {
                self.htmlelement.trace(trc);
            }
        }
    )
)

macro_rules! generate_traceable_node(
    ($name: path) => (
        impl Traceable for $name {
            fn trace(&self, trc: *mut JSTracer) {
                self.node.trace(trc);
            }
        }
    )
)

generate_cacheable_wrapper!(Comment, CommentBinding::Wrap)
generate_binding_object!(Comment)
generate_traceable!(Comment)
generate_cacheable_wrapper_node!(DocumentType<ScriptView>, DocumentTypeBinding::Wrap)
generate_binding_object_node!(DocumentType<ScriptView>)
generate_traceable_node!(DocumentType<ScriptView>)
generate_cacheable_wrapper!(Text, TextBinding::Wrap)
generate_binding_object!(Text)
generate_traceable!(Text)
generate_cacheable_wrapper_htmlelement!(HTMLHeadElement, HTMLHeadElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLHeadElement)
generate_traceable_htmlelement!(HTMLHeadElement)
generate_cacheable_wrapper_htmlelement!(HTMLAnchorElement, HTMLAnchorElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLAnchorElement)
generate_traceable_htmlelement!(HTMLAnchorElement)
generate_cacheable_wrapper_htmlelement!(HTMLAppletElement, HTMLAppletElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLAppletElement)
generate_traceable_htmlelement!(HTMLAppletElement)
generate_cacheable_wrapper_htmlelement!(HTMLAreaElement, HTMLAreaElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLAreaElement)
generate_traceable_htmlelement!(HTMLAreaElement)
generate_cacheable_wrapper_htmlelement!(HTMLAudioElement, HTMLAudioElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLAudioElement)
generate_traceable_htmlelement!(HTMLAudioElement)
generate_cacheable_wrapper_htmlelement!(HTMLBaseElement, HTMLBaseElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLBaseElement)
generate_traceable_htmlelement!(HTMLBaseElement)
generate_cacheable_wrapper_htmlelement!(HTMLBodyElement, HTMLBodyElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLBodyElement)
generate_traceable_htmlelement!(HTMLBodyElement)
generate_cacheable_wrapper_htmlelement!(HTMLButtonElement, HTMLButtonElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLButtonElement)
generate_traceable_htmlelement!(HTMLButtonElement)
generate_cacheable_wrapper_htmlelement!(HTMLCanvasElement, HTMLCanvasElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLCanvasElement)
generate_traceable_htmlelement!(HTMLCanvasElement)
generate_cacheable_wrapper_htmlelement!(HTMLDataListElement, HTMLDataListElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLDataListElement)
generate_traceable_htmlelement!(HTMLDataListElement)
generate_cacheable_wrapper_htmlelement!(HTMLDListElement, HTMLDListElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLDListElement)
generate_traceable_htmlelement!(HTMLDListElement)
generate_cacheable_wrapper_htmlelement!(HTMLFormElement, HTMLFormElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLFormElement)
generate_traceable_htmlelement!(HTMLFormElement)
generate_cacheable_wrapper_htmlelement!(HTMLFrameElement, HTMLFrameElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLFrameElement)
generate_traceable_htmlelement!(HTMLFrameElement)
generate_cacheable_wrapper_htmlelement!(HTMLFrameSetElement, HTMLFrameSetElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLFrameSetElement)
generate_traceable_htmlelement!(HTMLFrameSetElement)
generate_cacheable_wrapper_htmlelement!(HTMLBRElement, HTMLBRElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLBRElement)
generate_traceable_htmlelement!(HTMLBRElement)
generate_cacheable_wrapper_htmlelement!(HTMLHRElement, HTMLHRElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLHRElement)
generate_traceable_htmlelement!(HTMLHRElement)
generate_cacheable_wrapper_htmlelement!(HTMLHtmlElement, HTMLHtmlElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLHtmlElement)
generate_traceable_htmlelement!(HTMLHtmlElement)
generate_cacheable_wrapper_htmlelement!(HTMLDataElement, HTMLDataElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLDataElement)
generate_traceable_htmlelement!(HTMLDataElement)
generate_cacheable_wrapper_htmlelement!(HTMLDirectoryElement, HTMLDirectoryElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLDirectoryElement)
generate_traceable_htmlelement!(HTMLDirectoryElement)
generate_cacheable_wrapper_htmlelement!(HTMLDivElement, HTMLDivElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLDivElement)
generate_traceable_htmlelement!(HTMLDivElement)
generate_cacheable_wrapper_htmlelement!(HTMLEmbedElement, HTMLEmbedElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLEmbedElement)
generate_traceable_htmlelement!(HTMLEmbedElement)
generate_cacheable_wrapper_htmlelement!(HTMLFieldSetElement, HTMLFieldSetElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLFieldSetElement)
generate_traceable_htmlelement!(HTMLFieldSetElement)
generate_cacheable_wrapper_htmlelement!(HTMLFontElement, HTMLFontElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLFontElement)
generate_traceable_htmlelement!(HTMLFontElement)
generate_cacheable_wrapper_htmlelement!(HTMLHeadingElement, HTMLHeadingElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLHeadingElement)
generate_traceable_htmlelement!(HTMLHeadingElement)
generate_cacheable_wrapper_htmlelement!(HTMLIFrameElement, HTMLIFrameElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLIFrameElement)
generate_traceable_htmlelement!(HTMLIFrameElement)
generate_cacheable_wrapper_htmlelement!(HTMLImageElement, HTMLImageElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLImageElement)
generate_traceable_htmlelement!(HTMLImageElement)
generate_cacheable_wrapper_htmlelement!(HTMLInputElement, HTMLInputElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLInputElement)
generate_traceable_htmlelement!(HTMLInputElement)
generate_cacheable_wrapper_htmlelement!(HTMLLabelElement, HTMLLabelElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLLabelElement)
generate_traceable_htmlelement!(HTMLLabelElement)
generate_cacheable_wrapper_htmlelement!(HTMLLegendElement, HTMLLegendElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLLegendElement)
generate_traceable_htmlelement!(HTMLLegendElement)
generate_cacheable_wrapper_htmlelement!(HTMLLIElement, HTMLLIElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLLIElement)
generate_traceable_htmlelement!(HTMLLIElement)
generate_cacheable_wrapper_htmlelement!(HTMLLinkElement, HTMLLinkElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLLinkElement)
generate_traceable_htmlelement!(HTMLLinkElement)
generate_cacheable_wrapper_htmlelement!(HTMLMapElement, HTMLMapElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLMapElement)
generate_traceable_htmlelement!(HTMLMapElement)
generate_cacheable_wrapper_htmlelement!(HTMLMediaElement, HTMLMediaElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLMediaElement)
generate_traceable_htmlelement!(HTMLMediaElement)
generate_cacheable_wrapper_htmlelement!(HTMLMetaElement, HTMLMetaElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLMetaElement)
generate_traceable_htmlelement!(HTMLMetaElement)
generate_cacheable_wrapper_htmlelement!(HTMLMeterElement, HTMLMeterElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLMeterElement)
generate_traceable_htmlelement!(HTMLMeterElement)
generate_cacheable_wrapper_htmlelement!(HTMLModElement, HTMLModElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLModElement)
generate_traceable_htmlelement!(HTMLModElement)
generate_cacheable_wrapper_htmlelement!(HTMLObjectElement, HTMLObjectElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLObjectElement)
generate_traceable_htmlelement!(HTMLObjectElement)
generate_cacheable_wrapper_htmlelement!(HTMLOListElement, HTMLOListElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLOListElement)
generate_traceable_htmlelement!(HTMLOListElement)
generate_cacheable_wrapper_htmlelement!(HTMLOptGroupElement, HTMLOptGroupElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLOptGroupElement)
generate_traceable_htmlelement!(HTMLOptGroupElement)
generate_cacheable_wrapper_htmlelement!(HTMLOptionElement, HTMLOptionElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLOptionElement)
generate_traceable_htmlelement!(HTMLOptionElement)
generate_cacheable_wrapper_htmlelement!(HTMLOutputElement, HTMLOutputElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLOutputElement)
generate_traceable_htmlelement!(HTMLOutputElement)
generate_cacheable_wrapper_htmlelement!(HTMLParagraphElement, HTMLParagraphElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLParagraphElement)
generate_traceable_htmlelement!(HTMLParagraphElement)
generate_cacheable_wrapper_htmlelement!(HTMLParamElement, HTMLParamElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLParamElement)
generate_traceable_htmlelement!(HTMLParamElement)
generate_cacheable_wrapper_htmlelement!(HTMLPreElement, HTMLPreElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLPreElement)
generate_traceable_htmlelement!(HTMLPreElement)
generate_cacheable_wrapper_htmlelement!(HTMLProgressElement, HTMLProgressElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLProgressElement)
generate_traceable_htmlelement!(HTMLProgressElement)
generate_cacheable_wrapper_htmlelement!(HTMLQuoteElement, HTMLQuoteElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLQuoteElement)
generate_traceable_htmlelement!(HTMLQuoteElement)
generate_cacheable_wrapper_htmlelement!(HTMLScriptElement, HTMLScriptElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLScriptElement)
generate_traceable_htmlelement!(HTMLScriptElement)
generate_cacheable_wrapper_htmlelement!(HTMLSelectElement, HTMLSelectElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLSelectElement)
generate_traceable_htmlelement!(HTMLSelectElement)
generate_cacheable_wrapper_htmlelement!(HTMLSourceElement, HTMLSourceElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLSourceElement)
generate_traceable_htmlelement!(HTMLSourceElement)
generate_cacheable_wrapper_htmlelement!(HTMLSpanElement, HTMLSpanElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLSpanElement)
generate_traceable_htmlelement!(HTMLSpanElement)
generate_cacheable_wrapper_htmlelement!(HTMLStyleElement, HTMLStyleElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLStyleElement)
generate_traceable_htmlelement!(HTMLStyleElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableElement, HTMLTableElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableElement)
generate_traceable_htmlelement!(HTMLTableElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableCaptionElement, HTMLTableCaptionElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableCaptionElement)
generate_traceable_htmlelement!(HTMLTableCaptionElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableCellElement, HTMLTableCellElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableCellElement)
generate_traceable_htmlelement!(HTMLTableCellElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableColElement, HTMLTableColElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableColElement)
generate_traceable_htmlelement!(HTMLTableColElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableRowElement, HTMLTableRowElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableRowElement)
generate_traceable_htmlelement!(HTMLTableRowElement)
generate_cacheable_wrapper_htmlelement!(HTMLTableSectionElement, HTMLTableSectionElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTableSectionElement)
generate_traceable_htmlelement!(HTMLTableSectionElement)
generate_cacheable_wrapper_htmlelement!(HTMLTemplateElement, HTMLTemplateElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTemplateElement)
generate_traceable_htmlelement!(HTMLTemplateElement)
generate_cacheable_wrapper_htmlelement!(HTMLTextAreaElement, HTMLTextAreaElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTextAreaElement)
generate_traceable_htmlelement!(HTMLTextAreaElement)
generate_cacheable_wrapper_htmlelement!(HTMLTitleElement, HTMLTitleElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTitleElement)
generate_traceable_htmlelement!(HTMLTitleElement)
generate_cacheable_wrapper_htmlelement!(HTMLTimeElement, HTMLTimeElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTimeElement)
generate_traceable_htmlelement!(HTMLTimeElement)
generate_cacheable_wrapper_htmlelement!(HTMLTrackElement, HTMLTrackElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLTrackElement)
generate_traceable_htmlelement!(HTMLTrackElement)
generate_cacheable_wrapper_htmlelement!(HTMLUListElement, HTMLUListElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLUListElement)
generate_traceable_htmlelement!(HTMLUListElement)
generate_cacheable_wrapper_htmlelement!(HTMLUnknownElement, HTMLUnknownElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLUnknownElement)
generate_traceable_htmlelement!(HTMLUnknownElement)
generate_cacheable_wrapper_htmlelement!(HTMLVideoElement, HTMLVideoElementBinding::Wrap)
generate_binding_object_htmlelement!(HTMLVideoElement)
generate_traceable_htmlelement!(HTMLVideoElement)

generate_traceable!(HTMLElement)
generate_traceable_node!(Element)
generate_traceable_node!(CharacterData)
