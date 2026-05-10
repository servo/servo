// META: variant=?exclude=(SFrameDecrypterStream|SFrameEncrypterStream|SFrameSenderTransform|SFrameReceiverTransform|SFrameTransform.*)
// META: variant=?include=(SFrameDecrypterStream|SFrameEncrypterStream|SFrameSenderTransform|SFrameReceiverTransform|SFrameTransform.*)
// META: script=/common/subset-tests-by-key.js
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=../webrtc/RTCPeerConnection-helper.js
// META: script=helper.js
// META: timeout=long

'use strict';

const idlTestObjects = {};

idl_test(
  ['webrtc-encoded-transform'],
  ['webrtc', 'streams', 'html', 'dom'],
  async idlArray => {
    idlArray.add_objects({
      RTCRtpSender: [`new RTCPeerConnection().addTransceiver('audio').sender`],
      RTCRtpReceiver: [`new RTCPeerConnection().addTransceiver('audio').receiver`],
      RTCEncodedVideoFrame: [`idlTestObjects.videoFrame`],
      RTCEncodedAudioFrame: [`idlTestObjects.audioFrame`],
    });
    idlTestObjects.videoFrame = await createRTCEncodedFrameFromScratch("video");
    idlTestObjects.audioFrame = await createRTCEncodedFrameFromScratch("audio");
  }
);
