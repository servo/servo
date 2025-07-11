/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::rust::HandleObject;
use servo_config::pref;

use crate::dom::bindings::error::{report_pending_exception, throw_dom_exception};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::customelementregistry::{
    CustomElementState, is_valid_custom_element_name, upgrade_element,
};
use crate::dom::document::Document;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlaudioelement::HTMLAudioElement;
use crate::dom::htmlbaseelement::HTMLBaseElement;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlbrelement::HTMLBRElement;
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::htmldataelement::HTMLDataElement;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmldetailselement::HTMLDetailsElement;
use crate::dom::htmldialogelement::HTMLDialogElement;
use crate::dom::htmldirectoryelement::HTMLDirectoryElement;
use crate::dom::htmldivelement::HTMLDivElement;
use crate::dom::htmldlistelement::HTMLDListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlembedelement::HTMLEmbedElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlfontelement::HTMLFontElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmlframeelement::HTMLFrameElement;
use crate::dom::htmlframesetelement::HTMLFrameSetElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlheadingelement::{HTMLHeadingElement, HeadingLevel};
use crate::dom::htmlhrelement::HTMLHRElement;
use crate::dom::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::htmllielement::HTMLLIElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlmapelement::HTMLMapElement;
use crate::dom::htmlmenuelement::HTMLMenuElement;
use crate::dom::htmlmetaelement::HTMLMetaElement;
use crate::dom::htmlmeterelement::HTMLMeterElement;
use crate::dom::htmlmodelement::HTMLModElement;
use crate::dom::htmlobjectelement::HTMLObjectElement;
use crate::dom::htmlolistelement::HTMLOListElement;
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmloutputelement::HTMLOutputElement;
use crate::dom::htmlparagraphelement::HTMLParagraphElement;
use crate::dom::htmlparamelement::HTMLParamElement;
use crate::dom::htmlpictureelement::HTMLPictureElement;
use crate::dom::htmlpreelement::HTMLPreElement;
use crate::dom::htmlprogresselement::HTMLProgressElement;
use crate::dom::htmlquoteelement::HTMLQuoteElement;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::htmlslotelement::HTMLSlotElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::htmlspanelement::HTMLSpanElement;
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmltablecaptionelement::HTMLTableCaptionElement;
use crate::dom::htmltablecellelement::HTMLTableCellElement;
use crate::dom::htmltablecolelement::HTMLTableColElement;
use crate::dom::htmltableelement::HTMLTableElement;
use crate::dom::htmltablerowelement::HTMLTableRowElement;
use crate::dom::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::htmltimeelement::HTMLTimeElement;
use crate::dom::htmltitleelement::HTMLTitleElement;
use crate::dom::htmltrackelement::HTMLTrackElement;
use crate::dom::htmlulistelement::HTMLUListElement;
use crate::dom::htmlunknownelement::HTMLUnknownElement;
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::svgelement::SVGElement;
use crate::dom::svgimageelement::SVGImageElement;
use crate::dom::svgsvgelement::SVGSVGElement;
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

fn create_svg_element(
    name: QualName,
    prefix: Option<Prefix>,
    document: &Document,
    proto: Option<HandleObject>,
) -> DomRoot<Element> {
    assert_eq!(name.ns, ns!(svg));

    macro_rules! make(
        ($ctor:ident) => ({
            let obj = $ctor::new(name.local, prefix, document, proto, CanGc::note());
            DomRoot::upcast(obj)
        });
        ($ctor:ident, $($arg:expr),+) => ({
            let obj = $ctor::new(name.local, prefix, document, proto, $($arg),+, CanGc::note());
            DomRoot::upcast(obj)
        })
    );

    if !pref!(dom_svg_enabled) {
        return Element::new(name.local, name.ns, prefix, document, proto, CanGc::note());
    }

    match name.local {
        local_name!("image") => make!(SVGImageElement),
        local_name!("svg") => make!(SVGSVGElement),
        _ => make!(SVGElement),
    }
}

