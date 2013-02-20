use core::dvec::DVec;
use geom::size::Size2D;
use std::net::url::Url;

pub struct ElementData {
    tag_name: ~str,
    kind: ~ElementKind,
    attrs: DVec<~Attr>,
}

#[allow(non_implicitly_copyable_typarams)]
impl ElementData {
    fn get_attr(name: &str) -> Option<~str> {
        let found = do self.attrs.find |attr| { name == attr.name };
        match found {
            Some(attr) => Some(copy attr.value),
            None => None
        }
    }

    // Gets an attribute without copying.
    //
    // FIXME: Should not take a closure, but we need covariant type parameters for
    // that.
    fn with_attr<R>(name: &str, f: &fn(Option<&str>) -> R) -> R {
        for self.attrs.each |attr| {
            if name == attr.name {
                let value: &str = attr.value;
                return f(Some(value));
            }
        }
        f(None)
    }

    fn set_attr(name: &str, value: ~str) {
        let idx = do self.attrs.position |attr| { name == attr.name };
        match idx {
            Some(idx) => self.attrs.set_elt(idx, ~Attr(name.to_str(), value)),
            None => {}
        }
    }
}

pub fn ElementData(tag_name: ~str, kind: ~ElementKind) -> ElementData {
    ElementData {
        tag_name : tag_name,
        kind : kind,
        attrs : DVec(),
    }
}

pub struct Attr {
    name: ~str,
    value: ~str,
}

pub fn Attr(name: ~str, value: ~str) -> Attr {
    Attr {
        name : name,
        value : value,
    }
}

pub fn HTMLImageData() -> HTMLImageData {
    HTMLImageData {
        image: None
    }
}

pub struct HTMLImageData {
    mut image: Option<Url>
}

pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

pub enum ElementKind {
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
    HTMLImageElement(HTMLImageData),
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
