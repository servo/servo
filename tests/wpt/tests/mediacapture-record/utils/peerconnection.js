/**
 * @fileoverview Utility functions for tests utilizing PeerConnections
 */

/**
 * Exchanges offers and answers between two peer connections.
 *
 * pc1's offer is set as local description in pc1 and
 * remote description in pc2. After that, pc2's answer
 * is set as it's local description and remote description in pc1.
 *
 * @param {!RTCPeerConnection} pc1 The first peer connection.
 * @param {!RTCPeerConnection} pc2 The second peer connection.
 */
async function exchangeOfferAnswer(pc1, pc2) {
  await pc1.setLocalDescription();
  await pc2.setRemoteDescription(pc1.localDescription);
  await pc2.setLocalDescription();
  await pc1.setRemoteDescription(pc2.localDescription);
}

/**
 * Sets the specified codec preference if it's included in the transceiver's
 * list of supported codecs.
 * @param {!RTCRtpTransceiver} transceiver The RTP transceiver.
 * @param {string} codecPreference The codec preference.
 */
function setTransceiverCodecPreference(transceiver, codecPreference) {
  for (const codec of RTCRtpSender.getCapabilities('video').codecs) {
    if (codec.mimeType.includes(codecPreference)) {
      transceiver.setCodecPreferences([codec]);
      return;
    }
  }
}

/**
 * Starts a connection between two peer connections, using a audio and/or video
 * stream.
 * @param {*} t Test instance.
 * @param {boolean} audio True if audio should be used.
 * @param {boolean} video True if video should be used.
 * @param {string} [videoCodecPreference] String containing the codec preference.
 * @returns an array with the two connected peer connections, the remote stream,
 * and an object containing transceivers by kind.
 */
async function startConnection(t, audio, video, videoCodecPreference) {
  const scope = [];
  if (audio) scope.push("microphone");
  if (video) scope.push("camera");
  await setMediaPermission("granted", scope);
  const stream = await navigator.mediaDevices.getUserMedia({audio, video});
  t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
  const pc1 = new RTCPeerConnection();
  t.add_cleanup(() => pc1.close());
  const pc2 = new RTCPeerConnection();
  t.add_cleanup(() => pc2.close());
  const transceivers = {};
  for (const track of stream.getTracks()) {
    const transceiver = pc1.addTransceiver(track, {streams: [stream]});
    transceivers[track.kind] = transceiver;
    if (videoCodecPreference && track.kind == 'video') {
      setTransceiverCodecPreference(transceiver, videoCodecPreference);
    }
  }
  for (const [local, remote] of [[pc1, pc2], [pc2, pc1]]) {
    local.addEventListener('icecandidate', ({candidate}) => {
      if (!candidate || remote.signalingState == 'closed') return;
      remote.addIceCandidate(candidate);
    });
  }
  const haveTrackEvent = new Promise(r => pc2.ontrack = r);
  await exchangeOfferAnswer(pc1, pc2);
  const {streams} = await haveTrackEvent;
  return [pc1, pc2, streams[0], transceivers];
}

/**
 * Given a peer connection, return after at least numFramesOrPackets
 * frames (video) or packets (audio) have been received.
 * @param {*} t Test instance.
 * @param {!RTCPeerConnection} pc The peer connection.
 * @param {boolean} lookForAudio True if audio packets should be waited for.
 * @param {boolean} lookForVideo True if video packets should be waited for.
 * @param {int} numFramesOrPackets Number of frames (video) and packets (audio)
 * to wait for.
 */
async function waitForReceivedFramesOrPackets(
    t, pc, lookForAudio, lookForVideo, numFramesOrPackets) {
  let initialAudioPackets = 0;
  let initialVideoFrames = 0;
  while (lookForAudio || lookForVideo) {
    const report = await pc.getStats();
    for (const stats of report.values()) {
      if (stats.type == 'inbound-rtp') {
        if (lookForAudio && stats.kind == 'audio') {
          if (!initialAudioPackets) {
            initialAudioPackets = stats.packetsReceived;
          } else if (stats.packetsReceived > initialAudioPackets +
                     numFramesOrPackets) {
            lookForAudio = false;
          }
        }
        if (lookForVideo && stats.kind == 'video') {
          if (!initialVideoFrames) {
            initialVideoFrames = stats.framesDecoded;
          } else if (stats.framesDecoded > initialVideoFrames +
                     numFramesOrPackets) {
            lookForVideo = false;
          }
        }
      }
    }
    await new Promise(r => t.step_timeout(r, 100));
  }
}

/**
 * Given a peer connection, return after one of its inbound RTP connections
 * includes use of the specified codec.
 * @param {*} t Test instance.
 * @param {!RTCPeerConnection} pc The peer connection.
 * @param {string} codecToLookFor The waited-for codec.
 */
async function waitForReceivedCodec(t, pc, codecToLookFor) {
  let currentCodecId;
  for (;;) {
    const report = await pc.getStats();
    for (const stats of report.values()) {
      if (stats.type == 'inbound-rtp' && stats.kind == 'video') {
        if (stats.codecId) {
          if (report.get(stats.codecId).mimeType.toLowerCase()
              .includes(codecToLookFor.toLowerCase())) {
            return;
          }
        }
      }
    }
    await new Promise(r => t.step_timeout(r, 100));
  }
}
