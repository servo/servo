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
use string_cache::{Atom, QualName};
use util::str::DOMString;

pub fn create_element(name: QualName,
                      prefix: Option<Atom>,
                      document: &Document,
                      creator: ElementCreator)
                      -> Root<Element> {
    // FIXME(ajeffrey): Convert directly from Atom to DOMString.

    let prefix = prefix.map(|p| DOMString::from(&*p));

    if name.ns != ns!(html) {
        return Element::new(name.local, name.ns, prefix, document);
    }

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
        atom!("a")          => make!(HTMLAnchorElement),
        atom!("abbr")       => make!(HTMLElement),
        atom!("acronym")    => make!(HTMLElement),
        atom!("address")    => make!(HTMLElement),
        atom!("applet")     => make!(HTMLAppletElement),
        atom!("area")       => make!(HTMLAreaElement),
        atom!("article")    => make!(HTMLElement),
        atom!("aside")      => make!(HTMLElement),
        atom!("audio")      => make!(HTMLAudioElement),
        atom!("b")          => make!(HTMLElement),
        atom!("base")       => make!(HTMLBaseElement),
        atom!("bdi")        => make!(HTMLElement),
        atom!("bdo")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:bgsound
        atom!("bgsound")    => make!(HTMLUnknownElement),
        atom!("big")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:blink
        atom!("blink")      => make!(HTMLUnknownElement),
        // https://html.spec.whatwg.org/multipage/#the-blockquote-element
        atom!("blockquote") => make!(HTMLQuoteElement),
        atom!("body")       => make!(HTMLBodyElement),
        atom!("br")         => make!(HTMLBRElement),
        atom!("button")     => make!(HTMLButtonElement),
        atom!("canvas")     => make!(HTMLCanvasElement),
        atom!("caption")    => make!(HTMLTableCaptionElement),
        atom!("center")     => make!(HTMLElement),
        atom!("cite")       => make!(HTMLElement),
        atom!("code")       => make!(HTMLElement),
        atom!("col")        => make!(HTMLTableColElement),
        atom!("colgroup")   => make!(HTMLTableColElement),
        atom!("data")       => make!(HTMLDataElement),
        atom!("datalist")   => make!(HTMLDataListElement),
        atom!("dd")         => make!(HTMLElement),
        atom!("del")        => make!(HTMLModElement),
        atom!("details")    => make!(HTMLDetailsElement),
        atom!("dfn")        => make!(HTMLElement),
        atom!("dialog")     => make!(HTMLDialogElement),
        atom!("dir")        => make!(HTMLDirectoryElement),
        atom!("div")        => make!(HTMLDivElement),
        atom!("dl")         => make!(HTMLDListElement),
        atom!("dt")         => make!(HTMLElement),
        atom!("em")         => make!(HTMLElement),
        atom!("embed")      => make!(HTMLEmbedElement),
        atom!("fieldset")   => make!(HTMLFieldSetElement),
        atom!("figcaption") => make!(HTMLElement),
        atom!("figure")     => make!(HTMLElement),
        atom!("font")       => make!(HTMLFontElement),
        atom!("footer")     => make!(HTMLElement),
        atom!("form")       => make!(HTMLFormElement),
        atom!("frame")      => make!(HTMLFrameElement),
        atom!("frameset")   => make!(HTMLFrameSetElement),
        atom!("h1")         => make!(HTMLHeadingElement, HeadingLevel::Heading1),
        atom!("h2")         => make!(HTMLHeadingElement, HeadingLevel::Heading2),
        atom!("h3")         => make!(HTMLHeadingElement, HeadingLevel::Heading3),
        atom!("h4")         => make!(HTMLHeadingElement, HeadingLevel::Heading4),
        atom!("h5")         => make!(HTMLHeadingElement, HeadingLevel::Heading5),
        atom!("h6")         => make!(HTMLHeadingElement, HeadingLevel::Heading6),
        atom!("head")       => make!(HTMLHeadElement),
        atom!("header")     => make!(HTMLElement),
        atom!("hgroup")     => make!(HTMLElement),
        atom!("hr")         => make!(HTMLHRElement),
        atom!("html")       => make!(HTMLHtmlElement),
        atom!("i")          => make!(HTMLElement),
        atom!("iframe")     => make!(HTMLIFrameElement),
        atom!("img")        => make!(HTMLImageElement),
        atom!("input")      => make!(HTMLInputElement),
        atom!("ins")        => make!(HTMLModElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:isindex-2
        atom!("isindex")    => make!(HTMLUnknownElement),
        atom!("kbd")        => make!(HTMLElement),
        atom!("label")      => make!(HTMLLabelElement),
        atom!("legend")     => make!(HTMLLegendElement),
        atom!("li")         => make!(HTMLLIElement),
        atom!("link")       => make!(HTMLLinkElement, creator),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:listing
        atom!("listing")    => make!(HTMLPreElement),
        atom!("main")       => make!(HTMLElement),
        atom!("map")        => make!(HTMLMapElement),
        atom!("mark")       => make!(HTMLElement),
        atom!("marquee")    => make!(HTMLElement),
        atom!("meta")       => make!(HTMLMetaElement),
        atom!("meter")      => make!(HTMLMeterElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:multicol
        atom!("multicol")   => make!(HTMLUnknownElement),
        atom!("nav")        => make!(HTMLElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:nextid
        atom!("nextid")     => make!(HTMLUnknownElement),
        atom!("nobr")       => make!(HTMLElement),
        atom!("noframes")   => make!(HTMLElement),
        atom!("noscript")   => make!(HTMLElement),
        atom!("object")     => make!(HTMLObjectElement),
        atom!("ol")         => make!(HTMLOListElement),
        atom!("optgroup")   => make!(HTMLOptGroupElement),
        atom!("option")     => make!(HTMLOptionElement),
        atom!("output")     => make!(HTMLOutputElement),
        atom!("p")          => make!(HTMLParagraphElement),
        atom!("param")      => make!(HTMLParamElement),
        atom!("plaintext")  => make!(HTMLPreElement),
        atom!("pre")        => make!(HTMLPreElement),
        atom!("progress")   => make!(HTMLProgressElement),
        atom!("q")          => make!(HTMLQuoteElement),
        atom!("rp")         => make!(HTMLElement),
        atom!("rt")         => make!(HTMLElement),
        atom!("ruby")       => make!(HTMLElement),
        atom!("s")          => make!(HTMLElement),
        atom!("samp")       => make!(HTMLElement),
        atom!("script")     => make!(HTMLScriptElement, creator),
        atom!("section")    => make!(HTMLElement),
        atom!("select")     => make!(HTMLSelectElement),
        atom!("small")      => make!(HTMLElement),
        atom!("source")     => make!(HTMLSourceElement),
        // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:spacer
        atom!("spacer")     => make!(HTMLUnknownElement),
        atom!("span")       => make!(HTMLSpanElement),
        atom!("strike")     => make!(HTMLElement),
        atom!("strong")     => make!(HTMLElement),
        atom!("style")      => make!(HTMLStyleElement),
        atom!("sub")        => make!(HTMLElement),
        atom!("summary")    => make!(HTMLElement),
        atom!("sup")        => make!(HTMLElement),
        atom!("table")      => make!(HTMLTableElement),
        atom!("tbody")      => make!(HTMLTableSectionElement),
        atom!("td")         => make!(HTMLTableDataCellElement),
        atom!("template")   => make!(HTMLTemplateElement),
        atom!("textarea")   => make!(HTMLTextAreaElement),
        // https://html.spec.whatwg.org/multipage/#the-tfoot-element:concept-element-dom
        atom!("tfoot")      => make!(HTMLTableSectionElement),
        atom!("th")         => make!(HTMLTableHeaderCellElement),
        // https://html.spec.whatwg.org/multipage/#the-thead-element:concept-element-dom
        atom!("thead")      => make!(HTMLTableSectionElement),
        atom!("time")       => make!(HTMLTimeElement),
        atom!("title")      => make!(HTMLTitleElement),
        atom!("tr")         => make!(HTMLTableRowElement),
        atom!("tt")         => make!(HTMLElement),
        atom!("track")      => make!(HTMLTrackElement),
        atom!("u")          => make!(HTMLElement),
        atom!("ul")         => make!(HTMLUListElement),
        atom!("var")        => make!(HTMLElement),
        atom!("video")      => make!(HTMLVideoElement),
        atom!("wbr")        => make!(HTMLElement),
        atom!("xmp")        => make!(HTMLPreElement),
        _                   => make!(HTMLUnknownElement),
    }
}
