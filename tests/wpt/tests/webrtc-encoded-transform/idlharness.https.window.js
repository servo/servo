// META: variant=?exclude=(SFrameDecrypterStream|SFrameEncrypterStream|SFrameSenderTransform|SFrameReceiverTransform|SFrameTransform.*)
// META: variant=?include=(SFrameDecrypterStream|SFrameEncrypterStream|SFrameSenderTransform|SFrameReceiverTransform|SFrameTransform.*)
// META: script=/common/subset-tests-by-key.js
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./RTCPeerConnection-helper.js

'use strict';

idl_test(
  ['webrtc-encoded-transform'],
  ['webrtc', 'streams', 'html', 'dom'],
  async idlArray => {
    idlArray.add_objects({
      // TODO: RTCEncodedVideoFrame
      // TODO: RTCEncodedAudioFrame
      RTCRtpSender: [`new RTCPeerConnection().addTransceiver('audio').sender`],
      RTCRtpReceiver: [`new RTCPeerConnection().addTransceiver('audio').receiver`],
    });
  }
);
