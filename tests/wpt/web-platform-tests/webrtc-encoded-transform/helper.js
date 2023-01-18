"use strict";

async function setupLoopbackWithCodecAndGetReader(t, codec) {
  const caller = new RTCPeerConnection({encodedInsertableStreams:true});
  t.add_cleanup(() => caller.close());
  const callee = new RTCPeerConnection();
  t.add_cleanup(() => callee.close());

  await setMediaPermission("granted", ["camera"]);
  const stream = await navigator.mediaDevices.getUserMedia({video:true});
  const videoTrack = stream.getVideoTracks()[0];
  t.add_cleanup(() => videoTrack.stop());

  const transceiver = caller.addTransceiver(videoTrack);
  const codecCapability =
      RTCRtpSender.getCapabilities('video').codecs.find(capability => {
        return capability.mimeType.includes(codec);
      });
  assert_not_equals(codecCapability, undefined);
  transceiver.setCodecPreferences([codecCapability]);

  const senderStreams = transceiver.sender.createEncodedStreams();
  exchangeIceCandidates(caller, callee);
  await exchangeOfferAnswer(caller, callee);
  return senderStreams.readable.getReader();
}
