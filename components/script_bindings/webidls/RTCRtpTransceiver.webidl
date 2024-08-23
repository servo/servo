/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#rtcrtptransceiver-interface

[Exposed=Window, Pref="dom.webrtc.transceiver.enabled"]
interface RTCRtpTransceiver {
  //readonly attribute DOMString? mid;
  [SameObject] readonly attribute RTCRtpSender sender;
  //[SameObject] readonly attribute RTCRtpReceiver receiver;
  attribute RTCRtpTransceiverDirection direction;
  //readonly attribute RTCRtpTransceiverDirection? currentDirection;
  //void stop();
  //void setCodecPreferences(sequence<RTCRtpCodecCapability> codecs);
};

enum RTCRtpTransceiverDirection {
  "sendrecv",
  "sendonly",
  "recvonly",
  "inactive",
  "stopped"
};
