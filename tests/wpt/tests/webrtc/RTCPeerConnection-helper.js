'use strict'

/*
 *  Helper Methods for testing the following methods in RTCPeerConnection:
 *    createOffer
 *    createAnswer
 *    setLocalDescription
 *    setRemoteDescription
 *
 *  This file offers the following features:
 *    SDP similarity comparison
 *    Generating offer/answer using anonymous peer connection
 *    Test signalingstatechange event
 *    Test promise that never resolve
 */

const audioLineRegex = /\r\nm=audio.+\r\n/g;
const videoLineRegex = /\r\nm=video.+\r\n/g;
const applicationLineRegex = /\r\nm=application.+\r\n/g;

function countLine(sdp, regex) {
  const matches = sdp.match(regex);
  if(matches === null) {
    return 0;
  } else {
    return matches.length;
  }
}

function countAudioLine(sdp) {
  return countLine(sdp, audioLineRegex);
}

function countVideoLine(sdp) {
  return countLine(sdp, videoLineRegex);
}

function countApplicationLine(sdp) {
  return countLine(sdp, applicationLineRegex);
}

function similarMediaDescriptions(sdp1, sdp2) {
  if(sdp1 === sdp2) {
    return true;
  } else if(
    countAudioLine(sdp1) !== countAudioLine(sdp2) ||
    countVideoLine(sdp1) !== countVideoLine(sdp2) ||
    countApplicationLine(sdp1) !== countApplicationLine(sdp2))
  {
    return false;
  } else {
    return true;
  }
}

// Assert that given object is either an
// RTCSessionDescription or RTCSessionDescriptionInit
function assert_is_session_description(sessionDesc) {
  if(sessionDesc instanceof RTCSessionDescription) {
    return;
  }

  assert_not_equals(sessionDesc, undefined,
    'Expect session description to be defined');

  assert_true(typeof(sessionDesc) === 'object',
    'Expect sessionDescription to be either a RTCSessionDescription or an object');

  assert_true(typeof(sessionDesc.type) === 'string',
    'Expect sessionDescription.type to be a string');

  assert_true(typeof(sessionDesc.sdp) === 'string',
    'Expect sessionDescription.sdp to be a string');
}


// We can't do string comparison to the SDP content,
// because RTCPeerConnection may return SDP that is
// slightly modified or reordered from what is given
// to it due to ICE candidate events or serialization.
// Instead, we create SDP with different number of media
// lines, and if the SDP strings are not the same, we
// simply count the media description lines and if they
// are the same, we assume it is the same.
function isSimilarSessionDescription(sessionDesc1, sessionDesc2) {
  assert_is_session_description(sessionDesc1);
  assert_is_session_description(sessionDesc2);

  if(sessionDesc1.type !== sessionDesc2.type) {
    return false;
  } else {
    return similarMediaDescriptions(sessionDesc1.sdp, sessionDesc2.sdp);
  }
}

function assert_session_desc_similar(sessionDesc1, sessionDesc2) {
  assert_true(isSimilarSessionDescription(sessionDesc1, sessionDesc2),
    'Expect both session descriptions to have the same count of media lines');
}

function assert_session_desc_not_similar(sessionDesc1, sessionDesc2) {
  assert_false(isSimilarSessionDescription(sessionDesc1, sessionDesc2),
    'Expect both session descriptions to have different count of media lines');
}

async function generateDataChannelOffer(pc) {
  pc.createDataChannel('test');
  const offer = await pc.createOffer();
  assert_equals(countApplicationLine(offer.sdp), 1, 'Expect m=application line to be present in generated SDP');
  return offer;
}

async function generateAudioReceiveOnlyOffer(pc)
{
    try {
        pc.addTransceiver('audio', { direction: 'recvonly' });
        return pc.createOffer();
    } catch(e) {
        return pc.createOffer({ offerToReceiveAudio: true });
    }
}

async function generateVideoReceiveOnlyOffer(pc)
{
    try {
        pc.addTransceiver('video', { direction: 'recvonly' });
        return pc.createOffer();
    } catch(e) {
        return pc.createOffer({ offerToReceiveVideo: true });
    }
}

