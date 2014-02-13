/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::ElementTypeId;
use dom::htmlelement::HTMLElement;

pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
}

impl HTMLMediaElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: AbstractDocument) -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, document)
        }
    }
}

impl HTMLMediaElement {
    pub fn Src(&self) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CurrentSrc(&self) -> DOMString {
        ~""
    }

    pub fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Preload(&self) -> DOMString {
        ~""
    }

    pub fn SetPreload(&mut self, _preload: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Load(&self) {
    }

    pub fn CanPlayType(&self, _type: DOMString) -> DOMString {
        ~""
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

    pub fn SetCurrentTime(&mut self, _current_time: f64) -> ErrorResult {
        Ok(())
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

    pub fn SetDefaultPlaybackRate(&mut self, _default_playback_rate: f64) -> ErrorResult {
        Ok(())
    }

    pub fn PlaybackRate(&self) -> f64 {
        0f64
    }

    pub fn SetPlaybackRate(&mut self, _playback_rate: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Ended(&self) -> bool {
        false
    }

    pub fn Autoplay(&self) -> bool {
        false
    }

    pub fn SetAutoplay(&mut self, _autoplay: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Loop(&self) -> bool {
        false
    }

    pub fn SetLoop(&mut self, _loop: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Play(&self) -> ErrorResult {
        Ok(())
    }

    pub fn Pause(&self) -> ErrorResult {
        Ok(())
    }

    pub fn Controls(&self) -> bool {
        false
    }

    pub fn SetControls(&mut self, _controls: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Volume(&self) -> f64 {
        0f64
    }

    pub fn SetVolume(&mut self, _volume: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Muted(&self) -> bool {
        false
    }

    pub fn SetMuted(&mut self, _muted: bool) {
    }

    pub fn DefaultMuted(&self) -> bool {
        false
    }

    pub fn SetDefaultMuted(&mut self, _default_muted: bool) -> ErrorResult {
        Ok(())
    }
}

