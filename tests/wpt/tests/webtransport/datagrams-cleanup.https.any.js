// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const reader = wt.datagrams.readable.getReader();
  const read = reader.read();
  wt.close();

  const {done} = await read;
  assert_true(done, 'pending read should complete as done');
  await reader.closed;
  await wt.closed;
}, 'close() closes datagrams.readable');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'),
                              {datagramsReadableType: 'bytes'});
  await wt.ready;

  const reader = wt.datagrams.readable.getReader({mode: 'byob'});
  const read = reader.read(new Uint8Array(1024));
  wt.close();

  const {done} = await read;
  assert_true(done, 'pending read should complete as done');
  await reader.closed;
  await wt.closed;
}, 'close() closes a byte datagrams.readable');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));

  // Closing while connecting cleans up with an error rather than closeInfo.
  const reader = wt.datagrams.readable.getReader();
  const read = reader.read();
  wt.close();

  const error = await wt.closed.then(
      () => assert_unreached('closed should reject'), e => e);
  assert_true(error instanceof WebTransportError, 'closed error type');
  await promise_rejects_exactly(t, error, read,
                                'pending read should reject with the error');
  await promise_rejects_exactly(t, error, reader.closed);
}, 'a failed session errors datagrams.readable');