// Helper function to generate answer based on given offer using a freshly
// created RTCPeerConnection object
async function generateAnswer(offer) {
  const pc = new RTCPeerConnection();
  await pc.setRemoteDescription(offer);
  const answer = await pc.createAnswer();
  pc.close();
  return answer;
}

// Helper function to generate offer using a freshly
// created RTCPeerConnection object
async function generateOffer() {
  const pc = new RTCPeerConnection();
  const offer = await pc.createOffer();
  pc.close();
  return offer;
}

// Run a test function that return a promise that should
// never be resolved. For lack of better options,
// we wait for a time out and pass the test if the
// promise doesn't resolve within that time.
function test_never_resolve(testFunc, testName) {
  async_test(t => {
    testFunc(t)
    .then(
      t.step_func(result => {
        assert_unreached(`Pending promise should never be resolved. Instead it is fulfilled with: ${result}`);
      }),
      t.step_func(err => {
        assert_unreached(`Pending promise should never be resolved. Instead it is rejected with: ${err}`);
      }));

    t.step_timeout(t.step_func_done(), 100)
  }, testName);
}

// Helper function to exchange ice candidates between
// two local peer connections
function exchangeIceCandidates(pc1, pc2) {
  // private function
  function doExchange(localPc, remotePc) {
    localPc.addEventListener('icecandidate', event => {
      const { candidate } = event;

      // Guard against already closed peerconnection to
      // avoid unrelated exceptions.
      if (remotePc.signalingState !== 'closed') {
        remotePc.addIceCandidate(candidate);
      }
    });
  }

  doExchange(pc1, pc2);
  doExchange(pc2, pc1);
}

// Returns a promise that resolves when a |name| event is fired.
function waitUntilEvent(obj, name) {
  return new Promise(r => obj.addEventListener(name, r, {once: true}));
}

// Returns a promise that resolves when the |transport.state| is |state|
// This should work for RTCSctpTransport, RTCDtlsTransport and RTCIceTransport.
async function waitForState(transport, state) {
  while (transport.state != state) {
    await waitUntilEvent(transport, 'statechange');
  }
}

// Returns a promise that resolves when |pc.iceConnectionState| is 'connected'
// or 'completed'.
async function listenToIceConnected(pc) {
  await waitForIceStateChange(pc, ['connected', 'completed']);
}

// Returns a promise that resolves when |pc.iceConnectionState| is in one of the
// wanted states.
async function waitForIceStateChange(pc, wantedStates) {
  while (!wantedStates.includes(pc.iceConnectionState)) {
    await waitUntilEvent(pc, 'iceconnectionstatechange');
  }
}

// Returns a promise that resolves when |pc.connectionState| is 'connected'.
async function listenToConnected(pc) {
  while (pc.connectionState != 'connected') {
    await waitUntilEvent(pc, 'connectionstatechange');
  }
}

// Returns a promise that resolves when |pc.connectionState| is in one of the
// wanted states.
async function waitForConnectionStateChange(pc, wantedStates) {
  while (!wantedStates.includes(pc.connectionState)) {
    await waitUntilEvent(pc, 'connectionstatechange');
  }
}

function waitForConnectionStateChangeWithTimeout(t, pc, wantedStates, timeout) {
  return new Promise((resolve, reject) => {
    if (wantedStates.includes(pc.connectionState)) {
      resolve();
      return;
    }
    pc.addEventListener('connectionstatechange', () => {
      if (wantedStates.includes(pc.connectionState))
        resolve();
    });
    t.step_timeout(reject, timeout);
  });
}

async function waitForIceGatheringState(pc, wantedStates) {
  while (!wantedStates.includes(pc.iceGatheringState)) {
    await waitUntilEvent(pc, 'icegatheringstatechange');
  }
}

async function waitForTrackUnmuted(track) {
  if (track.muted === false) return true;
  return waitUntilEvent(track, 'unmute');
}

