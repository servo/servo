// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('custom-response.py?:status=200'));
  await wt.ready;
}, 'WebTransport connection succeeds with status code 200');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('custom-response.py?:status=204'));
  await wt.ready;
}, 'WebTransport connection succeeds with status code 204');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('custom-response.py?:status=301'));
  await promise_rejects_js(t, TypeError, wt.ready, 'ready promise shoud be rejected');
  await promise_rejects_js(t, TypeError, wt.closed, 'closed promise should be rejected');
}, 'WebTransport connection fails with status code 301');

promise_test(async t => {
  const  wt = new WebTransport(webtransport_url('custom-response.py?:status=401'));
  await promise_rejects_js(t, TypeError, wt.ready, 'ready promise should be rejected');
  await promise_rejects_js(t, TypeError, wt.closed, 'closed promise should be rejected');
}, 'WebTransport connection fails with status code 401');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('custom-response.py?:status=404'));
  await promise_rejects_js(t, TypeError, wt.ready, 'ready promise should be rejected');
  await promise_rejects_js(t, TypeError, wt.closed, 'closed promise should be rejected');
}, 'WebTransport connection fails with status code 404');

promise_test(async t => {
  // Create WebTransport session.
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'));
  await wt.ready;

  // Read incoming unidirectional stream for echoed request headers.
  const streams = await wt.incomingUnidirectionalStreams;

  const stream_reader = streams.getReader();
  const { value: recv_stream } = await stream_reader.read();
  stream_reader.releaseLock();

  const request_headers = await read_stream_as_json(recv_stream);

  // Check the standard request headers.
  check_and_remove_standard_headers(request_headers);
}, 'Echo back request headers');

promise_test(async t => {
  // Create WebTransport session, and attach "Set-Cookie: foo=bar" to the response of
  // the handshake.
  const encodedSetCookie = encodeURIComponent('foo=bar');
  let wt = new WebTransport(webtransport_url('custom-response.py?set-cookie=' + encodedSetCookie));
  await wt.ready;

  wt = new WebTransport(webtransport_url('echo-request-headers.py'));
  await wt.ready;

  // Read incoming unidirectional stream for echoed request headers.
  const streams = await wt.incomingUnidirectionalStreams;

  const stream_reader = streams.getReader();
  const { value: recv_stream } = await stream_reader.read();
  stream_reader.releaseLock();

  const request_headers = await read_stream_as_json(recv_stream);

  // Check cookie header is not echoed back.
  check_and_remove_standard_headers(request_headers);
  assert_equals(request_headers['cookie'], undefined);
}, 'Cookie header is not echoed back');
