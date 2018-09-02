'use strict';

function makeQuicTransport(t, certificates) {
  const iceTransport = new RTCIceTransport();
  t.add_cleanup(() => iceTransport.stop());
  const quicTransport = new RTCQuicTransport(iceTransport, certificates);
  t.add_cleanup(() => quicTransport.stop());
  return quicTransport;
}

