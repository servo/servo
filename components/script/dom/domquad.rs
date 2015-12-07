/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::DOMQuadBinding::{DOMQuadMethods, Wrap};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::XMLDocumentBinding::{self, XMLDocumentMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference, JS};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::dompoint::DOMPoint;
use dom::dompointreadonly::{DOMPointReadOnly, DOMPointWriteMethods};
use dom::domrectreadonly::DOMRectReadOnly;
use dom::location::Location;
use dom::node::Node;
use dom::window::Window;
use js::jsapi::{JSContext, JSObject};
use std::cmp::{max, min};
use url::Url;
use util::str::DOMString;

fn create_bound(p1: &DOMPointReadOnly, p2: &DOMPointReadOnly, p3: &DOMPointReadOnly, p4: &DOMPointReadOnly) -> DOMRectReadOnly {
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
                     p1: DOMPointReadOnly,
                     p2: DOMPointReadOnly,
                     p3: DOMPointReadOnly,
                     p4: DOMPointReadOnly) -> Root<DOMQuad> {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            reflector_: Reflector::new(),
            p1: JS::from_rooted(DOMPointReadOnly::from_point(global, p1)),
            p2: JS::from_rooted(DOMPointReadOnly::from_point(global, p2)),
            p3: JS::from_rooted(DOMPointReadOnly::from_point(global, p3)),
            p4: JS::from_rooted(DOMPointReadOnly::from_point(global, p4)),
            bounds: bounds,
        }
    }

    pub fn new(global: GlobalRef,
               p1: DOMPointReadOnly,
               p2: DOMPointReadOnly,
               p3: DOMPointReadOnly,
               p4: DOMPointReadOnly) -> Root<DOMQuad> {
        reflect_dom_object(box DOMQuad::new_inherited(p1, p2, p3, p4),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: GlobalRef,
                       p1: DOMPointReadOnly,
                       p2: DOMPointReadOnly,
                       p3: DOMPointReadOnly,
                       p4: DOMPointReadOnly)
                       -> Fallible<Root<DOMQuad>> {
        Ok(DOMQuad::new(global, p1, p2, p3, p4))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromrect
    fn FromRect(global: GlobalRef, other: DOMRectReadOnly) -> DOMQuad {
        DOMQuad {
            p1: DOMPointReadOnly::new(global, other.X(), other.Y(), 0, 1),
            p2: DOMPointReadOnly::new(global, other.X() + other.Width(), other.Y(), 0, 1),
            p3: DOMPointReadOnly::new(global, other.X() + other.Width(), other.Y() + other.Height(), 0, 1),
            p4: DOMPointReadOnly::new(global, other.X(), other.Y() + other.Height(), 0, 1),
            bounds: DOMRectReadOnly::new(global, other.X(), other.Y(), other.Width(), other.Height()),
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    fn FromQuad(global: GlobalRef, other: DOMQuad) -> DOMQuad {
        let bounds = create_bound(&other.p1, &other.p2, &other.p3, &other.p4);

        DOMQuad {
            p1: JS::from_rooted(DOMPoint::from_point(global, other.p1)),
            p2: JS::from_rooted(DOMPoint::from_point(global, other.p2)),
            p3: JS::from_rooted(DOMPoint::from_point(global, other.p3)),
            p4: JS::from_rooted(DOMPoint::from_point(global, other.p4)),
            bounds: bounds,
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn SetP1(&mut self, p1: DOMPointReadOnly) {
        self.p1 = p1;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn SetP2(&mut self, p2: DOMPointReadOnly) {
        self.p2 = p2;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn SetP3(&mut self, p3: DOMPointReadOnly) {
        self.p3 = p3;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn SetP4(&mut self, p4: DOMPointReadOnly) {
        self.p4 = p4;
        self.bounds = create_bound(&self.p1, &self.p2, &self.p3, &self.p4);
    }
}

impl DOMQuadMethods for DOMQuad {

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> Root<DOMPointReadOnly> {
        self.p1
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> Root<DOMPointReadOnly> {
        self.p2
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> Root<DOMPointReadOnly> {
        self.p3
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> Root<DOMPointReadOnly> {
        self.p4
    }
}
