// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream.
  const bidi_stream = await wt.createBidirectionalStream();

  // Write a message to the writable end, and close it.
  const writer = bidi_stream.writable.getWriter();
  const encoder = new TextEncoder();
  await writer.write(encoder.encode('Hello World'));
  await writer.close();

  // Read the data on the readable end.
  const reply = await read_stream_as_string(bidi_stream.readable);

  // Check that the message from the readable end matches the writable end.
  assert_equals(reply, 'Hello World');
}, 'WebTransport server should be able to create and handle a bidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // The echo handler creates a bidirectional stream when a WebTransport session
  // is established. Accept the bidirectional stream.
  const stream_reader = wt.incomingBidirectionalStreams.getReader();
  const { value: bidi_stream } = await stream_reader.read();
  stream_reader.releaseLock();

  // Write a message to the writable end, and close it.
  const encoder = new TextEncoderStream();
  encoder.readable.pipeTo(bidi_stream.writable);
  const writer = encoder.writable.getWriter();
  await writer.write('Hello World');
  await writer.close();

  // Read the data on the readable end.
  const reply = await read_stream_as_string(bidi_stream.readable);

  // Check that the message from the readable end matches the writable end.
  assert_equals(reply, 'Hello World');
}, 'WebTransport server should be able to accept and handle a bidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a unidirectional stream.
  const writable = await wt.createUnidirectionalStream();

  // Write a message to the writable end, and close it.
  const encoder = new TextEncoderStream();
  encoder.readable.pipeTo(writable);
  const writer = encoder.writable.getWriter();
  await writer.write('Hello World');
  await writer.close();

  // The echo handler creates a new unidirectional stream to echo back data from
  // the server to client. Accept the unidirectional stream.
  const readable = wt.incomingUnidirectionalStreams;
  const stream_reader = readable.getReader();
  const { value: recv_stream } = await stream_reader.read();
  stream_reader.releaseLock();

  // Read the data on the readable end.
  const reply = await read_stream_as_string(recv_stream);

  // Make sure the message on the writable and readable ends of the streams
  // match.
  assert_equals(reply, 'Hello World');
}, 'WebTransport server should be able to create, accept, and handle a unidirectional stream');
