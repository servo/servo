/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLAreaElement {
    htmlelement: HTMLElement
}

impl HTMLAreaElement {
    pub fn Alt(&self) -> DOMString {
        None
    }

    pub fn SetAlt(&self, _alt: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Coords(&self) -> DOMString {
        None
    }

    pub fn SetCoords(&self, _coords: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Shape(&self) -> DOMString {
        None
    }

    pub fn SetShape(&self, _shape: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Href(&self) -> DOMString {
        None
    }

    pub fn SetHref(&self, _href: &DOMString) -> ErrorResult {
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

    pub fn NoHref(&self) -> bool {
        false
    }

    pub fn SetNoHref(&mut self, _no_href: bool) -> ErrorResult {
        Ok(())
    }
}
