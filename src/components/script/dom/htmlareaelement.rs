/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLAreaElement {
    parent: HTMLElement
}

impl HTMLAreaElement {
    pub fn Alt(&self) -> DOMString {
        null_string
    }

    pub fn SetAlt(&self, _alt: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Coords(&self) -> DOMString {
        null_string
    }

    pub fn SetCoords(&self, _coords: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Shape(&self) -> DOMString {
        null_string
    }

    pub fn SetShape(&self, _shape: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Href(&self) -> DOMString {
        null_string
    }

    pub fn SetHref(&self, _href: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Target(&self) -> DOMString {
        null_string
    }

    pub fn SetTarget(&self, _target: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Download(&self) -> DOMString {
        null_string
    }

    pub fn SetDownload(&self, _download: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Ping(&self) -> DOMString {
        null_string
    }

    pub fn SetPing(&self, _ping: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn NoHref(&self) -> bool {
        false
    }

    pub fn SetNoHref(&mut self, _no_href: bool, _rv: &mut ErrorResult) {
    }
}
