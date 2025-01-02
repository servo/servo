function supportsCodec(mimeType) {
  return RTCRtpSender.getCapabilities('video').codecs.filter(c => c.mimeType == mimeType).length() > 0;
}

async function supportsScalabilityMode(mimeType, scalabilityMode) {
  let result = await navigator.mediaCapabilities.encodingInfo({
    type: 'webrtc',
    video: {
      contentType: mimeType,
      width: 60,
      height: 60,
      bitrate: 10000,
      framerate: 30,
      scalabilityMode: scalabilityMode
    }
  });
  return result.supported;
}

function createScalabilityTest(mimeType, scalabilityModes) {
  for (const scalabilityMode of scalabilityModes) {
    promise_test(async t => {
      assert_implements_optional(
        supportsScalabilityMode(mimeType, scalabilityMode),
        `${mimeType} supported`
      );
      const v = document.createElement('video');
      v.autoplay = true;
      const pc1 = new RTCPeerConnection();
      const pc2 = new RTCPeerConnection();
      t.add_cleanup(() => pc1.close());
      t.add_cleanup(() => pc2.close());
      const stream1 = await getNoiseStream({ video: { signal: 100, width: 60, height: 60 } });
      const [track1] = stream1.getTracks();
      t.add_cleanup(() => track1.stop());
      const transceiver = pc1.addTransceiver(track1, {
        sendEncodings: [{ scalabilityMode: scalabilityMode }],
      });
      pc2.addEventListener('track', e => {
        e.transceiver.setCodecPreferences(RTCRtpReceiver.getCapabilities('video').codecs.filter(c => c.mimeType == mimeType));
      });
      const haveTrackEvent = new Promise(r => pc2.ontrack = r);
      exchangeIceCandidates(pc1, pc2);
      await exchangeOfferAnswer(pc1, pc2);
      v.srcObject = new MediaStream([(await haveTrackEvent).track]);
      await new Promise(r => v.onloadedmetadata = r);
      await detectSignal(t, v, 100);
      const sendParams = pc1.getSenders()[0].getParameters();
      assert_equals(sendParams.encodings[0].scalabilityMode, scalabilityMode);
    }, `${mimeType} - ${scalabilityMode} should produce valid video content`);
  }
}
