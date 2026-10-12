/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::codegen::Bindings::RTCRtpSenderBinding::{
    RTCRtcpParameters, RTCRtpParameters, RTCRtpSendParameters, RTCRtpSenderMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::{Promise, RootedPromise};

#[dom_struct]
pub(crate) struct RTCRtpSender {
    reflector_: Reflector,
    track: DomRefCell<Option<Dom<MediaStreamTrack>>>,
}

impl RTCRtpSender {
    fn new_inherited(track: Option<&MediaStreamTrack>) -> Self {
        Self {
            reflector_: Reflector::new(),
            track: DomRefCell::new(track.map(Dom::from_ref)),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        track: Option<&MediaStreamTrack>,
    ) -> DomRoot<Self> {
        reflect_dom_object(cx, Box::new(Self::new_inherited(track)), global)
    }

    pub(crate) fn set_track(&self, track: Option<&MediaStreamTrack>) {
        *self.track.borrow_mut() = track.map(Dom::from_ref);
    }
}

impl RTCRtpSenderMethods<crate::DomTypeHolder> for RTCRtpSender {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-track>
    fn Track(&self) -> Option<DomRoot<MediaStreamTrack>> {
        self.track.borrow().as_ref().map(|t| DomRoot::from_ref(&**t))
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-getparameters>
    fn GetParameters(&self) -> RTCRtpSendParameters {
        RTCRtpSendParameters {
            parent: RTCRtpParameters {
                headerExtensions: vec![],
                rtcp: RTCRtcpParameters {
                    cname: None,
                    reducedSize: None,
                },
                codecs: vec![],
            },
            transactionId: DOMString::new(),
            encodings: vec![],
        }
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-setparameters>
    fn SetParameters(
        &self,
        cx: &mut JSContext,
        _parameters: &RTCRtpSendParameters,
    ) -> RootedPromise {
        Promise::new_resolved(cx, &self.global(), ())
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-replacetrack>
    fn ReplaceTrack(
        &self,
        cx: &mut CurrentRealm,
        with_track: Option<&MediaStreamTrack>,
    ) -> RootedPromise {
        let p = Promise::new_in_realm(cx);
        *self.track.borrow_mut() = with_track.map(Dom::from_ref);
        p.resolve_native(cx, &());
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-setstreams>
    fn SetStreams(&self, _streams: Vec<DomRoot<MediaStream>>) {
        // TODO: Implement setStreams
    }
}