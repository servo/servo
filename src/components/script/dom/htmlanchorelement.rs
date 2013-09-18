/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::htmlelement::HTMLElement;
use dom::bindings::utils::{DOMString, ErrorResult};

pub struct HTMLAnchorElement {
    parent: HTMLElement
}

impl HTMLAnchorElement {
    pub fn Href(&self) -> DOMString {
        None
    }

    pub fn SetHref(&mut self, _href: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Target(&self) -> DOMString {
        None
    }

    pub fn SetTarget(&self, _target: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Download(&self) -> DOMString {
        None
    }

    pub fn SetDownload(&self, _download: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Ping(&self) -> DOMString {
        None
    }

    pub fn SetPing(&self, _ping: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Rel(&self) -> DOMString {
        None
    }

    pub fn SetRel(&self, _rel: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Hreflang(&self) -> DOMString {
        None
    }

    pub fn SetHreflang(&self, _href_lang: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Coords(&self) -> DOMString {
        None
    }

    pub fn SetCoords(&mut self, _coords: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Charset(&self) -> DOMString {
        None
    }

    pub fn SetCharset(&mut self, _charset: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Rev(&self) -> DOMString {
        None
    }

    pub fn SetRev(&mut self, _rev: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Shape(&self) -> DOMString {
        None
    }

    pub fn SetShape(&mut self, _shape: &DOMString, _rv: &mut ErrorResult) {
    }
}