// Resolves when RTP packets have been received.
async function listenForSSRCs(t, receiver) {
  while (true) {
    const ssrcs = receiver.getSynchronizationSources();
    if (Array.isArray(ssrcs) && ssrcs.length > 0) {
      return ssrcs;
    }
    await new Promise(r => t.step_timeout(r, 0));
  }
}

// Helper function to create a pair of connected data channels.
// On success the promise resolves to an array with two data channels.
// It does the heavy lifting of performing signaling handshake,
// ICE candidate exchange, and waiting for data channel at two
// end points to open. Can do both negotiated and non-negotiated setup.
async function createDataChannelPair(t, options,
                                     pc1 = createPeerConnectionWithCleanup(t),
                                     pc2 = createPeerConnectionWithCleanup(t)) {
  let pair = [], bothOpen;
  try {
    if (options.negotiated) {
      pair = [pc1, pc2].map(pc => pc.createDataChannel('', options));
      bothOpen = Promise.all(pair.map(dc => new Promise((r, e) => {
        dc.onopen = r;
        dc.onerror = ({error}) => e(error);
      })));
    } else {
      pair = [pc1.createDataChannel('', options)];
      bothOpen = Promise.all([
        new Promise((r, e) => {
          pair[0].onopen = r;
          pair[0].onerror = ({error}) => e(error);
        }),
        new Promise((r, e) => pc2.ondatachannel = ({channel}) => {
          pair[1] = channel;
          channel.onopen = r;
          channel.onerror = ({error}) => e(error);
        })
      ]);
    }
    exchangeIceCandidates(pc1, pc2);
    await exchangeOfferAnswer(pc1, pc2);
    await bothOpen;
    return pair;
  } finally {
    for (const dc of pair) {
       dc.onopen = dc.onerror = null;
    }
  }
}

// Wait for RTP and RTCP stats to arrive
async function waitForRtpAndRtcpStats(pc) {
  // If remote stats are never reported, return after 5 seconds.
  const startTime = performance.now();
  while (true) {
    const report = await pc.getStats();
    const stats = [...report.values()].filter(({type}) => type.endsWith("bound-rtp"));
    // Each RTP and RTCP stat has a reference
    // to the matching stat in the other direction
    if (stats.length && stats.every(({localId, remoteId}) => localId || remoteId)) {
      break;
    }
    if (performance.now() > startTime + 5000) {
      break;
    }
  }
}

// Wait for a single message event and return
// a promise that resolve when the event fires
function awaitMessage(channel) {
  const once = true;
  return new Promise((resolve, reject) => {
    channel.addEventListener('message', ({data}) => resolve(data), {once});
    channel.addEventListener('error', reject, {once});
  });
}

// Helper to convert a blob to array buffer so that
// we can read the content
async function blobToArrayBuffer(blob) {
  const reader = new FileReader();
  reader.readAsArrayBuffer(blob);
  return new Promise((resolve, reject) => {
    reader.addEventListener('load', () => resolve(reader.result), {once: true});
    reader.addEventListener('error', () => reject(reader.error), {once: true});
  });
}

// Assert that two TypedArray or ArrayBuffer objects have the same byte values
function assert_equals_typed_array(array1, array2) {
  const [view1, view2] = [array1, array2].map((array) => {
    if (array instanceof ArrayBuffer) {
      return new DataView(array);
    } else {
      assert_true(array.buffer instanceof ArrayBuffer,
        'Expect buffer to be instance of ArrayBuffer');
      return new DataView(array.buffer, array.byteOffset, array.byteLength);
    }
  });

  assert_equals(view1.byteLength, view2.byteLength,
    'Expect both arrays to be of the same byte length');

  const byteLength = view1.byteLength;

  for (let i = 0; i < byteLength; ++i) {
    assert_equals(view1.getUint8(i), view2.getUint8(i),
      `Expect byte at buffer position ${i} to be equal`);
  }
}

