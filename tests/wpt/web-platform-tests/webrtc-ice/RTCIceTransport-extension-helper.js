'use strict';

// Construct an RTCIceTransport instance. The instance will automatically be
// cleaned up when the test finishes.
function makeIceTransport(t) {
  const iceTransport = new RTCIceTransport();
  t.add_cleanup(() => iceTransport.stop());
  return iceTransport;
}

// Construct two RTCIceTransport instances, configure them to exchange
// candidates, then gather() them.
// Returns a 2-list: [ RTCIceTransport, RTCIceTransport ]
function makeAndGatherTwoIceTransports(t) {
  const localTransport = makeIceTransport(t);
  const remoteTransport = makeIceTransport(t);
  localTransport.onicecandidate = e => {
    if (e.candidate) {
      remoteTransport.addRemoteCandidate(e.candidate);
    }
  };
  remoteTransport.onicecandidate = e => {
    if (e.candidate) {
      localTransport.addRemoteCandidate(e.candidate);
    }
  };
  localTransport.gather({});
  remoteTransport.gather({});
  return [ localTransport, remoteTransport ];
}

// Construct two RTCIceTransport instances, configure them to exchange
// candidates and parameters, then gather() and start() them.
// Returns a 2-list:
//     [ controlling RTCIceTransport,
//       controlled RTCIceTransport ]
function makeGatherAndStartTwoIceTransports(t) {
  const [ localTransport, remoteTransport ] = makeAndGatherTwoIceTransports(t);
  localTransport.start(remoteTransport.getLocalParameters(), 'controlling');
  remoteTransport.start(localTransport.getLocalParameters(), 'controlled');
  return [ localTransport, remoteTransport ];
}
