// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./RTCPeerConnection-helper.js
// META: timeout=long

'use strict';

// The following helper functions are called from RTCPeerConnection-helper.js:
//   generateAnswer()
//   getNoiseStream()

// Put the global IDL test objects under a parent object.
// This allows easier search for the test cases when
// viewing the web page
const idlTestObjects = {};

// Helper function to create RTCTrackEvent object
function initTrackEvent() {
  const pc = new RTCPeerConnection();
  const transceiver = pc.addTransceiver('audio');
  const { sender, receiver } = transceiver;
  const { track } = receiver;
  return new RTCTrackEvent('track', {
    receiver, track, transceiver
  });
}

// List of async test driver functions
const asyncInitTasks = [
  asyncInitCertificate,
  asyncInitTransports,
  asyncInitMediaStreamTrack,
];

// Asynchronously generate an RTCCertificate
function asyncInitCertificate() {
  return RTCPeerConnection.generateCertificate({
    name: 'RSASSA-PKCS1-v1_5',
    modulusLength: 2048,
    publicExponent: new Uint8Array([1, 0, 1]),
    hash: 'SHA-256'
  }).then(cert => {
    idlTestObjects.certificate = cert;
  });
}

// Asynchronously generate instances of
// RTCSctpTransport, RTCDtlsTransport,
// and RTCIceTransport
function asyncInitTransports() {
  const pc = new RTCPeerConnection();
  pc.createDataChannel('test');

  // setting answer description initializes pc.sctp
  return pc.createOffer()
  .then(offer =>
    pc.setLocalDescription(offer)
    .then(() => generateAnswer(offer)))
  .then(answer => pc.setRemoteDescription(answer))
  .then(() => {
    const sctpTransport = pc.sctp;
    assert_true(sctpTransport instanceof RTCSctpTransport,
      'Expect pc.sctp to be instance of RTCSctpTransport');
    idlTestObjects.sctpTransport = sctpTransport;

    const dtlsTransport = sctpTransport.transport;
    assert_true(dtlsTransport instanceof RTCDtlsTransport,
      'Expect sctpTransport.transport to be instance of RTCDtlsTransport');
    idlTestObjects.dtlsTransport = dtlsTransport;

    const iceTransport = dtlsTransport.iceTransport;
    assert_true(iceTransport instanceof RTCIceTransport,
      'Expect sctpTransport.transport to be instance of RTCDtlsTransport');
    idlTestObjects.iceTransport = iceTransport;
  });
}

// Asynchoronously generate MediaStreamTrack from getUserMedia
function asyncInitMediaStreamTrack() {
  return getNoiseStream({ audio: true })
    .then(mediaStream => {
      idlTestObjects.mediaStreamTrack = mediaStream.getTracks()[0];
    });
}

// Run all async test drivers, report and swallow any error
// thrown/rejected. Proper test for correct initialization
// of the objects are done in their respective test files.
function asyncInit() {
  return Promise.all(asyncInitTasks.map(
    task => {
      const t = async_test(`Test driver for ${task.name}`);
      let promise;
      t.step(() => {
        promise = task().then(
          t.step_func_done(),
          t.step_func(err =>
            assert_unreached(`Failed to run ${task.name}: ${err}`)));
      });
      return promise;
    }));
}

idl_test(
  ['webrtc'],
  ['WebIDL', 'mediacapture-streams', 'dom', 'html'],
  async idlArray => {
    idlArray.add_objects({
      RTCPeerConnection: [`new RTCPeerConnection()`],
      RTCSessionDescription: [`new RTCSessionDescription({ type: 'offer' })`],
      RTCIceCandidate: [`new RTCIceCandidate({ sdpMid: 1 })`],
      RTCDataChannel: [`new RTCPeerConnection().createDataChannel('')`],
      RTCRtpTransceiver: [`new RTCPeerConnection().addTransceiver('audio')`],
      RTCRtpSender: [`new RTCPeerConnection().addTransceiver('audio').sender`],
      RTCRtpReceiver: [`new RTCPeerConnection().addTransceiver('audio').receiver`],
      RTCPeerConnectionIceEvent: [`new RTCPeerConnectionIceEvent('ice')`],
      RTCPeerConnectionIceErrorEvent: [
        `new RTCPeerConnectionIceErrorEvent('ice-error', { port: 0, errorCode: 701 });`
      ],
      RTCTrackEvent: [`initTrackEvent()`],
      RTCErrorEvent: [`new RTCErrorEvent('error')`],
      RTCDataChannelEvent: [
        `new RTCDataChannelEvent('channel', {
          channel: new RTCPeerConnection().createDataChannel('')
        })`
      ],
      // Async initialized objects below
      RTCCertificate: ['idlTestObjects.certificate'],
      RTCSctpTransport: ['idlTestObjects.sctpTransport'],
      RTCDtlsTransport: ['idlTestObjects.dtlsTransport'],
      RTCIceTransport: ['idlTestObjects.iceTransport'],
      MediaStreamTrack: ['idlTestObjects.mediaStreamTrack'],
    });
    /*
      TODO
        RTCRtpContributingSource
        RTCRtpSynchronizationSource
        RTCDTMFSender
        RTCDTMFToneChangeEvent
        RTCIdentityProviderRegistrar
        RTCIdentityAssertion
    */

    await asyncInit();
  }
);
