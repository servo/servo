use au = gfx::geometry;
use au::au;
use dvec::DVec;
use geom::size::Size2D;

struct ElementData {
    tag_name: ~str,
    kind: ~ElementKind,
    attrs: DVec<~Attr>,
}

impl ElementData {
    fn get_attr(attr_name: ~str) -> Option<~str> {
        let mut i = 0u;
        while i < self.attrs.len() {
            if attr_name == self.attrs[i].name {
                return Some(copy self.attrs[i].value);
            }
            i += 1u;
        }

        None
    }

    fn set_attr(_attr_name: ~str, attr_value: ~str) {
        // TODO: add new attr of name, or delete old one
    }
}

fn ElementData(tag_name: ~str, kind: ~ElementKind) -> ElementData {
    ElementData {
        tag_name : tag_name,
        kind : kind,
        attrs : DVec(),
    }
}

struct Attr {
    name: ~str,
    value: ~str,
}

fn Attr(name: ~str, value: ~str) -> Attr {
    Attr {
        name : name,
        value : value,
    }
}

enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

enum ElementKind {
    HTMLAnchorElement,
    HTMLAsideElement,
    HTMLBRElement,
    HTMLBodyElement,
    HTMLBoldElement,
    HTMLDivElement,
    HTMLFontElement,
    HTMLFormElement,
    HTMLHRElement,
    HTMLHeadElement,
    HTMLHeadingElement(HeadingLevel),
    HTMLHtmlElement,
    // TODO: should not take this as argument--it is fetched from
    // layout task as requested.
    HTMLImageElement({mut size: Size2D<au>}),
    HTMLInputElement,
    HTMLItalicElement,
    HTMLLinkElement,
    HTMLListItemElement,
    HTMLMetaElement,
    HTMLOListElement,
    HTMLOptionElement,
    HTMLParagraphElement,
    HTMLScriptElement,
    HTMLSectionElement,
    HTMLSelectElement,
    HTMLSmallElement,
    HTMLSpanElement,
    HTMLStyleElement,
    HTMLTableBodyElement,
    HTMLTableCellElement,
    HTMLTableElement,
    HTMLTableRowElement,
    HTMLTitleElement,
    HTMLUListElement,
    UnknownElement,
}
