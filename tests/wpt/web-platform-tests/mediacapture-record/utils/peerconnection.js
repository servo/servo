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
  await pc1.setLocalDescription(await pc1.createOffer());
  await pc2.setRemoteDescription(pc1.localDescription);
  await pc2.setLocalDescription(await pc2.createAnswer());
  await pc1.setRemoteDescription(pc2.localDescription);
}

/**
 * Sets the specified codec preference if it's included in the transceiver's
 * list of supported codecs.
 * @param {!RTCRtpTransceiver} transceiver The RTP transceiver.
 * @param {string} codecPreference The codec preference.
 */
function setTransceiverCodecPreference(transceiver, codecPreference) {
  for (let codec of RTCRtpSender.getCapabilities('video').codecs) {
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
 * @param {boolean} useAudio True if audio should be used.
 * @param {boolean} useVideo True if video should be used.
 * @param {string} [videoCodecPreference] String containing the codec preference.
 * @returns an array with the two connected peer connections, the remote stream,
 * and the list of transceivers.
 */
async function startConnection(t, useAudio, useVideo, videoCodecPreference) {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: useAudio, video: useVideo
  });
  t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
  const pc1 = new RTCPeerConnection();
  t.add_cleanup(() => pc1.close());
  const pc2 = new RTCPeerConnection();
  t.add_cleanup(() => pc2.close());
  let transceivers = {};
  stream.getTracks().forEach(track => {
    const transceiver = pc1.addTransceiver(track);
    transceivers[track.kind] = transceiver;
    if (videoCodecPreference && track.kind == 'video') {
      setTransceiverCodecPreference(transceiver, videoCodecPreference);
    }
  });
  function doExchange(localPc, remotePc) {
    localPc.addEventListener('icecandidate', event => {
      const { candidate } = event;
      if (candidate && remotePc.signalingState !== 'closed') {
        remotePc.addIceCandidate(candidate);
      }
    });
  }
  doExchange(pc1, pc2);
  doExchange(pc2, pc1);
  exchangeOfferAnswer(pc1, pc2);
  const remoteStream = await new Promise(resolve => {
    let tracks = [];
    pc2.ontrack = e => {
      tracks.push(e.track)
      if (tracks.length < useAudio + useVideo) return;
      const stream = new MediaStream(tracks);
      // The srcObject sink is needed for the tests to get exercised in Chrome.
      const remoteVideo = document.getElementById('remote');
      if (remoteVideo) {
        remoteVideo.srcObject = stream;
      }
      resolve(stream)
    }
  });
  return [pc1, pc2, remoteStream, transceivers]
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
async function waitForReceivedFrames(
    t, pc, lookForAudio, lookForVideo, numFramesOrPackets) {
  let initialAudioPackets = 0;
  let initialVideoFrames = 0;
  while (lookForAudio || lookForVideo) {
    const report = await pc.getStats();
    report.forEach(stats => {
      if (stats.type && stats.type == 'inbound-rtp') {
        if (lookForAudio && stats.kind == 'audio') {
          if (!initialAudioPackets) {
            initialAudioPackets = stats.packetsReceived
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
    });
    await new Promise(r => { t.step_timeout(r, 100); });
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
    report.forEach(stats => {
      if (stats.id) {
        if (stats.type == 'inbound-rtp' && stats.kind == 'video') {
          currentCodecId = stats.codecId;
        } else if (currentCodecId && stats.id == currentCodecId &&
                   stats.mimeType.toLowerCase().includes(
                      codecToLookFor.toLowerCase())) {
          return;
        }
      }
    });
    await new Promise(r => { t.step_timeout(r, 100); });
  }
}
