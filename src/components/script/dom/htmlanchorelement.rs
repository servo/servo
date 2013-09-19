/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::htmlelement::HTMLElement;
use dom::bindings::utils::{DOMString, ErrorResult};

pub struct HTMLAnchorElement {
    htmlelement: HTMLElement
}

impl HTMLAnchorElement {
    pub fn Href(&self) -> DOMString {
        None
    }

    pub fn SetHref(&mut self, _href: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        None
    }

    pub fn SetTarget(&self, _target: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Download(&self) -> DOMString {
        None
    }

    pub fn SetDownload(&self, _download: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ping(&self) -> DOMString {
        None
    }

    pub fn SetPing(&self, _ping: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rel(&self) -> DOMString {
        None
    }

    pub fn SetRel(&self, _rel: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Hreflang(&self) -> DOMString {
        None
    }

    pub fn SetHreflang(&self, _href_lang: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Coords(&self) -> DOMString {
        None
    }

    pub fn SetCoords(&mut self, _coords: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> DOMString {
        None
    }

    pub fn SetCharset(&mut self, _charset: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rev(&self) -> DOMString {
        None
    }

    pub fn SetRev(&mut self, _rev: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Shape(&self) -> DOMString {
        None
    }

    pub fn SetShape(&mut self, _shape: &DOMString) -> ErrorResult {
        Ok(())
    }
}
