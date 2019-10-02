/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#rtcsessiondescription-class

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCSessionDescription {
    [Throws] constructor(RTCSessionDescriptionInit descriptionInitDict);
    readonly attribute RTCSdpType type;
    readonly attribute DOMString  sdp;
    [Default] object toJSON();
};

dictionary RTCSessionDescriptionInit {
    required RTCSdpType type;
             DOMString  sdp = "";
};

enum RTCSdpType {
    "offer",
    "pranswer",
    "answer",
    "rollback"
};
