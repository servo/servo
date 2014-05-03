/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JSRef};
use dom::bindings::codegen::InheritTypes::HTMLMediaElementDerived;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{ElementTypeId, HTMLAudioElementTypeId, HTMLVideoElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::ElementNodeTypeId;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLMediaElement {
    pub htmlelement: HTMLElement,
}

impl HTMLMediaElementDerived for EventTarget {
    fn is_htmlmediaelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLVideoElementTypeId)) |
            NodeTargetTypeId(ElementNodeTypeId(HTMLAudioElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMediaElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, document: &JSRef<Document>) -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, document)
        }
    }
}

pub trait HTMLMediaElementMethods {
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult;
    fn CurrentSrc(&self) -> DOMString;
    fn CrossOrigin(&self) -> DOMString;
    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult;
    fn Preload(&self) -> DOMString;
    fn SetPreload(&mut self, _preload: DOMString) -> ErrorResult;
    fn Load(&self);
    fn CanPlayType(&self, _type: DOMString) -> DOMString;
    fn ReadyState(&self) -> u16;
    fn Seeking(&self) -> bool;
    fn CurrentTime(&self) -> f64;
    fn SetCurrentTime(&mut self, _current_time: f64) -> ErrorResult;
    fn GetDuration(&self) -> f64;
    fn Paused(&self) -> bool;
    fn DefaultPlaybackRate(&self) -> f64;
    fn SetDefaultPlaybackRate(&mut self, _default_playback_rate: f64) -> ErrorResult;
    fn PlaybackRate(&self) -> f64;
    fn SetPlaybackRate(&mut self, _playback_rate: f64) -> ErrorResult;
    fn Ended(&self) -> bool;
    fn Autoplay(&self) -> bool;
    fn SetAutoplay(&mut self, _autoplay: bool) -> ErrorResult;
    fn Loop(&self) -> bool;
    fn SetLoop(&mut self, _loop: bool) -> ErrorResult;
    fn Play(&self) -> ErrorResult;
    fn Pause(&self) -> ErrorResult;
    fn Controls(&self) -> bool;
    fn SetControls(&mut self, _controls: bool) -> ErrorResult;
    fn Volume(&self) -> f64;
    fn SetVolume(&mut self, _volume: f64) -> ErrorResult;
    fn Muted(&self) -> bool;
    fn SetMuted(&mut self, _muted: bool);
    fn DefaultMuted(&self) -> bool;
    fn SetDefaultMuted(&mut self, _default_muted: bool) -> ErrorResult;
}

impl<'a> HTMLMediaElementMethods for JSRef<'a, HTMLMediaElement> {
    fn Src(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CurrentSrc(&self) -> DOMString {
        "".to_owned()
    }

    fn CrossOrigin(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Preload(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPreload(&mut self, _preload: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Load(&self) {
    }

    fn CanPlayType(&self, _type: DOMString) -> DOMString {
        "".to_owned()
    }

    fn ReadyState(&self) -> u16 {
        0
    }

    fn Seeking(&self) -> bool {
        false
    }

    fn CurrentTime(&self) -> f64 {
        0f64
    }

    fn SetCurrentTime(&mut self, _current_time: f64) -> ErrorResult {
        Ok(())
    }

    fn GetDuration(&self) -> f64 {
        0f64
    }

    fn Paused(&self) -> bool {
        false
    }

    fn DefaultPlaybackRate(&self) -> f64 {
        0f64
    }

    fn SetDefaultPlaybackRate(&mut self, _default_playback_rate: f64) -> ErrorResult {
        Ok(())
    }

    fn PlaybackRate(&self) -> f64 {
        0f64
    }

    fn SetPlaybackRate(&mut self, _playback_rate: f64) -> ErrorResult {
        Ok(())
    }

    fn Ended(&self) -> bool {
        false
    }

    fn Autoplay(&self) -> bool {
        false
    }

    fn SetAutoplay(&mut self, _autoplay: bool) -> ErrorResult {
        Ok(())
    }

    fn Loop(&self) -> bool {
        false
    }

    fn SetLoop(&mut self, _loop: bool) -> ErrorResult {
        Ok(())
    }

    fn Play(&self) -> ErrorResult {
        Ok(())
    }

    fn Pause(&self) -> ErrorResult {
        Ok(())
    }

    fn Controls(&self) -> bool {
        false
    }

    fn SetControls(&mut self, _controls: bool) -> ErrorResult {
        Ok(())
    }

    fn Volume(&self) -> f64 {
        0f64
    }

    fn SetVolume(&mut self, _volume: f64) -> ErrorResult {
        Ok(())
    }

    fn Muted(&self) -> bool {
        false
    }

    fn SetMuted(&mut self, _muted: bool) {
    }

    fn DefaultMuted(&self) -> bool {
        false
    }

    fn SetDefaultMuted(&mut self, _default_muted: bool) -> ErrorResult {
        Ok(())
    }
}
