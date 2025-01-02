/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#rtcicecandidate-interface


[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCIceCandidate {
    [Throws] constructor(optional RTCIceCandidateInit candidateInitDict = {});
    readonly attribute DOMString               candidate;
    readonly attribute DOMString?              sdpMid;
    readonly attribute unsigned short?         sdpMLineIndex;
    // readonly attribute DOMString?              foundation;
    // readonly attribute RTCIceComponent?        component;
    // readonly attribute unsigned long?          priority;
    // readonly attribute DOMString?              address;
    // readonly attribute RTCIceProtocol?         protocol;
    // readonly attribute unsigned short?         port;
    // readonly attribute RTCIceCandidateType?    type;
    // readonly attribute RTCIceTcpCandidateType? tcpType;
    // readonly attribute DOMString?              relatedAddress;
    // readonly attribute unsigned short?         relatedPort;
    readonly attribute DOMString?              usernameFragment;
    RTCIceCandidateInit toJSON();
};

dictionary RTCIceCandidateInit {
    DOMString       candidate = "";
    DOMString?      sdpMid = null;
    unsigned short? sdpMLineIndex = null;
    DOMString       usernameFragment;
};
