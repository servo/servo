/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use crate::dom::bindings::codegen::Bindings::SpeechSynthesisUtteranceBinding::SpeechSynthesisUtteranceMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::SpeechSynthesisVoice;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use js::gc::HandleObject;
use net_traits::speech_thread::Utterance;
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use std::sync::Mutex;

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

    pub fn new(global: &GlobalScope, inner: Utterance, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(inner)), global, can_gc)
    }

    pub fn inner(&self) -> Utterance {
        self.inner.lock().unwrap().clone()
    }
}

impl SpeechSynthesisUtteranceMethods<crate::DomTypeHolder> for SpeechSynthesisUtterance {
    fn Constructor(global: &Window, proto: Option<HandleObject>, can_gc: CanGc, text: Option<DOMString>) -> DomRoot<SpeechSynthesisUtterance> {
        Self::new(global.as_global_scope(), Utterance {
            text: text.map_or_else(|| String::new(), |s| s.to_string()),
            ..Default::default()
        }, can_gc)
    }

    fn Text(&self) -> DOMString {
        DOMString::from_string(self.inner.lock().unwrap().text.clone())
    }

    fn SetText(&self, text: DOMString) {
        self.inner.lock().unwrap().text = text.to_string();
    }

    // TODO: make this nullable
    fn Lang(&self) -> DOMString {
        DOMString::from_string(self.inner.lock().unwrap().lang.clone().unwrap_or_default())
    }

    fn SetLang(&self, lang: DOMString) {
        self.inner.lock().unwrap().lang = Some(lang.to_string());
    }

    fn GetVoice(&self) -> Option<DomRoot<SpeechSynthesisVoice>> {
        self.inner.lock().unwrap().voice.clone().map(|voice| {
            SpeechSynthesisVoice::new(&self.global(), voice, CanGc::note())
        })
    }

    fn SetVoice(&self, value: Option<&SpeechSynthesisVoice>) {
        self.inner.lock().unwrap().voice = value.map(|v| v.inner().clone());
    }

    fn Volume(&self) -> Finite<f32> {
        Finite::wrap(self.inner.lock().unwrap().volume)
    }

    fn SetVolume(&self, value: Finite<f32>) {
        self.inner.lock().unwrap().volume = *value;
    }

    fn Rate(&self) -> Finite<f32> {
        Finite::wrap(self.inner.lock().unwrap().rate)
    }

    fn SetRate(&self, value: Finite<f32>) {
        self.inner.lock().unwrap().rate = *value;
    }

    fn Pitch(&self) -> Finite<f32> {
        Finite::wrap(self.inner.lock().unwrap().pitch)
    }

    fn SetPitch(&self, value: Finite<f32>) {
        self.inner.lock().unwrap().pitch = *value;
    }

    event_handler!(onstart, GetOnstart, SetOnstart);

    event_handler!(onend, GetOnend, SetOnend);

    event_handler!(onerror, GetOnerror, SetOnerror);

    event_handler!(onpause, GetOnpause, SetOnpause);

    event_handler!(onresume, GetOnresume, SetOnresume);

    event_handler!(onmark, GetOnmark, SetOnmark);

    event_handler!(onboundary, GetOnboundary, SetOnboundary);
}