// These media tracks will be continually updated with deterministic "noise" in
// order to ensure UAs do not cease transmission in response to apparent
// silence.
//
// > Many codecs and systems are capable of detecting "silence" and changing
// > their behavior in this case by doing things such as not transmitting any
// > media.
//
// Source: https://w3c.github.io/webrtc-pc/#offer-answer-options
const trackFactories = {
  // Share a single context between tests to avoid exceeding resource limits
  // without requiring explicit destruction.
  audioContext: null,

  /**
   * Given a set of requested media types, determine if the user agent is
   * capable of procedurally generating a suitable media stream.
   *
   * @param {object} requested
   * @param {boolean} [requested.audio] - flag indicating whether the desired
   *                                      stream should include an audio track
   * @param {boolean} [requested.video] - flag indicating whether the desired
   *                                      stream should include a video track
   *
   * @returns {boolean}
   */
  canCreate(requested) {
    const supported = {
      audio: !!window.AudioContext && !!window.MediaStreamAudioDestinationNode,
      video: !!HTMLCanvasElement.prototype.captureStream
    };

    return (!requested.audio || supported.audio) &&
      (!requested.video || supported.video);
  },

  audio() {
    const ctx = trackFactories.audioContext = trackFactories.audioContext ||
      new AudioContext();
    const oscillator = ctx.createOscillator();
    const dst = oscillator.connect(ctx.createMediaStreamDestination());
    oscillator.start();
    return dst.stream.getAudioTracks()[0];
  },

  video({width = 640, height = 480, signal} = {}) {
    const canvas = Object.assign(
      document.createElement("canvas"), {width, height}
    );
    const ctx = canvas.getContext('2d');
    const stream = canvas.captureStream();

    let count = 0;
    const interval = setInterval(() => {
      ctx.fillStyle = `rgb(${count%255}, ${count*count%255}, ${count%255})`;
      count += 1;
      ctx.fillRect(0, 0, width, height);
      // Add some bouncing boxes in contrast color to add a little more noise.
      const contrast = count + 128;
      ctx.fillStyle = `rgb(${contrast%255}, ${contrast*contrast%255}, ${contrast%255})`;
      const xpos = count % (width - 20);
      const ypos = count % (height - 20);
      ctx.fillRect(xpos, ypos, xpos + 20, ypos + 20);
      const xpos2 = (count + width / 2) % (width - 20);
      const ypos2 = (count + height / 2) % (height - 20);
      ctx.fillRect(xpos2, ypos2, xpos2 + 20, ypos2 + 20);
      // If signal is set (0-255), add a constant-color box of that luminance to
      // the video frame at coordinates 20 to 60 in both X and Y direction.
      // (big enough to avoid color bleed from surrounding video in some codecs,
      // for more stable tests).
      if (signal != undefined) {
        ctx.fillStyle = `rgb(${signal}, ${signal}, ${signal})`;
        ctx.fillRect(20, 20, 40, 40);
      }
    }, 100);

    if (document.body) {
      document.body.appendChild(canvas);
    } else {
      document.addEventListener('DOMContentLoaded', () => {
        document.body.appendChild(canvas);
      }, {once: true});
    }

    // Implement track.stop() for performance in some tests on some platforms
    const track = stream.getVideoTracks()[0];
    const nativeStop = track.stop;
    track.stop = function stop() {
      clearInterval(interval);
      nativeStop.apply(this);
      if (document.body && canvas.parentElement == document.body) {
        document.body.removeChild(canvas);
      }
    };
    return track;
  }
};

// Get the signal from a video element inserted by createNoiseStream
function getVideoSignal(v) {
  if (v.videoWidth < 60 || v.videoHeight < 60) {
    throw new Error('getVideoSignal: video too small for test');
  }
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 60;
  const context = canvas.getContext('2d');
  context.drawImage(v, 0, 0);
  // Extract pixel value at position 40, 40
  const pixel = context.getImageData(40, 40, 1, 1);
  // Use luma reconstruction to get back original value according to
  // ITU-R rec BT.709
  return (pixel.data[0] * 0.21 + pixel.data[1] * 0.72 + pixel.data[2] * 0.07);
}

