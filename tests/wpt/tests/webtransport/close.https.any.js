// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(webtransport_url(`client-close.py?token=${id}`));
  add_completion_callback(() => wt.close());
  await wt.ready;

  wt.close();

  const close_info = await wt.closed;

  assert_equals(close_info.closeCode, 0, 'code');
  assert_equals(close_info.reason, '', 'reason');

  await wait(10);
  const data = await query(id);

  assert_own_property(data, 'session-close-info');
  const info = data['session-close-info']

  assert_false(info.abruptly, 'abruptly');
  assert_equals(info.close_info.code, 0, 'code');
  assert_equals(info.close_info.reason, '', 'reason');
}, 'close');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.close();
  try {
    await wt.closed;
  } catch(e) {
    await promise_rejects_exactly(t, e, wt.ready, 'ready promise should be rejected');
    assert_true(e instanceof WebTransportError);
    assert_equals(e.source, 'session', 'source');
    assert_equals(e.streamErrorCode, null, 'streamErrorCode');
  }
}, 'close without waiting for ready');

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(webtransport_url(`client-close.py?token=${id}`));
  add_completion_callback(() => wt.close());
  await wt.ready;

  wt.close({closeCode: 99, reason: 'reason X'});

  const close_info = await wt.closed;

  assert_equals(close_info.closeCode, 99, 'code');
  assert_equals(close_info.reason, 'reason X', 'reason');

  await wait(10);
  const data = await query(id);

  assert_own_property(data, 'session-close-info');
  const info = data['session-close-info']

  assert_false(info.abruptly, 'abruptly');
  assert_equals(info.close_info.code, 99, 'code');
  assert_equals(info.close_info.reason, 'reason X', 'reason');
}, 'close with code and reason');

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(webtransport_url(`client-close.py?token=${id}`));
  add_completion_callback(() => wt.close());
  await wt.ready;
  const reason = 'あいうえお'.repeat(1000);

  wt.close({closeCode: 11, reason});

  const close_info = await wt.closed;

  assert_equals(close_info.closeCode, 11, 'code');
  // `close_info.reason` should report the original, non-truncated reason as
  // step 9 of https://w3c.github.io/webtransport/#dom-webtransport-close
  // uses the original `closeInfo` to perform `Cleanup`.
  assert_equals(close_info.reason, reason, 'reason');

  await wait(10);
  const data = await query(id);

  assert_own_property(data, 'session-close-info');
  const info = data['session-close-info']

  // Server should have received truncated reason as step 6 of
  // https://w3c.github.io/webtransport/#dom-webtransport-close specifies.
  const expected_reason =
    new TextDecoder().decode(
      new TextEncoder().encode(reason).slice(0, 1024)).replaceAll('\ufffd', '');
  assert_false(info.abruptly, 'abruptly');
  assert_equals(info.close_info.code, 11, 'code');
  assert_equals(info.close_info.reason, expected_reason, 'reason');
}, 'close with code and long reason');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-close.py'));

  const close_info = await wt.closed;
  assert_equals(close_info.closeCode, 0, 'code');
  assert_equals(close_info.reason, '', 'reason');
}, 'server initiated closure without code and reason');

promise_test(async t => {
  const code = 32;
  const reason = 'abc';
  const wt = new WebTransport(
    webtransport_url(`server-close.py?code=${code}&reason=${reason}`));
  add_completion_callback(() => wt.close());

  const close_info = await wt.closed;
  assert_equals(close_info.closeCode, code, 'code');
  assert_equals(close_info.reason, reason, 'reason');
}, 'server initiated closure with code and reason');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-connection-close.py'));
  add_completion_callback(() => wt.close());

  const streams_reader = wt.incomingBidirectionalStreams.getReader();
  const { value: bidi } = await streams_reader.read();
  const writer = bidi.writable.getWriter();
  const reader = bidi.readable.getReader();
  try {
    writer.write(new Uint8Array([65]));
  } catch (e) {
  }

  // Sadly we cannot use promise_rejects_dom as the error constructor is
  // WebTransportError rather than DOMException.
  // We get a possible error, and then make sure wt.closed is rejected with it.
  const e = await wt.closed.catch(e => e);
  await promise_rejects_exactly(t, e, wt.closed, 'wt.closed');
  await promise_rejects_exactly(t, e, writer.closed, 'writer.closed');
  await promise_rejects_exactly(t, e, reader.closed, 'reader.closed');
  assert_true(e instanceof WebTransportError);
  assert_equals(e.source, 'session', 'source');
  assert_equals(e.streamErrorCode, null, 'streamErrorCode');
}, 'server initiated connection closure');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  const stream = await wt.createUnidirectionalStream();
  await wt.ready;
}, 'opening unidirectional stream before ready');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  const stream = await wt.createBidirectionalStream();
  await wt.ready;
}, 'opening bidirectional stream before ready');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-close.py'));
  await promise_rejects_dom(t, "InvalidStateError",
                            wt.createUnidirectionalStream());
}, 'server initiated closure while opening unidirectional stream before ready');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-close.py'));
  await promise_rejects_dom(t, "InvalidStateError",
                            wt.createBidirectionalStream());
}, 'server initiated closure while opening bidirectional stream before ready');

// Regression test for https://crbug.com/347710668.
promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-read-then-close.py'));
  add_completion_callback(() => wt.close());
  await wt.ready;

  const bidi_reader = wt.incomingBidirectionalStreams.getReader();
  const { value: bidi } = await bidi_reader.read();

  bidi.writable.getWriter().write(new TextEncoder().encode('some data'));
  const reader = bidi.readable.getReader();
  await reader.closed.catch(t.step_func(
      e => assert_true(e instanceof WebTransportError)));

  // The WebTransport session will already be closed.
  const {reason, closeCode} = await wt.closed;

  assert_equals(reason, '', 'reason should be default');
  assert_equals(closeCode, 0, 'closeCode should be default');
}, 'reading closed property after close should work');
