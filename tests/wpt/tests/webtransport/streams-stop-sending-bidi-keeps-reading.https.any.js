// META: global=window,worker
// META: script=/common/utils.js
// META: script=resources/webtransport-test-helpers.sub.js

// STOP_SENDING from the peer ends the send half of a bidirectional stream
// only; the receive half stays readable until the peer finishes sending.
promise_test(async t => {
  const WT_CODE = 21;
  const HTTP_CODE = webtransport_code_to_http_code(WT_CODE);
  const wt = new WebTransport(webtransport_url(
      `stop-sending-bidi-stream-then-send.py?code=${HTTP_CODE}`));
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const bidi = await wt.createBidirectionalStream();
  const writer = bidi.writable.getWriter();

  writer.write(new Uint8Array([1])).catch(() => {});

  const e = await writer.closed.catch(e => e);
  assert_true(e instanceof WebTransportError, 'writable errored');
  assert_equals(e.source, 'stream', 'source');
  assert_equals(e.streamErrorCode, WT_CODE, 'streamErrorCode');

  const received = [];
  for await (const chunk of bidi.readable) {
    received.push(...chunk);
  }
  assert_array_equals(received, [97, 98, 99], 'data after STOP_SENDING');
}, 'STOP_SENDING from the server leaves the receive half of a bidirectional stream readable');
