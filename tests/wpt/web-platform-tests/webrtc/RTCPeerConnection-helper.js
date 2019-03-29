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

      // candidate may be null to indicate end of candidate gathering.
      // There is ongoing discussion on w3c/webrtc-pc#1213
      // that there should be an empty candidate string event
      // for end of candidate for each m= section.
      if(candidate && remotePc.signalingState !== 'closed') {
        remotePc.addIceCandidate(candidate);
      }
    });
  }

  doExchange(pc1, pc2);
  doExchange(pc2, pc1);
}

// Helper function for doing one round of offer/answer exchange
// between two local peer connections
async function doSignalingHandshake(localPc, remotePc, options={}) {
  let offer = await localPc.createOffer();
  // Modify offer if callback has been provided
  if (options.modifyOffer) {
    offer = await options.modifyOffer(offer);
  }

  // Apply offer
  await localPc.setLocalDescription(offer);
  await remotePc.setRemoteDescription(offer);

  let answer = await remotePc.createAnswer();
  // Modify answer if callback has been provided
  if (options.modifyAnswer) {
    answer = await options.modifyAnswer(answer);
  }

  // Apply answer
  await remotePc.setLocalDescription(answer);
  await localPc.setRemoteDescription(answer);
}

// Returns a promise that resolves when |pc.iceConnectionState| is 'connected'
// or 'completed'.
function listenToIceConnected(pc) {
  return new Promise((resolve) => {
    function isConnected(pc) {
      return pc.iceConnectionState == 'connected' ||
            pc.iceConnectionState == 'completed';
    }
    if (isConnected(pc)) {
      resolve();
      return;
    }
    pc.addEventListener('iceconnectionstatechange', () => {
      if (isConnected(pc))
        resolve();
    });
  });
}

// Returns a promise that resolves when |pc.connectionState| is 'connected'.
function listenToConnected(pc) {
  return new Promise((resolve) => {
    if (pc.connectionState == 'connected') {
      resolve();
      return;
    }
    pc.onconnectionstatechange = () => {
      if (pc.connectionState == 'connected')
        resolve();
    };
  });
}

// Resolves when RTP packets have been received.
function listenForSSRCs(t, receiver) {
  return new Promise((resolve) => {
    function listen() {
      const ssrcs = receiver.getSynchronizationSources();
      assert_true(ssrcs != undefined);
      if (ssrcs.length > 0) {
        resolve(ssrcs);
        return;
      }
      t.step_timeout(listen, 0);
    };
    listen();
  });
}

// Helper function to create a pair of connected data channel.
// On success the promise resolves to an array with two data channels.
// It does the heavy lifting of performing signaling handshake,
// ICE candidate exchange, and waiting for data channel at two
// end points to open.
function createDataChannelPair(
  pc1=new RTCPeerConnection(),
  pc2=new RTCPeerConnection(),
  options={})
{
  options = Object.assign({}, {
    channelLabel: '',
    channelOptions: undefined,
    doSignaling: true
  }, options);

  let channel1Options;
  let channel2Options = null;
  if (options.channelOptions instanceof Array) {
    [channel1Options, channel2Options] = options.channelOptions;
  } else {
    channel1Options = options.channelOptions;
  }

  const channel1 = pc1.createDataChannel(options.channelLabel, channel1Options);

  return new Promise((resolve, reject) => {
    let channel2;
    let opened1 = false;
    let opened2 = false;

    function cleanup() {
      channel1.removeEventListener('open', onOpen1);
      channel2.removeEventListener('open', onOpen2);
      channel1.removeEventListener('error', onError);
      channel2.removeEventListener('error', onError);
    }

    function onBothOpened() {
      cleanup();
      resolve([channel1, channel2]);
    }

    function onError(...args) {
      cleanup();
      reject(...args);
    }

    function onOpen1() {
      opened1 = true;
      if (opened2) {
        onBothOpened();
      }
    }

    function onOpen2() {
      opened2 = true;
      if (opened1) {
        onBothOpened();
      }
    }

    function onDataChannelPairFound() {
      channel2.addEventListener('error', onError, { once: true });
      const { readyState } = channel2;

      if (readyState === 'open') {
        onOpen2();
      } else if (readyState === 'connecting') {
        channel2.addEventListener('open', onOpen2, { once: true });
      } else {
        onError(new Error(`Unexpected ready state ${readyState}`));
      }
    }

    function onDataChannel(event) {
      channel2 = event.channel;
      onDataChannelPairFound();
    }

    channel1.addEventListener('open', onOpen1, { once: true });
    channel1.addEventListener('error', onError, { once: true });

    if (channel2Options !== null) {
      channel2 = pc2.createDataChannel(options.channelLabel, channel2Options);
      onDataChannelPairFound();
    } else {
      pc2.addEventListener('datachannel', onDataChannel);
    }

    if (options.doSignaling) {
      exchangeIceCandidates(pc1, pc2);
      doSignalingHandshake(pc1, pc2, options);
    }
  });
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
  return new Promise((resolve, reject) => {
    channel.addEventListener('message',
      event => resolve(event.data),
      { once: true });

    channel.addEventListener('error', reject, { once: true });
  });
}

// Helper to convert a blob to array buffer so that
// we can read the content
function blobToArrayBuffer(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.addEventListener('load', () => {
      resolve(reader.result);
    });

    reader.addEventListener('error', reject);

    reader.readAsArrayBuffer(blob);
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

  video({width = 640, height = 480} = {}) {
    const canvas = Object.assign(
      document.createElement("canvas"), {width, height}
    );
    const ctx = canvas.getContext('2d');
    const stream = canvas.captureStream();

    let count = 0;
    setInterval(() => {
      ctx.fillStyle = `rgb(${count%255}, ${count*count%255}, ${count%255})`;
      count += 1;

      ctx.fillRect(0, 0, width, height);
    }, 100);

    if (document.body) {
      document.body.appendChild(canvas);
    } else {
      document.addEventListener('DOMContentLoaded', () => {
        document.body.appendChild(canvas);
      });
    }

    return stream.getVideoTracks()[0];
  }
};

// Generate a MediaStream bearing the specified tracks.
//
// @param {object} [caps]
// @param {boolean} [caps.audio] - flag indicating whether the generated stream
//                                 should include an audio track
// @param {boolean} [caps.video] - flag indicating whether the generated stream
//                                 should include a video track
async function getNoiseStream(caps = {}) {
  if (!trackFactories.canCreate(caps)) {
    return navigator.mediaDevices.getUserMedia(caps);
  }
  const tracks = [];

  if (caps.audio) {
    tracks.push(trackFactories.audio());
  }

  if (caps.video) {
    tracks.push(trackFactories.video());
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
  const offer = await caller.createOffer();
  await caller.setLocalDescription(offer);
  return callee.setRemoteDescription(offer);
}
// Performs an answer exchange caller -> callee.
async function exchangeAnswer(caller, callee) {
  const answer = await callee.createAnswer();
  await callee.setLocalDescription(answer);
  return caller.setRemoteDescription(answer);
}
async function exchangeOfferAnswer(caller, callee) {
  await exchangeOffer(caller, callee);
  return exchangeAnswer(caller, callee);
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

function addEventListenerPromise(t, target, type, listener) {
  return new Promise((resolve, reject) => {
    target.addEventListener(type, t.step_func(e => {
      if (listener != undefined)
        e = listener(e);
      resolve(e);
    }));
  });
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
