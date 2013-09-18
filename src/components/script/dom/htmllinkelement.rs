/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLLinkElement {
    parent: HTMLElement,
}

impl HTMLLinkElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disable: bool) {
    }

    pub fn Href(&self) -> DOMString {
        None
    }

    pub fn SetHref(&mut self, _href: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CrossOrigin(&self) -> DOMString {
        None
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Rel(&self) -> DOMString {
        None
    }

    pub fn SetRel(&mut self, _rel: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Media(&self) -> DOMString {
        None
    }

    pub fn SetMedia(&mut self, _media: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Hreflang(&self) -> DOMString {
        None
    }

    pub fn SetHreflang(&mut self, _href: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Charset(&self) -> DOMString {
        None
    }

    pub fn SetCharset(&mut self, _charset: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Rev(&self) -> DOMString {
        None
    }

    pub fn SetRev(&mut self, _rev: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Target(&self) -> DOMString {
        None
    }

    pub fn SetTarget(&mut self, _target: &DOMString, _rv: &mut ErrorResult) {
    }
}
