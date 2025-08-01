/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use net_traits::speech_thread::Voice;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
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
    fn VoiceURI(&self) -> DOMString {
        DOMString::from_string(self.inner.uri.clone())
    }

    fn Name(&self) -> DOMString {
        DOMString::from_string(self.inner.name.clone())
    }

    fn Lang(&self) -> DOMString {
        DOMString::from_string(self.inner.lang.clone())
    }

    fn LocalService(&self) -> bool {
        self.inner.local_service
    }

    fn Default(&self) -> bool {
        self.inner.default
    }
}
