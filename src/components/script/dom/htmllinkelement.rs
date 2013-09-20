/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
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

    pub fn SetHref(&mut self, _href: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CrossOrigin(&self) -> DOMString {
        None
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rel(&self) -> DOMString {
        None
    }

    pub fn SetRel(&mut self, _rel: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Media(&self) -> DOMString {
        None
    }

    pub fn SetMedia(&mut self, _media: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Hreflang(&self) -> DOMString {
        None
    }

    pub fn SetHreflang(&mut self, _href: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> DOMString {
        None
    }

    pub fn SetCharset(&mut self, _charset: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rev(&self) -> DOMString {
        None
    }

    pub fn SetRev(&mut self, _rev: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        None
    }

    pub fn SetTarget(&mut self, _target: &DOMString) -> ErrorResult {
        Ok(())
    }
}
