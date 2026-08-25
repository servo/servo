// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: timeout=long

// Regression test for https://bugzilla.mozilla.org/show_bug.cgi?id=2046262

promise_test(async t => {
  const NUM_STREAMS = 5;
  const wt = new WebTransport(
    webtransport_url(
      `server-create-multiple-streams.py?type=unidi&count=${NUM_STREAMS}`));
  await wt.ready;

  const reader = wt.incomingUnidirectionalStreams.getReader();
  let received = [];
  for (let i = 0; i < NUM_STREAMS; i++) {
    const {value: stream} = await reader.read();
    const data = await read_stream_as_string(stream);
    received.push(data);
  }
  reader.releaseLock();

  received.sort();
  for (let i = 0; i < NUM_STREAMS; i++) {
    assert_equals(received[i], `stream${i}`);
  }
  wt.close();
}, 'Multiple server-initiated unidirectional streams should all be received');

promise_test(async t => {
  const NUM_STREAMS = 5;
  const wt = new WebTransport(
    webtransport_url(
      `server-create-multiple-streams.py?type=bidi&count=${NUM_STREAMS}`));
  await wt.ready;

  const reader = wt.incomingBidirectionalStreams.getReader();
  let received = [];
  for (let i = 0; i < NUM_STREAMS; i++) {
    const {value: stream} = await reader.read();
    const data = await read_stream_as_string(stream.readable);
    received.push(data);
  }
  reader.releaseLock();

  received.sort();
  for (let i = 0; i < NUM_STREAMS; i++) {
    assert_equals(received[i], `stream${i}`);
  }
  wt.close();
}, 'Multiple server-initiated bidirectional streams should all be received');
