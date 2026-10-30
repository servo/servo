// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: timeout=long

async function create_web_transport(t) {
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});
  await wt.ready;
  return wt;
}

function assert_send_stats(stats, expected_bytes) {
  assert_equals(stats.bytesWritten, expected_bytes, 'bytesWritten');
  assert_equals(stats.bytesSent, expected_bytes, 'bytesSent');
  assert_equals(stats.bytesAcknowledged, expected_bytes, 'bytesAcknowledged');
}

async function wait_for_send_stats(stats_source, expected_bytes) {
  let stats;
  for (let i = 0; i < 100; ++i) {
    stats = await stats_source.getStats();
    if (stats.bytesSent === expected_bytes &&
        stats.bytesAcknowledged === expected_bytes) {
      break;
    }
    await wait(50);
  }
  assert_send_stats(stats, expected_bytes);
}

async function read_bytes(readable, expected) {
  const reader = readable.getReader({mode: 'byob'});
  const data = await readInto(reader, new ArrayBuffer(expected.byteLength));
  reader.releaseLock();
  assert_array_equals(new Uint8Array(data), expected);
}

promise_test(async t => {
  const wt = await create_web_transport(t);
  const group = wt.createSendGroup();
  assert_send_stats(await group.getStats(), 0);

  const bidirectional = await wt.createBidirectionalStream({sendGroup: group});
  const unidirectional =
      await wt.createUnidirectionalStream({sendGroup: group});
  assert_send_stats(await bidirectional.writable.getStats(), 0);
  assert_send_stats(await unidirectional.getStats(), 0);
  const bidirectional_data =
      new TextEncoder().encode('bidirectional stream stats');
  const unidirectional_data =
      new TextEncoder().encode('unidirectional stream stats');
  const expected_group_bytes =
      bidirectional_data.byteLength + unidirectional_data.byteLength;
  const bidirectional_writer = bidirectional.writable.getWriter();
  const unidirectional_writer = unidirectional.getWriter();

  await Promise.all([
    bidirectional_writer.write(bidirectional_data),
    unidirectional_writer.write(unidirectional_data),
  ]);

  const incoming_reader = wt.incomingUnidirectionalStreams.getReader();
  const {value: unidirectional_echo} = await incoming_reader.read();
  incoming_reader.releaseLock();
  await Promise.all([
    read_bytes(bidirectional.readable, bidirectional_data),
    read_bytes(unidirectional_echo, unidirectional_data),
  ]);

  await wait_for_send_stats(bidirectional.writable,
                            bidirectional_data.byteLength);
  await wait_for_send_stats(unidirectional, unidirectional_data.byteLength);
  await wait_for_send_stats(group, expected_group_bytes);

  await Promise.all([
    bidirectional_writer.close(),
    unidirectional_writer.close(),
  ]);
  await Promise.all([
    read_stream(bidirectional.readable),
    read_stream(unidirectional_echo),
  ]);

  assert_send_stats(await bidirectional.writable.getStats(),
                    bidirectional_data.byteLength);
  assert_send_stats(await unidirectional.getStats(),
                    unidirectional_data.byteLength);
  assert_send_stats(await group.getStats(), expected_group_bytes);

  wt.close();
  await wt.closed;

  assert_send_stats(await bidirectional.writable.getStats(),
                    bidirectional_data.byteLength);
  assert_send_stats(await unidirectional.getStats(),
                    unidirectional_data.byteLength);
  assert_send_stats(await group.getStats(), expected_group_bytes);
}, 'getStats() reports active and finalized send stream and group counters');
