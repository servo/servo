use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use net_traits::speech_thread::SpeechSynthesisThreadMsg;
use net_traits::IpcSend;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::Bindings::SpeechSynthesisBinding::SpeechSynthesisMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
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

    fn get_speech_synthesis_thread(&self) -> IpcSender<SpeechSynthesisThreadMsg> {
        self.global().resource_threads().sender()
    }
}

impl SpeechSynthesisMethods<crate::DomTypeHolder> for SpeechSynthesis {
    fn Cancel(&self) {
        let _ = self.get_speech_synthesis_thread().send(SpeechSynthesisThreadMsg::Cancel);
    }

    fn Pause(&self) {
        let _ = self.get_speech_synthesis_thread().send(SpeechSynthesisThreadMsg::Pause);
    }

    fn Resume(&self) {
        let _ = self.get_speech_synthesis_thread().send(SpeechSynthesisThreadMsg::Resume);
    }

    fn GetVoices(&self) -> Vec<DomRoot<SpeechSynthesisVoice>> {
        let (sender, receiver) = ipc_channel::ipc::channel().unwrap();
        let _ = self.get_speech_synthesis_thread().send(SpeechSynthesisThreadMsg::GetVoices(sender));
        let voices = receiver.recv().unwrap_or_default();
        let global = self.global();
        let voices: Vec<_> = voices.into_iter()
            .map(|voice| {
                SpeechSynthesisVoice::new(&global, voice, CanGc::note())
            })
            .collect();
        voices
    }

    fn Speak(&self, utterance: &SpeechSynthesisUtterance) {
        let _ = self.get_speech_synthesis_thread().send(SpeechSynthesisThreadMsg::Speak(utterance.inner()));
    }
}
