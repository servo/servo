// META: global=window,worker
// META: script=/common/utils.js
// META: script=resources/webtransport-test-helpers.sub.js

// A RESET_STREAM from the peer ends the receive half of a bidirectional
// stream only; the send half stays writable until the peer sends
// STOP_SENDING.
promise_test(async t => {
  const id = token();
  const WT_CODE = 33;
  const HTTP_CODE = webtransport_code_to_http_code(WT_CODE);
  const wt = new WebTransport(webtransport_url(
      `reset-bidi-stream-then-read.py?token=${id}&code=${HTTP_CODE}`));
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const bidi = await wt.createBidirectionalStream();
  const writer = bidi.writable.getWriter();
  const reader = bidi.readable.getReader();

  await writer.write(new Uint8Array([1, 2, 3]));

  const e = await reader.closed.catch(e => e);
  assert_true(e instanceof WebTransportError, 'readable errored');
  assert_equals(e.streamErrorCode, WT_CODE, 'streamErrorCode');

  await writer.write(new Uint8Array([4, 5, 6, 7]));
  await writer.close();

  const info = await query_stream_close_info(id);
  assert_equals(info.source, 'FIN', 'send half closed with FIN');
  assert_equals(info.received, 7, 'bytes received after the reset');
}, 'RESET_STREAM from the server leaves the send half of a bidirectional stream writable');
