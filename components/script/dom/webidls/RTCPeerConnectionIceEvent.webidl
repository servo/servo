/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#rtcpeerconnectioniceevent

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCPeerConnectionIceEvent : Event {
    [Throws] constructor(DOMString type, optional RTCPeerConnectionIceEventInit eventInitDict = {});
    readonly attribute RTCIceCandidate? candidate;
    readonly attribute DOMString?       url;
};

dictionary RTCPeerConnectionIceEventInit : EventInit {
    RTCIceCandidate? candidate;
    DOMString?       url;
};
