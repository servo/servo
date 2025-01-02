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
}, 'WebTransport client should be able to create and handle a bidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));

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
}, 'WebTransport client should be able to create and handle a bidirectional stream without waiting for ready');

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
}, 'WebTransport client should be able to create, accept, and handle a unidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));

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
}, 'WebTransport client should be able to create, accept, and handle a unidirectional stream without waiting for ready');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // The echo handler creates a bidirectional stream when a WebTransport session
  // is established. Accept the bidirectional stream.
  const stream_reader = wt.incomingBidirectionalStreams.getReader();
  const {value: bidi_stream} = await stream_reader.read();
  stream_reader.releaseLock();

  // Write data to the writable end, and close it.
  const buffer_size = 256;
  const data = new Uint8Array(buffer_size);
  for (let i = 0; i < data.byteLength; ++i) {
    data[i] = i;
  }
  const writer = bidi_stream.writable.getWriter();
  writer.write(data);
  await writer.close();

  // Read the data on the readable end and check if it matches the writable end.
  const reader = bidi_stream.readable.getReader({mode: 'byob'});
  assert_true(reader instanceof ReadableStreamBYOBReader);
  const half_buffer_size = buffer_size / 2;
  for (let i = 0; i < 2; i++) {
    let buffer = new ArrayBuffer(half_buffer_size);
    buffer = await readInto(reader, buffer);
    assert_array_equals(
        new Uint8Array(buffer),
        data.subarray(half_buffer_size * i, half_buffer_size * (i + 1)))
  }
  reader.releaseLock();
}, 'Can read data from a bidirectional stream with BYOB reader');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a unidirectional stream.
  const writable = await wt.createUnidirectionalStream();

  // Write data to the writable end, and close it.
  const buffer_size = 256;
  const data = new Uint8Array(buffer_size);
  for (let i = 0; i < data.byteLength; ++i) {
    data[i] = i;
  }
  const writer = writable.getWriter();
  writer.write(data);
  await writer.close();

  // The echo handler creates a new unidirectional stream to echo back data from
  // the server to client. Accept the unidirectional stream.
  const readable = wt.incomingUnidirectionalStreams;
  const stream_reader = readable.getReader();
  const {value: recv_stream} = await stream_reader.read();
  stream_reader.releaseLock();

  // Read the data on the readable end and check if it matches the writable end.
  const reader = recv_stream.getReader({mode: 'byob'});
  assert_true(reader instanceof ReadableStreamBYOBReader);
  const half_buffer_size = buffer_size / 2;
  let buffer = new ArrayBuffer(half_buffer_size);
  for (let i = 0; i < 2; i++) {
    buffer = await readInto(reader, buffer);
    assert_array_equals(
        new Uint8Array(buffer),
        data.subarray(half_buffer_size * i, half_buffer_size * (i + 1)))
  }
  reader.releaseLock();
}, 'Can read data from a unidirectional stream with BYOB reader');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream.
  const bidi_stream = await wt.createBidirectionalStream();

  // Write a message to the writable end, and close it.
  const writer = bidi_stream.writable.getWriter();
  const bytes = new Uint8Array(16384);
  const [reply] = await Promise.all([
    read_stream(bidi_stream.readable),
    writer.write(bytes),
    writer.write(bytes),
    writer.write(bytes),
    writer.close()
  ]);
  let len = 0;
  for (chunk of reply) {
    len += chunk.length;
  }
  // Check that the message from the readable end matches the writable end.
  assert_equals(len, 3*bytes.length);
}, 'Transfer large chunks of data on a bidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a unidirectional stream.
  const uni_stream = await wt.createUnidirectionalStream();

  // Write a message to the writable end, and close it.
  const writer = uni_stream.getWriter();
  const bytes = new Uint8Array(16384);
  await Promise.all([
    writer.write(bytes),
    writer.write(bytes),
    writer.write(bytes),
    writer.close()
  ]);
  // XXX Update once chrome fixes https://crbug.com/929585
  // The echo handler creates a new unidirectional stream to echo back data from
  // the server to client. Accept the unidirectional stream.
  const readable = wt.incomingUnidirectionalStreams;
  const stream_reader = readable.getReader();
  const { value: recv_stream } = await stream_reader.read();
  stream_reader.releaseLock();

  // Read the data on the readable end.
  const reply = await read_stream(recv_stream);
  let len = 0;
  for (chunk of reply) {
    len += chunk.length;
  }
  // Check that the message from the readable end matches the writable end.
  assert_equals(len, 3*bytes.length);
}, 'Transfer large chunks of data on a unidirectional stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream.
  const bidi_stream = await wt.createBidirectionalStream();

  // Close the writable end with no data at all.
  const writer = bidi_stream.writable.getWriter();
  writer.close();

  // Read the data on the readable end.
  const chunks = await read_stream(bidi_stream.readable);
  assert_equals(chunks.length, 0);

  await bidi_stream.readable.closed;
}, 'Closing the stream with no data still resolves the read request');
