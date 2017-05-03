/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::element::ElementCreator;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlaudioelement::HTMLAudioElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlbrelement::HTMLBRElement;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmldataelement::HTMLDataElement;
use dom::htmldatalistelement::HTMLDataListElement;
use dom::htmldetailselement::HTMLDetailsElement;
use dom::htmldialogelement::HTMLDialogElement;
use dom::htmldirectoryelement::HTMLDirectoryElement;
use dom::htmldivelement::HTMLDivElement;
use dom::htmldlistelement::HTMLDListElement;
use dom::htmlelement::HTMLElement;
use dom::htmlembedelement::HTMLEmbedElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::HTMLFontElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlframeelement::HTMLFrameElement;
use dom::htmlframesetelement::HTMLFrameSetElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlheadingelement::HTMLHeadingElement;
use dom::htmlheadingelement::HeadingLevel;
use dom::htmlhrelement::HTMLHRElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllegendelement::HTMLLegendElement;
use dom::htmllielement::HTMLLIElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmapelement::HTMLMapElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlmeterelement::HTMLMeterElement;
use dom::htmlmodelement::HTMLModElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmlolistelement::HTMLOListElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmloutputelement::HTMLOutputElement;
use dom::htmlparagraphelement::HTMLParagraphElement;
use dom::htmlparamelement::HTMLParamElement;
use dom::htmlpreelement::HTMLPreElement;
use dom::htmlprogresselement::HTMLProgressElement;
use dom::htmlquoteelement::HTMLQuoteElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmlsourceelement::HTMLSourceElement;
use dom::htmlspanelement::HTMLSpanElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::htmltablecolelement::HTMLTableColElement;
use dom::htmltabledatacellelement::HTMLTableDataCellElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltableheadercellelement::HTMLTableHeaderCellElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::htmltimeelement::HTMLTimeElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::htmltrackelement::HTMLTrackElement;
use dom::htmlulistelement::HTMLUListElement;
use dom::htmlunknownelement::HTMLUnknownElement;
use dom::htmlvideoelement::HTMLVideoElement;
use dom::svgsvgelement::SVGSVGElement;
use html5ever::{QualName, Prefix};
use servo_config::prefs::PREFS;

fn create_svg_element(name: QualName,
                      prefix: Option<Prefix>,
                      document: &Document)
                      -> Root<Element> {
    assert!(name.ns == ns!(svg));

    macro_rules! make(
        ($ctor:ident) => ({
            let obj = $ctor::new(name.local, prefix, document);
            Root::upcast(obj)
        });
        ($ctor:ident, $($arg:expr),+) => ({
            let obj = $ctor::new(name.local, prefix, document, $($arg),+);
            Root::upcast(obj)
        })
    );

    if !PREFS.get("dom.svg.enabled").as_boolean().unwrap_or(false) {
        return Element::new(name.local, name.ns, prefix, document);
    }

    match name.local {
        local_name!("svg")        => make!(SVGSVGElement),
        _                   => Element::new(name.local, name.ns, prefix, document),
    }
}

