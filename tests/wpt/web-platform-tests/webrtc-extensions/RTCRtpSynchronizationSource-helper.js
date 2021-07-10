'use strict';

// This file depends on `webrtc/RTCPeerConnection-helper.js`
// which should be loaded from the main HTML file.

var kAbsCaptureTime =
    'http://www.webrtc.org/experiments/rtp-hdrext/abs-capture-time';

function addHeaderExtensionToSdp(sdp, uri) {
  const extmap = new RegExp('a=extmap:(\\d+)');
  let sdpLines = sdp.split('\r\n');

  // This assumes at most one audio m= section and one video m= section.
  // If more are present, only the first section of each kind is munged.
  for (const section of ['audio', 'video']) {
    let found_section = false;
    let maxId = undefined;
    let maxIdLine = undefined;
    let extmapAllowMixed = false;

    // find the largest header extension id for section.
    for (let i = 0; i < sdpLines.length; ++i) {
      if (!found_section) {
        if (sdpLines[i].startsWith('m=' + section)) {
          found_section = true;
        }
        continue;
      } else {
        if (sdpLines[i].startsWith('m=')) {
          // end of section
          break;
        }
      }

      if (sdpLines[i] === 'a=extmap-allow-mixed') {
        extmapAllowMixed = true;
      }
      let result = sdpLines[i].match(extmap);
      if (result && result.length === 2) {
        if (maxId == undefined || result[1] > maxId) {
          maxId = parseInt(result[1]);
          maxIdLine = i;
        }
      }
    }

    if (maxId == 14 && !extmapAllowMixed) {
      // Reaching the limit of one byte header extension. Adding two byte header
      // extension support.
      sdpLines.splice(maxIdLine + 1, 0, 'a=extmap-allow-mixed');
    }
    if (maxIdLine !== undefined) {
      sdpLines.splice(maxIdLine + 1, 0,
                      'a=extmap:' + (maxId + 1).toString() + ' ' + uri);
    }
  }
  return sdpLines.join('\r\n');
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
