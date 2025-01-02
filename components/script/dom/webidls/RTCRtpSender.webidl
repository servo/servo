/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender

dictionary RTCRtpHeaderExtensionParameters {
  required DOMString uri;
  required unsigned short id;
  boolean encrypted = false;
};

dictionary RTCRtcpParameters {
  DOMString cname;
  boolean reducedSize;
};

dictionary RTCRtpCodecParameters {
  required octet payloadType;
  required DOMString mimeType;
  required unsigned long clockRate;
  unsigned short channels;
  DOMString sdpFmtpLine;
};

dictionary RTCRtpParameters {
  required sequence<RTCRtpHeaderExtensionParameters> headerExtensions;
  required RTCRtcpParameters rtcp;
  required sequence<RTCRtpCodecParameters> codecs;
};

dictionary RTCRtpSendParameters : RTCRtpParameters {
  required DOMString transactionId;
  required sequence<RTCRtpEncodingParameters> encodings;
};

[Exposed=Window, Pref="dom.webrtc.transceiver.enabled"]
interface RTCRtpSender {
  //readonly attribute MediaStreamTrack? track;
  //readonly attribute RTCDtlsTransport? transport;
  //static RTCRtpCapabilities? getCapabilities(DOMString kind);
  Promise<undefined> setParameters(RTCRtpSendParameters parameters);
  RTCRtpSendParameters getParameters();
  //Promise<void> replaceTrack(MediaStreamTrack? withTrack);
  //void setStreams(MediaStream... streams);
  //Promise<RTCStatsReport> getStats();
};
