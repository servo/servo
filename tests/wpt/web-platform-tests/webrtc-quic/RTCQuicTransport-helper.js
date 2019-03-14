'use strict';

// This file depends on RTCIceTransport-extension-helper.js which should be
// loaded from the main HTML file.
// The following helper functions are called from
// RTCIceTransport-extension-helper.js:
//   makeIceTransport
//   makeGatherAndStartTwoIceTransports

// Construct an RTCQuicTransport instance with the given RTCIceTransport
// instance and the given certificates. The RTCQuicTransport instance will be
// automatically cleaned up when the test finishes.
function makeQuicTransport(t, iceTransport) {
  const quicTransport = new RTCQuicTransport(iceTransport);
  t.add_cleanup(() => quicTransport.stop());
  return quicTransport;
}

// Construct an RTCQuicTransport instance with a new RTCIceTransport instance
// and a single, newly-generated certificate. The RTCQuicTransport and
// RTCIceTransport instances will be automatically cleaned up when the test
// finishes.
function makeStandaloneQuicTransport(t) {
  return makeQuicTransport(t, makeIceTransport(t));
}

// Construct two RTCQuicTransport instances and each call start() with the other
// transport's local parameters.
// Returns a 2-list:
//     [ server RTCQuicTransport,
//       client RTCQuicTransport ]
function makeAndStartTwoQuicTransports(t) {
  const [ localIceTransport, remoteIceTransport ] =
      makeGatherAndStartTwoIceTransports(t);
  const localQuicTransport =
      makeQuicTransport(t, localIceTransport);
  const remoteQuicTransport =
      makeQuicTransport(t, remoteIceTransport);
  const remote_key = remoteQuicTransport.getKey();
  localQuicTransport.listen(remote_key);
  remoteQuicTransport.connect();
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
