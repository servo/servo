// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['webtransport'],
  ['webidl', 'streams'],
  idl_array => {
    idl_array.add_objects({
      WebTransport: ['webTransport'],
      // TODO: The stream APIs below require a working connection to create.
      // BidirectionalStream
      // SendStream
      // ReceiveStream
    });
    self.webTransport = new WebTransport("https://example.com/");
    // `ready` and `closed` promises will be rejected due to connection error.
    // Catches them to avoid unhandled rejections.
    self.webTransport.ready.catch(() => {});
    self.webTransport.closed.catch(() => {});
  }
);
