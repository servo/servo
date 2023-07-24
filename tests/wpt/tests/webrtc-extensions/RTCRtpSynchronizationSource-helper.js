'use strict';

// This file depends on `webrtc/RTCPeerConnection-helper.js`
// which should be loaded from the main HTML file.

var kAbsCaptureTime =
    'http://www.webrtc.org/experiments/rtp-hdrext/abs-capture-time';

function addHeaderExtensionToSdp(sdp, uri) {
  // Find the highest used header extension id by sorting the extension ids used,
  // eliminating duplicates and adding one. This is not quite correct
  // but this code will go away with the header extension API.
  const usedIds = sdp.split('\n')
    .filter(line => line.startsWith('a=extmap:'))
    .map(line => parseInt(line.split(' ')[0].substring(9), 10))
    .sort((a, b) => a - b)
    .filter((item, index, array) => array.indexOf(item) === index);
  const nextId = usedIds[usedIds.length - 1] + 1;
  const extmapLine = 'a=extmap:' + nextId + ' ' + uri + '\r\n';

  const sections = sdp.split('\nm=').map((part, index) => {
    return (index > 0 ? 'm=' + part : part).trim() + '\r\n';
  });
  const sessionPart = sections.shift();
  return sessionPart + sections.map(mediaSection => mediaSection + extmapLine).join('');
}

// TODO(crbug.com/1051821): Use RTP header extension API instead of munging
// when the RTP header extension API is implemented.
async function addAbsCaptureTimeAndExchangeOffer(caller, callee) {
  let offer = await caller.createOffer();

  // Absolute capture time header extension may not be offered by default,
  // in such case, munge the SDP.
  offer.sdp = addHeaderExtensionToSdp(offer.sdp, kAbsCaptureTime);

  await caller.setLocalDescription(offer);
  return callee.setRemoteDescription(offer);
}

// TODO(crbug.com/1051821): Use RTP header extension API instead of munging
// when the RTP header extension API is implemented.
async function checkAbsCaptureTimeAndExchangeAnswer(caller, callee,
                                                    absCaptureTimeAnswered) {
  let answer = await callee.createAnswer();

  const extmap = new RegExp('a=extmap:\\d+ ' + kAbsCaptureTime + '\r\n', 'g');
  if (answer.sdp.match(extmap) == null) {
    // We expect that absolute capture time RTP header extension is answered.
    // But if not, there is no need to proceed with the test.
    assert_false(absCaptureTimeAnswered, 'Absolute capture time RTP ' +
        'header extension is not answered');
  } else {
    if (!absCaptureTimeAnswered) {
      // We expect that absolute capture time RTP header extension is not
      // answered, but it is, then we munge the answer to remove it.
      answer.sdp = answer.sdp.replace(extmap, '');
    }
  }

  await callee.setLocalDescription(answer);
  return caller.setRemoteDescription(answer);
}

async function exchangeOfferAndListenToOntrack(t, caller, callee,
                                               absCaptureTimeOffered) {
  const ontrackPromise = addEventListenerPromise(t, callee, 'track');
  // Absolute capture time header extension is expected not offered by default,
  // and thus munging is needed to enable it.
  await absCaptureTimeOffered
      ? addAbsCaptureTimeAndExchangeOffer(caller, callee)
      : exchangeOffer(caller, callee);
  return ontrackPromise;
}

async function initiateSingleTrackCall(t, cap, absCaptureTimeOffered,
                                       absCaptureTimeAnswered) {
  const caller = new RTCPeerConnection();
  t.add_cleanup(() => caller.close());
  const callee = new RTCPeerConnection();
  t.add_cleanup(() => callee.close());

  const stream = await getNoiseStream(cap);
  stream.getTracks().forEach(track => {
    caller.addTrack(track, stream);
    t.add_cleanup(() => track.stop());
  });

  // TODO(crbug.com/988432): `getSynchronizationSources() on the audio side
  // needs a hardware sink for the returned dictionary entries to get updated.
  const remoteVideo = document.getElementById('remote');

  callee.ontrack = e => {
    remoteVideo.srcObject = e.streams[0];
  }

  exchangeIceCandidates(caller, callee);

  await exchangeOfferAndListenToOntrack(t, caller, callee,
                                        absCaptureTimeOffered);

  // Exchange answer and check whether the absolute capture time RTP header
  // extension is answered.
  await checkAbsCaptureTimeAndExchangeAnswer(caller, callee,
                                             absCaptureTimeAnswered);

  return [caller, callee];
}
