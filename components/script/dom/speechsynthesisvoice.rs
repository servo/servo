/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use speech_traits::synthesis::Voice;

use crate::dom::bindings::codegen::Bindings::SpeechSynthesisVoiceBinding::SpeechSynthesisVoiceMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct SpeechSynthesisVoice {
    reflector_: Reflector,
    #[no_trace]
    inner: Voice,
}

impl SpeechSynthesisVoice {
    pub fn new_inherited(inner: Voice) -> Self {
        Self {
            reflector_: Reflector::new(),
            inner,
        }
    }

    pub fn new(global: &GlobalScope, inner: Voice, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(inner)), global, can_gc)
    }

    pub fn inner(&self) -> &Voice {
        &self.inner
    }
}

impl SpeechSynthesisVoiceMethods<crate::DomTypeHolder> for SpeechSynthesisVoice {
    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisvoice-voiceuri>
    fn VoiceURI(&self) -> DOMString {
        DOMString::from_string(self.inner.uri.clone())
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisvoice-name>
    fn Name(&self) -> DOMString {
        DOMString::from_string(self.inner.name.clone())
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisvoice-lang>
    fn Lang(&self) -> DOMString {
        DOMString::from_string(self.inner.lang.clone())
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisvoice-localservice>
    fn LocalService(&self) -> bool {
        self.inner.local_service
    }

    /// <https://webaudio.github.io/web-speech-api/#dom-speechsynthesisvoice-default>
    fn Default(&self) -> bool {
        self.inner.default
    }
}
