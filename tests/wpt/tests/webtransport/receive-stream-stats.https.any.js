// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

const server_data = 'stream0';
const server_data_byte_length =
    new TextEncoder().encode(server_data).byteLength;

async function create_web_transport(t, handler) {
  const wt = new WebTransport(webtransport_url(handler));
  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});
  await wt.ready;
  return wt;
}

async function open_server_stream(t, type) {
  const wt = await create_web_transport(
      t, `server-create-multiple-streams.py?type=${type}&count=1`);
  const incoming = type === 'unidi' ? wt.incomingUnidirectionalStreams :
                                      wt.incomingBidirectionalStreams;
  const reader = incoming.getReader();
  const {value: stream} = await reader.read();
  reader.releaseLock();
  return type === 'unidi' ? stream : stream.readable;
}

async function assert_finalized_receive_stream_stats(stream) {
  assert_true(stream instanceof WebTransportReceiveStream);

  const data = await read_stream_as_string(stream);
  assert_equals(data, server_data);
  const stats = await stream.getStats();
  assert_equals(stats.bytesReceived, server_data_byte_length, 'bytesReceived');
  assert_equals(stats.bytesRead, server_data_byte_length, 'bytesRead');
}

const SERVER_STREAM_CASES = [
  {type: 'unidi', description: 'incoming unidirectional'},
  {type: 'bidi', description: 'incoming bidirectional'},
];

for (const {type, description} of SERVER_STREAM_CASES) {
  promise_test(async t => {
    const stream = await open_server_stream(t, type);
    await assert_finalized_receive_stream_stats(stream);
  }, `getStats() reports finalized ${description} stream counters`);
}

promise_test(async t => {
  const wt = await create_web_transport(t, 'echo.py');
  const stream = await wt.createBidirectionalStream();
  const writer = stream.writable.getWriter();
  await writer.write(new TextEncoder().encode(server_data));
  await writer.close();
  await assert_finalized_receive_stream_stats(stream.readable);
}, 'getStats() reports finalized client-initiated bidirectional stream counters');

promise_test(async t => {
  const wt = await create_web_transport(t, 'echo.py');
  const stream = await wt.createBidirectionalStream();
  const data = new Uint8Array(64);

  const writer = stream.writable.getWriter();
  await writer.write(data);
  await writer.close();

  const initial_stats = await stream.readable.getStats();
  assert_equals(initial_stats.bytesRead, 0, 'bytesRead before read');

  const reader = stream.readable.getReader({mode: 'byob'});
  const {value, done} = await reader.read(new Uint8Array(1));
  assert_false(done);
  assert_equals(value.byteLength, 1);

  const before_cancel = await stream.readable.getStats();
  assert_equals(before_cancel.bytesRead, 1, 'bytesRead before cancel');
  assert_greater_than_equal(before_cancel.bytesReceived,
                            before_cancel.bytesRead,
                            'bytesReceived before cancel');
  assert_less_than_equal(before_cancel.bytesReceived, data.byteLength,
                         'bytesReceived before cancel');

  await reader.cancel();
  reader.releaseLock();

  const after_cancel = await stream.readable.getStats();
  assert_equals(after_cancel.bytesRead, 1, 'bytesRead after cancel');
  assert_greater_than_equal(after_cancel.bytesReceived,
                            before_cancel.bytesReceived,
                            'bytesReceived after cancel');
  assert_less_than_equal(after_cancel.bytesReceived, data.byteLength,
                         'bytesReceived after cancel');

  wt.close();
  await wt.closed;
  const after_close = await stream.readable.getStats();
  assert_equals(after_close.bytesReceived, after_cancel.bytesReceived,
                'bytesReceived after close');
  assert_equals(after_close.bytesRead, after_cancel.bytesRead,
                'bytesRead after close');
}, 'getStats() preserves partial BYOB read counters after cancellation');
