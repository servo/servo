/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::Bindings::SpeechSynthesisBinding::SpeechSynthesisMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::eventtarget::EventTarget;
use crate::dom::speechsynthesisutterance::SpeechSynthesisUtterance;
use crate::dom::speechsynthesisvoice::SpeechSynthesisVoice;
use crate::dom::types::Window;

#[dom_struct]
pub(crate) struct SpeechSynthesis {
    eventtarget: EventTarget,
}

impl SpeechSynthesis {
    fn new_inherited() -> SpeechSynthesis {
        SpeechSynthesis {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub(crate) fn new(global: &Window, can_gc: CanGc) -> DomRoot<SpeechSynthesis> {
        reflect_dom_object(Box::new(SpeechSynthesis::new_inherited()), global, can_gc)
    }
}

impl SpeechSynthesisMethods<crate::DomTypeHolder> for SpeechSynthesis {
    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesis-cancel>
    fn Cancel(&self) {}

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesis-pause>
    fn Pause(&self) {}

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesis-resume>
    fn Resume(&self) {}

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesis-getvoices>
    fn GetVoices(&self) -> Vec<DomRoot<SpeechSynthesisVoice>> {
        vec![]
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesis-speak>
    fn Speak(&self, _utterance: &SpeechSynthesisUtterance) {}
}