async function detectSignal(t, v, value) {
  while (true) {
    const signal = getVideoSignal(v).toFixed();
    // allow off-by-two pixel error (observed in some implementations)
    if (value - 2 <= signal && signal <= value + 2) {
      return;
    }
    // We would like to wait for each new frame instead here,
    // but there seems to be no such callback.
    await new Promise(r => t.step_timeout(r, 100));
  }
}

// Generate a MediaStream bearing the specified tracks.
//
// @param {object} [caps]
// @param {boolean} [caps.audio] - flag indicating whether the generated stream
//                                 should include an audio track
// @param {boolean} [caps.video] - flag indicating whether the generated stream
//                                 should include a video track, or parameters for video
async function getNoiseStream(caps = {}) {
  if (!trackFactories.canCreate(caps)) {
    return navigator.mediaDevices.getUserMedia(caps);
  }
  const tracks = [];

  if (caps.audio) {
    tracks.push(trackFactories.audio());
  }

  if (caps.video) {
    tracks.push(trackFactories.video(caps.video));
  }

  return new MediaStream(tracks);
}

// Obtain a MediaStreamTrack of kind using procedurally-generated streams (and
// falling back to `getUserMedia` when the user agent cannot generate the
// requested streams).
// Return Promise of pair of track and associated mediaStream.
// Assumes that there is at least one available device
// to generate the track.
function getTrackFromUserMedia(kind) {
  return getNoiseStream({ [kind]: true })
  .then(mediaStream => {
    const [track] = mediaStream.getTracks();
    return [track, mediaStream];
  });
}

// Obtain |count| MediaStreamTracks of type |kind| and MediaStreams. The tracks
// do not belong to any stream and the streams are empty. Returns a Promise
// resolved with a pair of arrays [tracks, streams].
// Assumes there is at least one available device to generate the tracks and
// streams and that the getUserMedia() calls resolve.
function getUserMediaTracksAndStreams(count, type = 'audio') {
  let otherTracksPromise;
  if (count > 1)
    otherTracksPromise = getUserMediaTracksAndStreams(count - 1, type);
  else
    otherTracksPromise = Promise.resolve([[], []]);
  return otherTracksPromise.then(([tracks, streams]) => {
    return getTrackFromUserMedia(type)
    .then(([track, stream]) => {
      // Remove the default stream-track relationship.
      stream.removeTrack(track);
      tracks.push(track);
      streams.push(stream);
      return [tracks, streams];
    });
  });
}

// Performs an offer exchange caller -> callee.
async function exchangeOffer(caller, callee) {
  await caller.setLocalDescription(await caller.createOffer());
  await callee.setRemoteDescription(caller.localDescription);
}
// Performs an answer exchange caller -> callee.
async function exchangeAnswer(caller, callee) {
  // Note that caller's remote description must be set first; if not,
  // there's a chance that candidates from callee arrive at caller before
  // it has a remote description to apply them to.
  const answer = await callee.createAnswer();
  await caller.setRemoteDescription(answer);
  await callee.setLocalDescription(answer);
}
async function exchangeOfferAnswer(caller, callee) {
  await exchangeOffer(caller, callee);
  await exchangeAnswer(caller, callee);
}

// The returned promise is resolved with caller's ontrack event.
async function exchangeAnswerAndListenToOntrack(t, caller, callee) {
  const ontrackPromise = addEventListenerPromise(t, caller, 'track');
  await exchangeAnswer(caller, callee);
  return ontrackPromise;
}
// The returned promise is resolved with callee's ontrack event.
async function exchangeOfferAndListenToOntrack(t, caller, callee) {
  const ontrackPromise = addEventListenerPromise(t, callee, 'track');
  await exchangeOffer(caller, callee);
  return ontrackPromise;
}

// The resolver extends a |promise| that can be resolved or rejected using |resolve|
// or |reject|.
class Resolver extends Promise {
  constructor(executor) {
    let resolve, reject;
    super((resolve_, reject_) => {
      resolve = resolve_;
      reject = reject_;
      if (executor) {
        return executor(resolve_, reject_);
      }
    });

    this._done = false;
    this._resolve = resolve;
    this._reject = reject;
  }