/// <https://dom.spec.whatwg.org/#concept-create-element>
#[allow(unsafe_code)]
#[allow(clippy::too_many_arguments)]
fn create_html_element(
    name: QualName,
    prefix: Option<Prefix>,
    is: Option<LocalName>,
    document: &Document,
    creator: ElementCreator,
    mode: CustomElementCreationMode,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> DomRoot<Element> {
    assert_eq!(name.ns, ns!(html));

    // Step 2. Let definition be the result of looking up a custom element
    // definition given document, namespace, localName, and is.
    let definition = document.lookup_custom_element_definition(&name.ns, &name.local, is.as_ref());

    // Step 3. If definition is non-null...
    if let Some(definition) = definition {
        // ...and definition’s name is not equal to its local name
        // (i.e., definition represents a customized built-in element):
        if !definition.is_autonomous() {
            // Step 3.1. Let interface be the element interface for localName and the HTML namespace.
            // Step 3.2. Set result to a new element that implements interface, with no attributes,
            // namespace set to the HTML namespace, namespace prefix set to prefix,
            // local name set to localName, custom element state set to "undefined",
            // custom element definition set to null, is value set to is,
            // and node document set to document.
            let element = create_native_html_element(name, prefix, document, creator, proto);
            element.set_is(definition.name.clone());
            element.set_custom_element_state(CustomElementState::Undefined);
            match mode {
                // Step 3.3. If synchronousCustomElements is true, then run this step while catching any exceptions:
                CustomElementCreationMode::Synchronous => {
                    // Step 3.3.1. Upgrade result using definition.
                    upgrade_element(definition, &element, can_gc);
                    // TODO: "If this step threw an exception exception:" steps.
                },
                // Step 3.4. Otherwise, enqueue a custom element upgrade reaction given result and definition.
                CustomElementCreationMode::Asynchronous => {
                    ScriptThread::enqueue_upgrade_reaction(&element, definition)
                },
            }
            return element;
        } else {
            // Step 4. Otherwise, if definition is non-null:
            match mode {
                // Step 4.1. If synchronousCustomElements is true, then run these
                // steps while catching any exceptions:
                CustomElementCreationMode::Synchronous => {
                    let local_name = name.local.clone();
                    //TODO(jdm) Pass proto to create_element?
                    // Steps 4.1.1-4.1.11
                    return match definition.create_element(document, prefix.clone(), can_gc) {
                        Ok(element) => {
                            element.set_custom_element_definition(definition.clone());
                            element
                        },
                        Err(error) => {
                            // If any of these steps threw an exception exception:
                            let global =
                                GlobalScope::current().unwrap_or_else(|| document.global());
                            let cx = GlobalScope::get_cx();

                            // Substep 1. Report exception for definition’s constructor’s corresponding
                            // JavaScript object’s associated realm’s global object.

                            let ar = enter_realm(&*global);
                            throw_dom_exception(cx, &global, error, can_gc);
                            report_pending_exception(cx, true, InRealm::Entered(&ar), can_gc);

                            // Substep 2. Set result to a new element that implements the HTMLUnknownElement interface,
                            // with no attributes, namespace set to the HTML namespace, namespace prefix set to prefix,
                            // local name set to localName, custom element state set to "failed",
                            // custom element definition set to null, is value set to null,
                            // and node document set to document.
                            let element = DomRoot::upcast::<Element>(HTMLUnknownElement::new(
                                local_name, prefix, document, proto, can_gc,
                            ));
                            element.set_custom_element_state(CustomElementState::Failed);
                            element
                        },
                    };
                },
                // Step 4.2. Otherwise:
                CustomElementCreationMode::Asynchronous => {
                    // Step 4.2.1. Set result to a new element that implements the HTMLElement interface,
                    // with no attributes, namespace set to the HTML namespace, namespace prefix set to
                    // prefix, local name set to localName, custom element state set to "undefined",
                    // custom element definition set to null, is value set to null, and node document
                    // set to document.
                    let result = DomRoot::upcast::<Element>(HTMLElement::new(
                        name.local.clone(),
                        prefix.clone(),
                        document,
                        proto,
                        can_gc,
                    ));
                    result.set_custom_element_state(CustomElementState::Undefined);
                    // Step 4.2.2. Enqueue a custom element upgrade reaction given result and definition.
                    ScriptThread::enqueue_upgrade_reaction(&result, definition);
                    return result;
                },
            }
        }
    }

    // Step 5. Otherwise:
    // Step 5.1. Let interface be the element interface for localName and namespace.
    // Step 5.2. Set result to a new element that implements interface, with no attributes,
    // namespace set to namespace, namespace prefix set to prefix, local name set to localName,
    // custom element state set to "uncustomized", custom element definition set to null,
    // is value set to is, and node document set to document.
    let result = create_native_html_element(name.clone(), prefix, document, creator, proto);
    // Step 5.3. If namespace is the HTML namespace, and either localName is a valid custom element name or
    // is is non-null, then set result’s custom element state to "undefined".
    match is {
        Some(is) => {
            result.set_is(is);
            result.set_custom_element_state(CustomElementState::Undefined);
        },
        None => {
            if is_valid_custom_element_name(&name.local) {
                result.set_custom_element_state(CustomElementState::Undefined);
            } else {
                result.set_custom_element_state(CustomElementState::Uncustomized);
            }
        },
    };

    // Step 6. Return result.
    result
}

pub(crate) fn create_native_html_element(
    name: QualName,
    prefix: Option<Prefix>,
    document: &Document,
    creator: ElementCreator,
    proto: Option<HandleObject>,
) -> DomRoot<Element> {
    assert_eq!(name.ns, ns!(html));

    macro_rules! make(
        ($ctor:ident) => ({
            let obj = $ctor::new(name.local, prefix, document, proto, CanGc::note());
            DomRoot::upcast(obj)
        });
        ($ctor:ident, $($arg:expr),+) => ({
            let obj = $ctor::new(name.local, prefix, document, proto, $($arg),+, CanGc::note());
            DomRoot::upcast(obj)
        })
    );

    // This is a big match, and the IDs for inline-interned atoms are not very structured.
    // Perhaps we should build a perfect hash from those IDs instead.
    // https://html.spec.whatwg.org/multipage/#elements-in-the-dom
    match name.local {
        local_name!("a") => make!(HTMLAnchorElement),
        local_name!("abbr") => make!(HTMLElement),
        local_name!("acronym") => make!(HTMLElement),
        local_name!("address") => make!(HTMLElement),
        local_name!("area") => make!(HTMLAreaElement),
        local_name!("article") => make!(HTMLElement),
        local_name!("aside") => make!(HTMLElement),
        local_name!("audio") => make!(HTMLAudioElement),
        local_name!("b") => make!(HTMLElement),
        local_name!("base") => make!(HTMLBaseElement),
        local_name!("bdi") => make!(HTMLElement),
        local_name!("bdo") => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:bgsound
        local_name!("bgsound") => make!(HTMLUnknownElement),
        local_name!("big") => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:blink
        local_name!("blink") => make!(HTMLUnknownElement),
        // https://html.spec.whatwg.org/multipage/#the-blockquote-element
        local_name!("blockquote") => make!(HTMLQuoteElement),
        local_name!("body") => make!(HTMLBodyElement),
        local_name!("br") => make!(HTMLBRElement),
        local_name!("button") => make!(HTMLButtonElement),
        local_name!("canvas") => make!(HTMLCanvasElement),
        local_name!("caption") => make!(HTMLTableCaptionElement),
        local_name!("center") => make!(HTMLElement),
        local_name!("cite") => make!(HTMLElement),
        local_name!("code") => make!(HTMLElement),
        local_name!("col") => make!(HTMLTableColElement),
        local_name!("colgroup") => make!(HTMLTableColElement),
        local_name!("data") => make!(HTMLDataElement),
        local_name!("datalist") => make!(HTMLDataListElement),
        local_name!("dd") => make!(HTMLElement),
        local_name!("del") => make!(HTMLModElement),
        local_name!("details") => make!(HTMLDetailsElement),
        local_name!("dfn") => make!(HTMLElement),
        local_name!("dialog") => make!(HTMLDialogElement),
        local_name!("dir") => make!(HTMLDirectoryElement),
        local_name!("div") => make!(HTMLDivElement),
        local_name!("dl") => make!(HTMLDListElement),
        local_name!("dt") => make!(HTMLElement),
        local_name!("em") => make!(HTMLElement),
        local_name!("embed") => make!(HTMLEmbedElement),
        local_name!("fieldset") => make!(HTMLFieldSetElement),
        local_name!("figcaption") => make!(HTMLElement),
        local_name!("figure") => make!(HTMLElement),
        local_name!("font") => make!(HTMLFontElement),
        local_name!("footer") => make!(HTMLElement),
        local_name!("form") => make!(HTMLFormElement),
        local_name!("frame") => make!(HTMLFrameElement),
        local_name!("frameset") => make!(HTMLFrameSetElement),
        local_name!("h1") => make!(HTMLHeadingElement, HeadingLevel::Heading1),
        local_name!("h2") => make!(HTMLHeadingElement, HeadingLevel::Heading2),
        local_name!("h3") => make!(HTMLHeadingElement, HeadingLevel::Heading3),
        local_name!("h4") => make!(HTMLHeadingElement, HeadingLevel::Heading4),
        local_name!("h5") => make!(HTMLHeadingElement, HeadingLevel::Heading5),
        local_name!("h6") => make!(HTMLHeadingElement, HeadingLevel::Heading6),
        local_name!("head") => make!(HTMLHeadElement),
        local_name!("header") => make!(HTMLElement),
        local_name!("hgroup") => make!(HTMLElement),
        local_name!("hr") => make!(HTMLHRElement),
        local_name!("html") => make!(HTMLHtmlElement),
        local_name!("i") => make!(HTMLElement),
        local_name!("iframe") => make!(HTMLIFrameElement),
        local_name!("img") => make!(HTMLImageElement, creator),
        local_name!("input") => make!(HTMLInputElement),
        local_name!("ins") => make!(HTMLModElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:isindex-2
        local_name!("isindex") => make!(HTMLUnknownElement),
        local_name!("kbd") => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#keygen
        local_name!("keygen") => make!(HTMLUnknownElement),
        local_name!("label") => make!(HTMLLabelElement),
        local_name!("legend") => make!(HTMLLegendElement),
        local_name!("li") => make!(HTMLLIElement),
        local_name!("link") => make!(HTMLLinkElement, creator),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:listing
        local_name!("listing") => make!(HTMLPreElement),
        local_name!("main") => make!(HTMLElement),
        local_name!("map") => make!(HTMLMapElement),
        local_name!("mark") => make!(HTMLElement),
        local_name!("marquee") => make!(HTMLElement),
        local_name!("menu") => make!(HTMLMenuElement),
        local_name!("meta") => make!(HTMLMetaElement),
        local_name!("meter") => make!(HTMLMeterElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:multicol
        local_name!("multicol") => make!(HTMLUnknownElement),
        local_name!("nav") => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:nextid
        local_name!("nextid") => make!(HTMLUnknownElement),
        local_name!("nobr") => make!(HTMLElement),
        local_name!("noframes") => make!(HTMLElement),
        local_name!("noscript") => make!(HTMLElement),
        local_name!("object") => make!(HTMLObjectElement),
        local_name!("ol") => make!(HTMLOListElement),
        local_name!("optgroup") => make!(HTMLOptGroupElement),
        local_name!("option") => make!(HTMLOptionElement),
        local_name!("output") => make!(HTMLOutputElement),
        local_name!("p") => make!(HTMLParagraphElement),
        local_name!("param") => make!(HTMLParamElement),
        local_name!("picture") => make!(HTMLPictureElement),
        local_name!("plaintext") => make!(HTMLPreElement),
        local_name!("pre") => make!(HTMLPreElement),
        local_name!("progress") => make!(HTMLProgressElement),
        local_name!("q") => make!(HTMLQuoteElement),
        local_name!("rp") => make!(HTMLElement),
        local_name!("rt") => make!(HTMLElement),
        local_name!("ruby") => make!(HTMLElement),
        local_name!("s") => make!(HTMLElement),
        local_name!("samp") => make!(HTMLElement),
        local_name!("script") => make!(HTMLScriptElement, creator),
        local_name!("section") => make!(HTMLElement),
        local_name!("select") => make!(HTMLSelectElement),
        local_name!("slot") => make!(HTMLSlotElement),
        local_name!("small") => make!(HTMLElement),
        local_name!("source") => make!(HTMLSourceElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:spacer
        local_name!("spacer") => make!(HTMLUnknownElement),
        local_name!("span") => make!(HTMLSpanElement),
        local_name!("strike") => make!(HTMLElement),
        local_name!("strong") => make!(HTMLElement),
        local_name!("style") => make!(HTMLStyleElement, creator),
        local_name!("sub") => make!(HTMLElement),
        local_name!("summary") => make!(HTMLElement),
        local_name!("sup") => make!(HTMLElement),
        local_name!("table") => make!(HTMLTableElement),
        local_name!("tbody") => make!(HTMLTableSectionElement),
        local_name!("td") => make!(HTMLTableCellElement),
        local_name!("template") => make!(HTMLTemplateElement),
        local_name!("textarea") => make!(HTMLTextAreaElement),
        // https://html.spec.whatwg.org/multipage/#the-tfoot-element:concept-element-dom
        local_name!("tfoot") => make!(HTMLTableSectionElement),
        local_name!("th") => make!(HTMLTableCellElement),
        // https://html.spec.whatwg.org/multipage/#the-thead-element:concept-element-dom
        local_name!("thead") => make!(HTMLTableSectionElement),
        local_name!("time") => make!(HTMLTimeElement),
        local_name!("title") => make!(HTMLTitleElement),
        local_name!("tr") => make!(HTMLTableRowElement),
        local_name!("tt") => make!(HTMLElement),
        local_name!("track") => make!(HTMLTrackElement),
        local_name!("u") => make!(HTMLElement),
        local_name!("ul") => make!(HTMLUListElement),
        local_name!("var") => make!(HTMLElement),
        local_name!("video") => make!(HTMLVideoElement),
        local_name!("wbr") => make!(HTMLElement),
        local_name!("xmp") => make!(HTMLPreElement),
        _ if is_valid_custom_element_name(&name.local) => make!(HTMLElement),
        _ => make!(HTMLUnknownElement),
    }
}

pub(crate) fn create_element(
    name: QualName,
    is: Option<LocalName>,
    document: &Document,
    creator: ElementCreator,
    mode: CustomElementCreationMode,
    proto: Option<HandleObject>,
    can_gc: CanGc,
) -> DomRoot<Element> {
    let prefix = name.prefix.clone();
    match name.ns {
        ns!(html) => create_html_element(name, prefix, is, document, creator, mode, proto, can_gc),
        ns!(svg) => create_svg_element(name, prefix, document, proto),
        _ => Element::new(name.local, name.ns, prefix, document, proto, can_gc),
    }
}
