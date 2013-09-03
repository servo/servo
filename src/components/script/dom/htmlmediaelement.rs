/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::element::ElementTypeId;
use dom::htmlelement::HTMLElement;

pub struct HTMLMediaElement {
    parent: HTMLElement,
}

impl HTMLMediaElement {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> HTMLMediaElement {
        HTMLMediaElement {
            parent: HTMLElement::new(type_id, tag_name)
        }
    }
}

impl HTMLMediaElement {
    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CurrentSrc(&self) -> DOMString {
        null_string
    }

    pub fn CrossOrigin(&self) -> DOMString {
        null_string
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Preload(&self) -> DOMString {
        null_string
    }

    pub fn SetPreload(&mut self, _preload: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Load(&self) {
    }

    pub fn CanPlayType(&self, _type: &DOMString) -> DOMString {
        null_string
    }

    pub fn ReadyState(&self) -> u16 {
        0
    }

    pub fn Seeking(&self) -> bool {
        false
    }

    pub fn CurrentTime(&self) -> f64 {
        0f64
    }

    pub fn SetCurrentTime(&mut self, _current_time: f64, _rv: &mut ErrorResult) {
    }

    pub fn GetDuration(&self) -> f64 {
        0f64
    }

    pub fn Paused(&self) -> bool {
        false
    }

    pub fn DefaultPlaybackRate(&self) -> f64 {
        0f64
    }

    pub fn SetDefaultPlaybackRate(&mut self, _default_playback_rate: f64, _rv: &mut ErrorResult) {
    }

    pub fn PlaybackRate(&self) -> f64 {
        0f64
    }

    pub fn SetPlaybackRate(&mut self, _playback_rate: f64, _rv: &mut ErrorResult) {
    }

    pub fn Ended(&self) -> bool {
        false
    }

    pub fn Autoplay(&self) -> bool {
        false
    }

    pub fn SetAutoplay(&mut self, _autoplay: bool, _rv: &mut ErrorResult) {
    }

    pub fn Loop(&self) -> bool {
        false
    }

    pub fn SetLoop(&mut self, _loop: bool, _rv: &mut ErrorResult) {
    }

    pub fn Play(&self, _rv: &mut ErrorResult) {
    }

    pub fn Pause(&self, _rv: &mut ErrorResult) {
    }

    pub fn Controls(&self) -> bool {
        false
    }

    pub fn SetControls(&mut self, _controls: bool, _rv: &mut ErrorResult) {
    }

    pub fn Volume(&self) -> f64 {
        0f64
    }

    pub fn SetVolume(&mut self, _volume: f64, _rv: &mut ErrorResult) {
    }

    pub fn Muted(&self) -> bool {
        false
    }

    pub fn SetMuted(&mut self, _muted: bool) {
    }

    pub fn DefaultMuted(&self) -> bool {
        false
    }

    pub fn SetDefaultMuted(&mut self, _default_muted: bool, _rv: &mut ErrorResult) {
    }
}
