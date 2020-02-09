/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#dom-rtctrackevent

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCTrackEvent : Event {
    [Throws] constructor(DOMString type, RTCTrackEventInit eventInitDict);
    // readonly attribute RTCRtpReceiver           receiver;
    readonly attribute MediaStreamTrack         track;
    // [SameObject]
    // readonly attribute FrozenArray<MediaStream> streams;
    // readonly attribute RTCRtpTransceiver        transceiver;
};

// https://www.w3.org/TR/webrtc/#dom-rtctrackeventinit
dictionary RTCTrackEventInit : EventInit {
    // required RTCRtpReceiver        receiver;
    required MediaStreamTrack      track;
             // sequence<MediaStream> streams = [];
    // required RTCRtpTransceiver     transceiver;
};
