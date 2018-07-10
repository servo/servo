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

// Helper function to generate offer using a freshly created RTCPeerConnection
// object with any audio, video, data media lines present
function generateOffer(options={}) {
  const {
    audio = false,
    video = false,
    data = false,
    pc,
  } = options;

  if (data) {
    pc.createDataChannel('test');
  }

  const setup = {};

  if (audio) {
    setup.offerToReceiveAudio = true;
  }

  if (video) {
    setup.offerToReceiveVideo = true;
  }

  return pc.createOffer(setup).then(offer => {
    // Guard here to ensure that the generated offer really
    // contain the number of media lines we want
    const { sdp } = offer;

    if(audio) {
      assert_equals(countAudioLine(sdp), 1,
        'Expect m=audio line to be present in generated SDP');
    } else {
      assert_equals(countAudioLine(sdp), 0,
        'Expect m=audio line to be present in generated SDP');
    }

    if(video) {
      assert_equals(countVideoLine(sdp), 1,
        'Expect m=video line to be present in generated SDP');
    } else {
      assert_equals(countVideoLine(sdp), 0,
        'Expect m=video line to not present in generated SDP');
    }

    if(data) {
      assert_equals(countApplicationLine(sdp), 1,
        'Expect m=application line to be present in generated SDP');
    } else {
      assert_equals(countApplicationLine(sdp), 0,
        'Expect m=application line to not present in generated SDP');
    }

    return offer;
  });
}

// Helper function to generate answer based on given offer using a freshly
// created RTCPeerConnection object
function generateAnswer(offer) {
  const pc = new RTCPeerConnection();
  return pc.setRemoteDescription(offer)
  .then(() => pc.createAnswer());
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
// betweeen two local peer connections
function doSignalingHandshake(localPc, remotePc) {
  return localPc.createOffer()
  .then(offer => Promise.all([
    localPc.setLocalDescription(offer),
    remotePc.setRemoteDescription(offer)]))
  .then(() => remotePc.createAnswer())
  .then(answer => Promise.all([
    remotePc.setLocalDescription(answer),
    localPc.setRemoteDescription(answer)]))
}

// Helper function to create a pair of connected data channel.
// On success the promise resolves to an array with two data channels.
// It does the heavy lifting of performing signaling handshake,
// ICE candidate exchange, and waiting for data channel at two
// end points to open.
function createDataChannelPair(
  pc1=new RTCPeerConnection(),
  pc2=new RTCPeerConnection())
{
  const channel1 = pc1.createDataChannel('');

  exchangeIceCandidates(pc1, pc2);

  return new Promise((resolve, reject) => {
    let channel2;
    let opened1 = false;
    let opened2 = false;

    function onBothOpened() {
      resolve([channel1, channel2]);
    }

    function onOpen1() {
      opened1 = true;
      if(opened2) onBothOpened();
    }

    function onOpen2() {
      opened2 = true;
      if(opened1) onBothOpened();
    }

    function onDataChannel(event) {
      channel2 = event.channel;
      channel2.addEventListener('error', reject);
      const { readyState } = channel2;

      if(readyState === 'open') {
        onOpen2();
      } else if(readyState === 'connecting') {
        channel2.addEventListener('open', onOpen2);
      } else {
        reject(new Error(`Unexpected ready state ${readyState}`));
      }
    }

    channel1.addEventListener('open', onOpen1);
    channel1.addEventListener('error', reject);

    pc2.addEventListener('datachannel', onDataChannel);

    doSignalingHandshake(pc1, pc2);
  });
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

// Assert that two ArrayBuffer objects have the same byte values
function assert_equals_array_buffer(buffer1, buffer2) {
  assert_true(buffer1 instanceof ArrayBuffer,
    'Expect buffer to be instance of ArrayBuffer');

  assert_true(buffer2 instanceof ArrayBuffer,
    'Expect buffer to be instance of ArrayBuffer');

  assert_equals(buffer1.byteLength, buffer2.byteLength,
    'Expect both array buffers to be of the same byte length');

  const byteLength = buffer1.byteLength;
  const byteArray1 = new Uint8Array(buffer1);
  const byteArray2 = new Uint8Array(buffer2);

  for(let i=0; i<byteLength; i++) {
    assert_equals(byteArray1[i], byteArray2[i],
      `Expect byte at buffer position ${i} to be equal`);
  }
}

// Generate a MediaStreamTrack for testing use.
// We generate it by creating an anonymous RTCPeerConnection,
// call addTransceiver(), and use the remote track
// from RTCRtpReceiver. This track is supposed to
// receive media from a remote peer and be played locally.
// We use this approach instead of getUserMedia()
// to bypass the permission dialog and fake media devices,
// as well as being able to generate many unique tracks.
function generateMediaStreamTrack(kind) {
  const pc = new RTCPeerConnection();

  assert_idl_attribute(pc, 'addTransceiver',
    'Expect pc to have addTransceiver() method');

  const transceiver = pc.addTransceiver(kind);
  const { receiver } = transceiver;
  const { track } = receiver;

  assert_true(track instanceof MediaStreamTrack,
    'Expect receiver track to be instance of MediaStreamTrack');

  return track;
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
      audio: !!window.MediaStreamAudioDestinationNode,
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

async function exchangeOffer(caller, callee) {
  const offer = await caller.createOffer();
  await caller.setLocalDescription(offer);
  return callee.setRemoteDescription(offer);
}
async function exchangeAnswer(caller, callee) {
  const answer = await callee.createAnswer();
  await callee.setLocalDescription(answer);
  return caller.setRemoteDescription(answer);
}
async function exchangeOfferAnswer(caller, callee) {
  await exchangeOffer(caller, callee);
  return exchangeAnswer(caller, callee);
}


// The resolver has a |promise| that can be resolved or rejected using |resolve|
// or |reject|.
class Resolver {
  constructor() {
    let promiseResolve;
    let promiseReject;
    this.promise = new Promise(function(resolve, reject) {
      promiseResolve = resolve;
      promiseReject = reject;
    });
    this.resolve = promiseResolve;
    this.reject = promiseReject;
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
