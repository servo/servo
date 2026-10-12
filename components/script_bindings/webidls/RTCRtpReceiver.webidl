/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#dom-rtcrtpreceiver

[Exposed=Window, Pref="dom_webrtc_transceiver_enabled"]
interface RTCRtpReceiver {
  readonly attribute MediaStreamTrack track;
  // readonly attribute RTCDtlsTransport? transport;
  // static RTCRtpCapabilities? getCapabilities(DOMString kind);
  RTCRtpParameters getParameters();
  // sequence<RTCRtpContributingSource> getContributingSources();
  // sequence<RTCRtpSynchronizationSource> getSynchronizationSources();
  // Promise<RTCStatsReport> getStats();
};
