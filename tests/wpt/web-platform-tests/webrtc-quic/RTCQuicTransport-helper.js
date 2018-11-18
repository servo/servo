'use strict';

// This file depends on RTCIceTransport-extension-helper.js which should be
// loaded from the main HTML file.
// The following helper functions are called from
// RTCIceTransport-extension-helper.js:
//   makeIceTransport
//   makeGatherAndStartTwoIceTransports

// Return a promise to generate an RTCCertificate with the given keygen
// algorithm or a default one if none provided.
function generateCertificate(keygenAlgorithm) {
  return RTCPeerConnection.generateCertificate({
    name: 'ECDSA',
    namedCurve: 'P-256',
    ...keygenAlgorithm,
  });
}

// Construct an RTCQuicTransport instance with the given RTCIceTransport
// instance and the given certificates. The RTCQuicTransport instance will be
// automatically cleaned up when the test finishes.
function makeQuicTransport(t, iceTransport, certificates) {
  const quicTransport = new RTCQuicTransport(iceTransport, certificates);
  t.add_cleanup(() => quicTransport.stop());
  return quicTransport;
}

// Construct an RTCQuicTransport instance with a new RTCIceTransport instance
// and a single, newly-generated certificate. The RTCQuicTransport and
// RTCIceTransport instances will be automatically cleaned up when the test
// finishes.
async function makeStandaloneQuicTransport(t) {
  const certificate = await generateCertificate();
  return makeQuicTransport(t, makeIceTransport(t), [ certificate ]);
}

// Construct two RTCQuicTransport instances and each call start() with the other
// transport's local parameters.
// Returns a 2-list:
//     [ server RTCQuicTransport,
//       client RTCQuicTransport ]
async function makeAndStartTwoQuicTransports(t) {
  const [ localCertificate, remoteCertificate ] =
      await Promise.all([ generateCertificate(), generateCertificate() ]);
  const [ localIceTransport, remoteIceTransport ] =
      makeGatherAndStartTwoIceTransports(t);
  const localQuicTransport =
      makeQuicTransport(t, localIceTransport, [ localCertificate ]);
  const remoteQuicTransport =
      makeQuicTransport(t, remoteIceTransport, [ remoteCertificate ]);
  localQuicTransport.start(remoteQuicTransport.getLocalParameters());
  remoteQuicTransport.start(localQuicTransport.getLocalParameters());
  return [ localQuicTransport, remoteQuicTransport ];
}

// Construct two RTCQuicTransport instances and wait for them to connect.
// Returns a 2-list:
//     [ server RTCQuicTransport,
//       client RTCQuicTransport ]
async function makeTwoConnectedQuicTransports(t) {
  // Returns a promise that resolves when the transport fires a 'statechange'
  // event to 'connected'.
  function waitForConnected(transport) {
    return new Promise((resolve, reject) => {
      const eventHandler = t.step_func(() => {
        assert_equals(transport.state, 'connected');
        transport.removeEventListener('statechange', eventHandler, false);
        resolve();
      });
      transport.addEventListener('statechange', eventHandler, false);
    });
  }
  const [ localQuicTransport, remoteQuicTransport ] =
      await makeAndStartTwoQuicTransports(t);
  await Promise.all([
    waitForConnected(localQuicTransport),
    waitForConnected(remoteQuicTransport),
  ]);
  return [ localQuicTransport, remoteQuicTransport ];
}
