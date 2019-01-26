/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding::RTCConfiguration;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::task::TaskCanceller;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;

use servo_media::webrtc::MediaStream as BackendMediaStream;
use servo_media::webrtc::{IceCandidate, WebRtcController, WebRtcSignaller};
use servo_media::ServoMedia;

#[dom_struct]
pub struct RTCPeerConnection {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    controller: DomRefCell<Option<WebRtcController>>,
}

struct RTCSignaller {
    trusted: Trusted<RTCPeerConnection>,
    task_source: NetworkingTaskSource,
    canceller: TaskCanceller,
}

impl WebRtcSignaller for RTCSignaller {
    fn on_ice_candidate(&self, _: &WebRtcController, candidate: IceCandidate) {
        let this = self.trusted.clone();
        let _ = self.task_source.queue_with_canceller(
            task!(on_ice_candidate: move || {
                let this = this.root();
                this.on_ice_candidate(candidate);
            }),
            &self.canceller,
        );
    }

    fn on_negotiation_needed(&self, _: &WebRtcController) {
        let this = self.trusted.clone();
        let _ = self.task_source.queue_with_canceller(
            task!(on_negotiation_needed: move || {
                let this = this.root();
                this.on_negotiation_needed();
            }),
            &self.canceller,
        );
    }

    fn on_add_stream(&self, _: Box<BackendMediaStream>) {}

    fn close(&self) {
        // do nothing
    }
}

impl RTCPeerConnection {
    pub fn new_inherited() -> RTCPeerConnection {
        RTCPeerConnection {
            eventtarget: EventTarget::new_inherited(),
            controller: DomRefCell::new(None),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<RTCPeerConnection> {
        let this = reflect_dom_object(
            Box::new(RTCPeerConnection::new_inherited()),
            global,
            RTCPeerConnectionBinding::Wrap,
        );
        let signaller = this.make_signaller();
        *this.controller.borrow_mut() = Some(ServoMedia::get().unwrap().create_webrtc(signaller));
        this
    }

    pub fn Constructor(
        window: &Window,
        _config: &RTCConfiguration,
    ) -> Fallible<DomRoot<RTCPeerConnection>> {
        Ok(RTCPeerConnection::new(&window.global()))
    }

    fn make_signaller(&self) -> Box<WebRtcSignaller> {
        let trusted = Trusted::new(self);
        let (task_source, canceller) = self
            .global()
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        Box::new(RTCSignaller {
            trusted,
            task_source,
            canceller,
        })
    }

    fn on_ice_candidate(&self, _candidate: IceCandidate) {
        // todo
    }

    fn on_negotiation_needed(&self) {
        // todo
    }
}
