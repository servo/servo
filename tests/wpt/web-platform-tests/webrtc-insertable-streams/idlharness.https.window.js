// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./RTCPeerConnection-helper.js

'use strict';

idl_test(
  ['webrtc-insertable-streams'],
  ['webrtc'],
  async idlArray => {
    idlArray.add_objects({
      // TODO: RTCEncodedVideoFrame
      // TODO: RTCEncodedAudioFrame
      RTCRtpSender: [`new RTCPeerConnection().addTransceiver('audio').sender`],
      RTCRtpReceiver: [`new RTCPeerConnection().addTransceiver('audio').receiver`],
    });
  }
);