  /**
   * Return whether the promise is done (resolved or rejected).
   */
  get done() {
    return this._done;
  }

  /**
   * Resolve the promise.
   */
  resolve(...args) {
    this._done = true;
    return this._resolve(...args);
  }

  /**
   * Reject the promise.
   */
  reject(...args) {
    this._done = true;
    return this._reject(...args);
  }
}

function addEventListenerPromise(t, obj, type, listener) {
  if (!listener) {
    return waitUntilEvent(obj, type);
  }
  return new Promise(r => obj.addEventListener(type,
                                               t.step_func(e => r(listener(e))),
                                               {once: true}));
}

function createPeerConnectionWithCleanup(t) {
  const pc = new RTCPeerConnection();
  t.add_cleanup(() => pc.close());
  return pc;
}

async function createTrackAndStreamWithCleanup(t, kind = 'audio') {
  let constraints = {};
  constraints[kind] = true;
  const stream = await getNoiseStream(constraints);
  const [track] = stream.getTracks();
  t.add_cleanup(() => track.stop());
  return [track, stream];
}

function findTransceiverForSender(pc, sender) {
  const transceivers = pc.getTransceivers();
  for (let i = 0; i < transceivers.length; ++i) {
    if (transceivers[i].sender == sender)
      return transceivers[i];
  }
  return null;
}

function preferCodec(transceiver, mimeType, sdpFmtpLine) {
  const {codecs} = RTCRtpReceiver.getCapabilities(transceiver.receiver.track.kind);
  // sdpFmtpLine is optional, pick the first partial match if not given.
  const selectedCodecIndex = codecs.findIndex(c => {
    return c.mimeType === mimeType && (c.sdpFmtpLine === sdpFmtpLine || !sdpFmtpLine);
  });
  const selectedCodec = codecs[selectedCodecIndex];
  codecs.slice(selectedCodecIndex, 1);
  codecs.unshift(selectedCodec);
  return transceiver.setCodecPreferences(codecs);
}

function findSendCodecCapability(mimeType, sdpFmtpLine) {
  return RTCRtpSender.getCapabilities(mimeType.split('/')[0])
    .codecs
    .filter(c => c.mimeType.localeCompare(name, undefined, { sensitivity: 'base' }) === 0
      && (c.sdpFmtpLine === sdpFmtpLine || !sdpFmtpLine))[0];
}

// Contains a set of values and will yell at you if you try to add a value twice.
class UniqueSet extends Set {
  constructor(items) {
    super();
    if (items !== undefined) {
      for (const item of items) {
        this.add(item);
      }
    }
  }

  add(value, message) {
    if (message === undefined) {
      message = `Value '${value}' needs to be unique but it is already in the set`;
    }
    assert_true(!this.has(value), message);
    super.add(value);
  }
}

const iceGatheringStateTransitions = async (pc, ...states) => {
  for (const state of states) {
    await new Promise((resolve, reject) => {
      pc.addEventListener('icegatheringstatechange', () => {
        if (pc.iceGatheringState == state) {
          resolve();
        } else {
          reject(`Unexpected gathering state: ${pc.iceGatheringState}, was expecting ${state}`);
        }
      }, {once: true});
    });
  }
  return states;
};

const initialOfferAnswerWithIceGatheringStateTransitions =
    async (pc1, pc2, offerOptions) => {
      await pc1.setLocalDescription(
        await pc1.createOffer(offerOptions));
      const pc1Transitions =
          iceGatheringStateTransitions(pc1, 'gathering', 'complete');
      await pc2.setRemoteDescription(pc1.localDescription);
      await pc2.setLocalDescription(await pc2.createAnswer());
      const pc2Transitions =
          iceGatheringStateTransitions(pc2, 'gathering', 'complete');
      await pc1.setRemoteDescription(pc2.localDescription);
      await pc1Transitions;
      await pc2Transitions;
    };

