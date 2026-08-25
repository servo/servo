// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();
  assert_true(sendGroup instanceof WebTransportSendGroup,
              'createSendGroup should return a WebTransportSendGroup');
}, 'WebTransport.createSendGroup() should return a WebTransportSendGroup instance');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create a unidirectional stream with the send group
  const stream = await wt.createUnidirectionalStream({ sendGroup });
  assert_equals(stream.sendGroup, sendGroup,
    'sendGroup attribute should match input');

  // Write to the stream
  const writer = stream.getWriter();
  const encoder = new TextEncoder();
  await writer.write(encoder.encode('Hello from send group'));
  await writer.close();
}, 'WebTransport.createUnidirectionalStream() with sendGroup should create a working stream');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create a bidirectional stream with the send group
  const {readable, writable} = await wt.createBidirectionalStream({ sendGroup });
  assert_true(writable instanceof WebTransportSendStream,
              'writable should be a WebTransportSendStream');
  assert_true(readable instanceof WebTransportReceiveStream,
              'readable should be a WebTransportReceiveStream');

  // Write a message to the writable end, and close it
  const writer = writable.getWriter();
  const encoder = new TextEncoder();
  await writer.write(encoder.encode('Hello from bidirectional send group'));
  await writer.close();

  // Read the data on the readable end
  const reply = await read_stream_as_string(readable);

  // Check that the message from the readable end matches the writable end
  assert_equals(reply, 'Hello from bidirectional send group');
}, 'WebTransport.createBidirectionalStream() with sendGroup should create a working stream');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create a bidirectional stream with sendOrder and sendGroup
  const {readable, writable} = await wt.createBidirectionalStream({ sendGroup, sendOrder: 5 });
  assert_equals(writable.sendOrder, 5, 'sendOrder should be set');

  // Write a message
  const writer = writable.getWriter();
  const encoder = new TextEncoder();
  await writer.write(encoder.encode('Test sendOrder'));
  await writer.close();

  // Read the echo
  const reply = await read_stream_as_string(readable);
  assert_equals(reply, 'Test sendOrder');
}, 'WebTransport.createBidirectionalStream() with sendGroup should respect sendOrder option');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create a unidirectional stream with sendOrder and sendGroup
  const stream = await wt.createUnidirectionalStream({ sendGroup, sendOrder: 7 });
  assert_equals(stream.sendOrder, 7, 'sendOrder should be set');

  // Write to the stream
  const writer = stream.getWriter();
  const encoder = new TextEncoder();
  await writer.write(encoder.encode('Test uni sendOrder'));
  await writer.close();
}, 'WebTransport.createUnidirectionalStream() with sendGroup should respect sendOrder option');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  // Create multiple send groups
  const sendGroup1 = wt.createSendGroup();
  const sendGroup2 = wt.createSendGroup();

  assert_not_equals(sendGroup1, sendGroup2,
                    'Different calls should create different send groups');

  // Both should be able to create streams
  const stream1 = await wt.createUnidirectionalStream({ sendGroup: sendGroup1 });
  const stream2 = await wt.createUnidirectionalStream({ sendGroup: sendGroup2 });

  assert_true(stream1 instanceof WebTransportSendStream);
  assert_true(stream2 instanceof WebTransportSendStream);

  // Write to both
  const writer1 = stream1.getWriter();
  const writer2 = stream2.getWriter();
  const encoder = new TextEncoder();

  await writer1.write(encoder.encode('Group 1'));
  await writer2.write(encoder.encode('Group 2'));
  await writer1.close();
  await writer2.close();
}, 'Multiple send groups can be created and used independently');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create multiple streams with the same send group
  const stream1 = await wt.createUnidirectionalStream({ sendGroup });
  const stream2 = await wt.createUnidirectionalStream({ sendGroup });
  const {writable: stream3} = await wt.createBidirectionalStream({ sendGroup });

  // All should work
  const writer1 = stream1.getWriter();
  const writer2 = stream2.getWriter();
  const writer3 = stream3.getWriter();
  const encoder = new TextEncoder();

  await writer1.write(encoder.encode('Stream 1'));
  await writer2.write(encoder.encode('Stream 2'));
  await writer3.write(encoder.encode('Stream 3'));

  await writer1.close();
  await writer2.close();
  await writer3.close();
}, 'Multiple streams can be created with the same send group');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();

  // Create a stream with sendGroup and sendOrder options
  const {writable} = await wt.createBidirectionalStream({
    sendOrder: 3,
    sendGroup
  });

  assert_equals(writable.sendOrder, 3);

  const writer = writable.getWriter();
  await writer.close();
}, 'WebTransport streams can have both sendGroup and sendOrder options');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup1 = wt.createSendGroup();
  const sendGroup2 = wt.createSendGroup();

  const stream = await wt.createUnidirectionalStream({ sendGroup: sendGroup1 });
  assert_equals(stream.sendGroup, sendGroup1, 'sendGroup should be initial group');

  stream.sendGroup = sendGroup2;
  assert_equals(stream.sendGroup, sendGroup2, 'sendGroup should be updated to new group');

  stream.sendGroup = sendGroup1;
  assert_equals(stream.sendGroup, sendGroup1, 'sendGroup should be set back to first group');
}, 'sendGroup setter on existing stream updates the sendGroup');

promise_test(async t => {
  const wt1 = new WebTransport(webtransport_url('echo.py'));
  const wt2 = new WebTransport(webtransport_url('echo.py'));
  await Promise.all([wt1.ready, wt2.ready]);
  t.add_cleanup(() => wt1.close());
  t.add_cleanup(() => wt2.close());

  const sendGroupFromWt2 = wt2.createSendGroup();
  const stream = await wt1.createUnidirectionalStream();

  assert_throws_dom('InvalidStateError', () => {
    stream.sendGroup = sendGroupFromWt2;
  }, 'assigning sendGroup from a different transport should throw InvalidStateError');
}, 'sendGroup setter throws InvalidStateError when sendGroup belongs to a different transport');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();
  const stream = await wt.createUnidirectionalStream({ sendGroup });
  assert_equals(stream.sendGroup, sendGroup, 'sendGroup should be set initially');

  stream.sendGroup = null;
  assert_equals(stream.sendGroup, null, 'sendGroup should be null after clearing');
}, 'sendGroup can be set to null to clear the group');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  wt.close();
  await wt.closed.catch(() => {});

  assert_throws_dom('InvalidStateError', () => {
    wt.createSendGroup();
  }, 'createSendGroup on closed transport should throw InvalidStateError');
}, 'createSendGroup() throws InvalidStateError on a closed transport');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const sendGroup = wt.createSendGroup();
  const datagrams = wt.datagrams;
  const writable = datagrams.createWritable({ sendGroup, sendOrder: 3 });

  assert_true(writable instanceof WebTransportDatagramsWritable,
              'createWritable should return a WebTransportDatagramsWritable');
  assert_equals(writable.sendGroup, sendGroup, 'datagram writable sendGroup should be set');
  assert_equals(writable.sendOrder, 3, 'datagram writable sendOrder should be set');

  writable.sendGroup = null;
  assert_equals(writable.sendGroup, null, 'datagram writable sendGroup can be cleared');

  writable.sendOrder = 10;
  assert_equals(writable.sendOrder, 10, 'datagram writable sendOrder can be updated');
}, 'WebTransportDatagramsWritable supports sendGroup and sendOrder');
