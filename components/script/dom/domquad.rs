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
use std::cmp::{max, min};
use url::Url;
use util::str::DOMString;

fn create_bound(p1: &DOMPointInit, p2: &DOMPointInit, p3: &DOMPointInit, p4: &DOMPointInit) -> DOMRectReadOnly {
    let left = min(min(min(p1.x, p2.x), p3.x), p4.x);
    let top = min(min(min(p1.y, p2.y), p3.y), p4.y);
    let right = max(max(max(p1.x, p2.x), p3.x), p4.x);
    let bottom = max(max(max(p1.y, p2.y), p3.y), p4.y);

    DOMRectReadOnly::new_inherited(left, top, right - left, bottom - top)
}

// https://drafts.fxtf.org/geometry/#DOMQuad
#[dom_struct]
pub struct DOMQuad {
    reflector_: Reflector,
    p1: JS<DOMPointReadOnly>,
    p2: JS<DOMPointReadOnly>,
    p3: JS<DOMPointReadOnly>,
    p4: JS<DOMPointReadOnly>,
    bounds: JS<DOMRectReadOnly>,
}

impl DOMQuad {
    fn new_inherited(global: GlobalRef,
                     p1: DOMPointInit,
                     p2: DOMPointInit,
                     p3: DOMPointInit,
                     p4: DOMPointInit) -> Root<DOMQuad> {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            reflector_: Reflector::new(),
            p1: DOMPointReadOnly::from_DOM_point_init(global, p1),
            p2: DOMPointReadOnly::from_DOM_point_init(global, p2),
            p3: DOMPointReadOnly::from_DOM_point_init(global, p3),
            p4: DOMPointReadOnly::from_DOM_point_init(global, p4),
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
    fn FromRect(global: GlobalRef, other: DOMRectInit) -> DOMQuad {
        DOMQuad {
            p1: DOMPointReadOnly::new(global, other.X(), other.Y(), 0, 1),
            p2: DOMPointReadOnly::new(global, other.X() + other.Width(), other.Y(), 0, 1),
            p3: DOMPointReadOnly::new(global, other.X() + other.Width(), other.Y() + other.Height(), 0, 1),
            p4: DOMPointReadOnly::new(global, other.X(), other.Y() + other.Height(), 0, 1),
            bounds: DOMRectReadOnly::new(global, other.X(), other.Y(), other.Width(), other.Height()),
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    fn FromQuad(global: GlobalRef, other: DOMQuadInit) -> DOMQuad {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            p1: DOMPoint::from_DOM_point(global, other.p1),
            p2: DOMPoint::from_DOM_point(global, other.p2),
            p3: DOMPoint::from_DOM_point(global, other.p3),
            p4: DOMPoint::from_DOM_point(global, other.p4),
            bounds: bounds,
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> Root<DOMPointReadOnly> {
        self.p1
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn SetP1(&mut self, p1: DOMPointInit) {
        self.p1 = p1;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> Root<DOMPointReadOnly> {
        self.p2
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn SetP2(&mut self, p2: DOMPointInit) {
        self.p2 = p2;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> Root<DOMPointReadOnly> {
        self.p3
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn SetP3(&mut self, p3: DOMPointInit) {
        self.p3 = p3;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> Root<DOMPointReadOnly> {
        self.p4
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn SetP4(&mut self, p4: DOMPointInit) {
        self.p4 = p4;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }
}

struct DOMPointInit {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl DOMPointInit {
    pub fn new() -> DOMPointInit {
        DOMPointInit {
            x: 0,
            y: 0,
            z: 0,
            w: 1,
        }
    }
}
