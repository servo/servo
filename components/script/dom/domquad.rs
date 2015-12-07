/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::XMLDocumentBinding::{self, XMLDocumentMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::dompoint::DOMPointReadOnly;
use dom::domrectreadonly::DOMRectReadOnly;
use dom::location::Location;
use dom::node::Node;
use dom::window::Window;
use js::jsapi::{JSContext, JSObject};
use url::Url;
use util::str::DOMString;

fn get<T>(values: &[T], comp_min: bool) -> T {
    let mut ret_value = 0;

    if comp_min {
        for x in values {
            if ret_value > x {
                ret_value = x;
            }
        }
    } else {
        for x in values {
            if ret_value < x {
                ret_value = x;
            }
        }
    }
    ret_value
}

fn create_bound(p1: &DOMPointInit, p2: &DOMPointInit, p3: &DOMPointInit, p4: &DOMPointInit) -> DOMRectReadOnly {
    let left = get(&vec!(p1.x, p2.x, p3.x, p4.x), true);
    let top = get(&vec!(p1.y, p2.y, p3.y, p4.y), true);
    let right = get(&vec!(p1.x, p2.x, p3.x, p4.x), false);
    let bottom = get(&vec!(p1.y, p2.y, p3.y, p4.y), false);

    DOMRectReadOnly::new_inherited(left, top, right - left, bottom - top)
}

// https://drafts.fxtf.org/geometry/#DOMQuad
#[dom_struct]
pub struct DOMQuad {
    p1: DOMPointReadOnly,
    p2: DOMPointReadOnly,
    p3: DOMPointReadOnly,
    p4: DOMPointReadOnly,
    bounds: DOMRectReadOnly,
}

impl DOMQuad {
    fn new_inherited(p1: DOMPointInit, p2: DOMPointInit, p3: DOMPointInit, p4: DOMPointInit) -> DOMQuad {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            p1: DOMPointReadOnly::FromDOMPointInit(p1),
            p2: DOMPointReadOnly::FromDOMPointInit(p2),
            p3: DOMPointReadOnly::FromDOMPointInit(p3),
            p4: DOMPointReadOnly::FromDOMPointInit(p4),
            bounds: bounds,
        }
    }

    pub fn new(global: GlobalRef,
               p1: DOMPointInit,
               p2: DOMPointInit,
               p3: DOMPointInit,
               p4: DOMPointInit) -> Root<DOMQuad> {
        reflect_dom_object(box DOMQuad::new_inherited(p1, p2, p3, p4),
                           global,
                           DOMQuadBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef,
                       p1: DOMPointInit,
                       p2: DOMPointInit,
                       p3: DOMPointInit,
                       p4: DOMPointInit)
                       -> Fallible<Root<DOMQuad>> {
        Ok(DOMQuad::new(global, p1, p2, p3, p4))
    }
}

impl DOMQuadMethods for DOMQuad {
    // https://drafts.fxtf.org/geometry/#dom-domquad-fromrect
    fn FromRect(other: DOMRectInit) -> DOMQuad {
        DOMQuad {
            p1: DOMPointReadOnly::new_inherited(other.X(), other.Y(), 0, 1),
            p2: DOMPointReadOnly::new_inherited(other.X() + other.Width(), other.Y(), 0, 1),
            p3: DOMPointReadOnly::new_inherited(other.X() + other.Width(), other.Y() + other.Height(), 0, 1),
            p4: DOMPointReadOnly::new_inherited(other.X(), other.Y() + other.Height(), 0, 1),
            bounds: DOMRectReadOnly::new_inherited(other.X(), other.Y(), other.Width(), other.Height()),
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    fn FromQuad(other: DOMQuadInit) -> DOMQuad {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            p1: DOMPoint::FromPoint(other.p1),
            p2: DOMPoint::FromPoint(other.p2),
            p3: DOMPoint::FromPoint(other.p3),
            p4: DOMPoint::FromPoint(other.p4),
            bounds: bounds,
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> DOMPointReadOnly {
        self.p1
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn SetP1(&mut self, p1: DOMPointInit) -> DOMPointReadOnly {
        self.p1 = p1;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> DOMPointReadOnly {
        self.p2
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn SetP2(&mut self, p2: DOMPointInit) -> DOMPointReadOnly {
        self.p2 = p2;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> DOMPointReadOnly {
        self.p3
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn SetP3(&mut self, p3: DOMPointInit) -> DOMPointReadOnly {
        self.p3 = p3;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> DOMPointReadOnly {
        self.p4
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn SetP4(&mut self, p4: DOMPointInit) -> DOMPointReadOnly {
        self.p4 = p4;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }
}

struct DOMPointInit {
    unrestricted double x = 0;
    unrestricted double y = 0;
    unrestricted double z = 0;
    unrestricted double w = 1;
}