const expectNoMoreIceConnectionStateChanges = async (t, pc) => {
  pc.oniceconnectionstatechange =
      t.step_func(() => {
        assert_unreached(
            'Should not get an iceconnectionstatechange right now!');
      });
};

const expectNoMoreGatheringStateChanges = async (t, pc) => {
  pc.onicegatheringstatechange =
      t.step_func(() => {
        assert_unreached(
            'Should not get an icegatheringstatechange right now!');
      });
};

function gatheringStateReached(object, state) {
  if (object instanceof RTCIceTransport) {
    return new Promise(r =>
      object.addEventListener("gatheringstatechange", function listener() {
        if (object.gatheringState == state) {
          object.removeEventListener("gatheringstatechange", listener);
          r(state);
        }
      })
    );
  } else if (object instanceof RTCPeerConnection) {
    return new Promise(r =>
      object.addEventListener("icegatheringstatechange", function listener() {
        if (object.iceGatheringState == state) {
          object.removeEventListener("icegatheringstatechange", listener);
          r(state);
        }
      })
    );
  } else {
    throw "First parameter is neither an RTCIceTransport nor an RTCPeerConnection";
  }
}

function nextGatheringState(object) {
  if (object instanceof RTCIceTransport) {
    return new Promise(resolve =>
      object.addEventListener(
        "gatheringstatechange",
        () => resolve(object.gatheringState),
        { once: true }
      )
    );
  } else if (object instanceof RTCPeerConnection) {
    return new Promise(resolve =>
      object.addEventListener(
        "icegatheringstatechange",
        () => resolve(object.iceGatheringState),
        { once: true }
      )
    );
  } else {
    throw "First parameter is neither an RTCIceTransport nor an RTCPeerConnection";
  }
}

function emptyCandidate(pc) {
  return new Promise(r =>
    pc.addEventListener("icecandidate", function listener(e) {
      if (e.candidate && e.candidate.candidate == "") {
        pc.removeEventListener("icecandidate", listener);
        r(e);
      }
    })
  );
}

function nullCandidate(pc) {
  return new Promise(r =>
    pc.addEventListener("icecandidate", function listener(e) {
      if (!e.candidate) {
        pc.removeEventListener("icecandidate", listener);
        r(e);
      }
    })
  );
}

function connectionStateReached(object, state) {
  if (object instanceof RTCIceTransport || object instanceof RTCDtlsTransport) {
    return new Promise(resolve =>
      object.addEventListener("statechange", function listener() {
        if (object.state == state) {
          object.removeEventListener("statechange", listener);
          resolve(state);
        }
      })
    );
  } else if (object instanceof RTCPeerConnection) {
    return new Promise(resolve =>
      object.addEventListener("connectionstatechange", function listener() {
        if (object.connectionState == state) {
          object.removeEventListener("connectionstatechange", listener);
          resolve(state);
        }
      })
    );
  } else {
    throw "First parameter is neither an RTCIceTransport, an RTCDtlsTransport, nor an RTCPeerConnection";
  }
}

function nextConnectionState(object) {
  if (object instanceof RTCIceTransport || object instanceof RTCDtlsTransport) {
    return new Promise(resolve =>
      object.addEventListener("statechange", () => resolve(object.state), {
        once: true,
      })
    );
  } else if (object instanceof RTCPeerConnection) {
    return new Promise(resolve =>
      object.addEventListener(
        "connectionstatechange",
        () => resolve(object.connectionState),
        { once: true }
      )
    );
  } else {
    throw "First parameter is neither an RTCIceTransport, an RTCDtlsTransport, nor an RTCPeerConnection";
  }
}

function nextIceConnectionState(pc) {
  if (pc instanceof RTCPeerConnection) {
    return new Promise(resolve =>
      pc.addEventListener(
        "iceconnectionstatechange",
        () => resolve(pc.iceConnectionState),
        { once: true }
      )
    );
  } else {
    throw "First parameter is not an RTCPeerConnection";
  }
}

async function queueAWebrtcTask() {
  const pc = new RTCPeerConnection();
  pc.addTransceiver('audio');
  await new Promise(r => pc.onnegotiationneeded = r);
}