fn create_html_element(name: QualName,
                      prefix: Option<Prefix>,
                      document: &Document,
                      creator: ElementCreator)
                      -> Root<Element> {
    assert!(name.ns == ns!(html));

    macro_rules! make(
        ($ctor:ident) => ({
            let obj = $ctor::new(name.local, prefix, document);
            Root::upcast(obj)
        });
        ($ctor:ident, $($arg:expr),+) => ({
            let obj = $ctor::new(name.local, prefix, document, $($arg),+);
            Root::upcast(obj)
        })
    );

    // This is a big match, and the IDs for inline-interned atoms are not very structured.
    // Perhaps we should build a perfect hash from those IDs instead.
    match name.local {
        local_name!("a")          => make!(HTMLAnchorElement),
        local_name!("abbr")       => make!(HTMLElement),
        local_name!("acronym")    => make!(HTMLElement),
        local_name!("address")    => make!(HTMLElement),
        local_name!("applet")     => make!(HTMLAppletElement),
        local_name!("area")       => make!(HTMLAreaElement),
        local_name!("article")    => make!(HTMLElement),
        local_name!("aside")      => make!(HTMLElement),
        local_name!("audio")      => make!(HTMLAudioElement),
        local_name!("b")          => make!(HTMLElement),
        local_name!("base")       => make!(HTMLBaseElement),
        local_name!("bdi")        => make!(HTMLElement),
        local_name!("bdo")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:bgsound
        local_name!("bgsound")    => make!(HTMLUnknownElement),
        local_name!("big")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:blink
        local_name!("blink")      => make!(HTMLUnknownElement),
        // https://html.spec.whatwg.org/multipage/#the-blockquote-element
        local_name!("blockquote") => make!(HTMLQuoteElement),
        local_name!("body")       => make!(HTMLBodyElement),
        local_name!("br")         => make!(HTMLBRElement),
        local_name!("button")     => make!(HTMLButtonElement),
        local_name!("canvas")     => make!(HTMLCanvasElement),
        local_name!("caption")    => make!(HTMLTableCaptionElement),
        local_name!("center")     => make!(HTMLElement),
        local_name!("cite")       => make!(HTMLElement),
        local_name!("code")       => make!(HTMLElement),
        local_name!("col")        => make!(HTMLTableColElement),
        local_name!("colgroup")   => make!(HTMLTableColElement),
        local_name!("data")       => make!(HTMLDataElement),
        local_name!("datalist")   => make!(HTMLDataListElement),
        local_name!("dd")         => make!(HTMLElement),
        local_name!("del")        => make!(HTMLModElement),
        local_name!("details")    => make!(HTMLDetailsElement),
        local_name!("dfn")        => make!(HTMLElement),
        local_name!("dialog")     => make!(HTMLDialogElement),
        local_name!("dir")        => make!(HTMLDirectoryElement),
        local_name!("div")        => make!(HTMLDivElement),
        local_name!("dl")         => make!(HTMLDListElement),
        local_name!("dt")         => make!(HTMLElement),
        local_name!("em")         => make!(HTMLElement),
        local_name!("embed")      => make!(HTMLEmbedElement),
        local_name!("fieldset")   => make!(HTMLFieldSetElement),
        local_name!("figcaption") => make!(HTMLElement),
        local_name!("figure")     => make!(HTMLElement),
        local_name!("font")       => make!(HTMLFontElement),
        local_name!("footer")     => make!(HTMLElement),
        local_name!("form")       => make!(HTMLFormElement),
        local_name!("frame")      => make!(HTMLFrameElement),
        local_name!("frameset")   => make!(HTMLFrameSetElement),
        local_name!("h1")         => make!(HTMLHeadingElement, HeadingLevel::Heading1),
        local_name!("h2")         => make!(HTMLHeadingElement, HeadingLevel::Heading2),
        local_name!("h3")         => make!(HTMLHeadingElement, HeadingLevel::Heading3),
        local_name!("h4")         => make!(HTMLHeadingElement, HeadingLevel::Heading4),
        local_name!("h5")         => make!(HTMLHeadingElement, HeadingLevel::Heading5),
        local_name!("h6")         => make!(HTMLHeadingElement, HeadingLevel::Heading6),
        local_name!("head")       => make!(HTMLHeadElement),
        local_name!("header")     => make!(HTMLElement),
        local_name!("hgroup")     => make!(HTMLElement),
        local_name!("hr")         => make!(HTMLHRElement),
        local_name!("html")       => make!(HTMLHtmlElement),
        local_name!("i")          => make!(HTMLElement),
        local_name!("iframe")     => make!(HTMLIFrameElement),
        local_name!("img")        => make!(HTMLImageElement),
        local_name!("input")      => make!(HTMLInputElement),
        local_name!("ins")        => make!(HTMLModElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:isindex-2
        local_name!("isindex")    => make!(HTMLUnknownElement),
        local_name!("kbd")        => make!(HTMLElement),
        local_name!("label")      => make!(HTMLLabelElement),
        local_name!("legend")     => make!(HTMLLegendElement),
        local_name!("li")         => make!(HTMLLIElement),
        local_name!("link")       => make!(HTMLLinkElement, creator),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:listing
        local_name!("listing")    => make!(HTMLPreElement),
        local_name!("main")       => make!(HTMLElement),
        local_name!("map")        => make!(HTMLMapElement),
        local_name!("mark")       => make!(HTMLElement),
        local_name!("marquee")    => make!(HTMLElement),
        local_name!("meta")       => make!(HTMLMetaElement),
        local_name!("meter")      => make!(HTMLMeterElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:multicol
        local_name!("multicol")   => make!(HTMLUnknownElement),
        local_name!("nav")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:nextid
        local_name!("nextid")     => make!(HTMLUnknownElement),
        local_name!("nobr")       => make!(HTMLElement),
        local_name!("noframes")   => make!(HTMLElement),
        local_name!("noscript")   => make!(HTMLElement),
        local_name!("object")     => make!(HTMLObjectElement),
        local_name!("ol")         => make!(HTMLOListElement),
        local_name!("optgroup")   => make!(HTMLOptGroupElement),
        local_name!("option")     => make!(HTMLOptionElement),
        local_name!("output")     => make!(HTMLOutputElement),
        local_name!("p")          => make!(HTMLParagraphElement),
        local_name!("param")      => make!(HTMLParamElement),
        local_name!("plaintext")  => make!(HTMLPreElement),
        local_name!("pre")        => make!(HTMLPreElement),
        local_name!("progress")   => make!(HTMLProgressElement),
        local_name!("q")          => make!(HTMLQuoteElement),
        local_name!("rp")         => make!(HTMLElement),
        local_name!("rt")         => make!(HTMLElement),
        local_name!("ruby")       => make!(HTMLElement),
        local_name!("s")          => make!(HTMLElement),
        local_name!("samp")       => make!(HTMLElement),
        local_name!("script")     => make!(HTMLScriptElement, creator),
        local_name!("section")    => make!(HTMLElement),
        local_name!("select")     => make!(HTMLSelectElement),
        local_name!("small")      => make!(HTMLElement),
        local_name!("source")     => make!(HTMLSourceElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:spacer
        local_name!("spacer")     => make!(HTMLUnknownElement),
        local_name!("span")       => make!(HTMLSpanElement),
        local_name!("strike")     => make!(HTMLElement),
        local_name!("strong")     => make!(HTMLElement),
        local_name!("style")      => make!(HTMLStyleElement, creator),
        local_name!("sub")        => make!(HTMLElement),
        local_name!("summary")    => make!(HTMLElement),
        local_name!("sup")        => make!(HTMLElement),
        local_name!("table")      => make!(HTMLTableElement),
        local_name!("tbody")      => make!(HTMLTableSectionElement),
        local_name!("td")         => make!(HTMLTableDataCellElement),
        local_name!("template")   => make!(HTMLTemplateElement),
        local_name!("textarea")   => make!(HTMLTextAreaElement),
        // https://html.spec.whatwg.org/multipage/#the-tfoot-element:concept-element-dom
        local_name!("tfoot")      => make!(HTMLTableSectionElement),
        local_name!("th")         => make!(HTMLTableHeaderCellElement),
        // https://html.spec.whatwg.org/multipage/#the-thead-element:concept-element-dom
        local_name!("thead")      => make!(HTMLTableSectionElement),
        local_name!("time")       => make!(HTMLTimeElement),
        local_name!("title")      => make!(HTMLTitleElement),
        local_name!("tr")         => make!(HTMLTableRowElement),
        local_name!("tt")         => make!(HTMLElement),
        local_name!("track")      => make!(HTMLTrackElement),
        local_name!("u")          => make!(HTMLElement),
        local_name!("ul")         => make!(HTMLUListElement),
        local_name!("var")        => make!(HTMLElement),
        local_name!("video")      => make!(HTMLVideoElement),
        local_name!("wbr")        => make!(HTMLElement),
        local_name!("xmp")        => make!(HTMLPreElement),
        _                   => make!(HTMLUnknownElement),
    }
}

pub fn create_element(name: QualName,
                      document: &Document,
                      creator: ElementCreator)
                      -> Root<Element> {
    let prefix = name.prefix.clone();
    match name.ns {
        ns!(html)   => create_html_element(name, prefix, document, creator),
        ns!(svg)    => create_svg_element(name, prefix, document),
        _           => Element::new(name.local, name.ns, prefix, document)
    }
}
