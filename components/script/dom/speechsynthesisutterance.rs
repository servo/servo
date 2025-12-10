/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use speech_traits::synthesis::Utterance;

use crate::dom::bindings::codegen::Bindings::SpeechSynthesisUtteranceBinding::SpeechSynthesisUtteranceMethods;
use crate::dom::bindings::reflector::{
    DomGlobal, reflect_dom_object, reflect_dom_object_with_proto,
};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::SpeechSynthesisVoice;
use crate::dom::window::Window;

#[dom_struct]
pub struct SpeechSynthesisUtterance {
    eventtarget: EventTarget,
    #[no_trace]
    inner: RefCell<Utterance>,
}

impl SpeechSynthesisUtterance {
    pub fn new_inherited(inner: Utterance) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            inner: RefCell::new(inner),
        }
    }

    #[expect(dead_code)]
    pub fn new(global: &GlobalScope, inner: Utterance, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(inner)), global, can_gc)
    }

    pub fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        inner: Utterance,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(inner)),
            window.as_global_scope(),
            proto,
            can_gc,
        )
    }

    #[expect(dead_code)]
    pub fn inner(&self) -> Utterance {
        self.inner.borrow().clone()
    }
}

impl SpeechSynthesisUtteranceMethods<crate::DomTypeHolder> for SpeechSynthesisUtterance {
    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-speechsynthesisutterance>
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        text: Option<DOMString>,
    ) -> DomRoot<SpeechSynthesisUtterance> {
        let utterance = Utterance {
            text: text.map_or(String::new(), |t| t.to_string()),
            ..Default::default()
        };
        Self::new_with_proto(global, proto, can_gc, utterance)
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-text>
    fn Text(&self) -> DOMString {
        DOMString::from_string(self.inner.borrow_mut().text.clone())
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-text>
    fn SetText(&self, text: DOMString) {
        self.inner.borrow_mut().text = text.to_string();
    }

    // TODO: make this nullable
    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-lang>
    fn Lang(&self) -> DOMString {
        DOMString::from_string(self.inner.borrow_mut().lang.clone().unwrap_or_default())
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-lang>
    fn SetLang(&self, lang: DOMString) {
        self.inner.borrow_mut().lang = Some(lang.to_string());
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-voice>
    fn GetVoice(&self) -> Option<DomRoot<SpeechSynthesisVoice>> {
        self.inner
            .borrow_mut()
            .voice
            .clone()
            .map(|voice| SpeechSynthesisVoice::new(&self.global(), voice, CanGc::note()))
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-voice>
    fn SetVoice(&self, value: Option<&SpeechSynthesisVoice>) {
        self.inner.borrow_mut().voice = value.map(|v| v.inner().clone());
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-volume>
    fn Volume(&self) -> Finite<f32> {
        Finite::wrap(self.inner.borrow_mut().volume)
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-volume>
    fn SetVolume(&self, value: Finite<f32>) {
        self.inner.borrow_mut().volume = *value;
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-rate>
    fn Rate(&self) -> Finite<f32> {
        Finite::wrap(self.inner.borrow_mut().rate)
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-rate>
    fn SetRate(&self, value: Finite<f32>) {
        self.inner.borrow_mut().rate = *value;
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-pitch>
    fn Pitch(&self) -> Finite<f32> {
        Finite::wrap(self.inner.borrow_mut().pitch)
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-pitch>
    fn SetPitch(&self, value: Finite<f32>) {
        self.inner.borrow_mut().pitch = *value;
    }

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onstart
    event_handler!(onstart, GetOnstart, SetOnstart);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onend
    event_handler!(onend, GetOnend, SetOnend);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onerror
    event_handler!(onerror, GetOnerror, SetOnerror);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onpause
    event_handler!(onpause, GetOnpause, SetOnpause);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onresume
    event_handler!(onresume, GetOnresume, SetOnresume);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onmark
    event_handler!(onmark, GetOnmark, SetOnmark);

    // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-onboundary
    event_handler!(onboundary, GetOnboundary, SetOnboundary);
}
