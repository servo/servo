// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: timeout=long

// Tests the pull steps of a WebTransportReceiveStream:
// https://w3c.github.io/webtransport/#webtransportreceivestream-pull-bytes

// Returns a bidirectional stream whose readable end receives |data| echoed back
// by the server, followed by FIN.
async function echo_bidirectional_stream(wt, data) {
  const bidi_stream = await wt.createBidirectionalStream();
  const writer = bidi_stream.writable.getWriter();
  await writer.write(data);
  await writer.close();
  return bidi_stream;
}

function ascending_bytes(length) {
  const data = new Uint8Array(length);
  for (let i = 0; i < data.byteLength; ++i) {
    data[i] = i;
  }
  return data;
}

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const data = ascending_bytes(64);
  const bidi_stream = await echo_bidirectional_stream(wt, data);

  // Give the echoed bytes and the FIN a chance to be received before anything
  // reads them, so that the reads below are served from buffered bytes. The test
  // is valid either way: a read that arrives first waits for the bytes instead.
  await wait(100);

  const chunks = await read_stream(bidi_stream.readable);
  const received = new Uint8Array(chunks.reduce((length, chunk) => length + chunk.byteLength, 0));
  let offset = 0;
  for (const chunk of chunks) {
    received.set(chunk, offset);
    offset += chunk.byteLength;
  }

  // No bytes may be lost when the stream is closed while bytes are still waiting
  // to be given to the readable end.
  assert_array_equals(received, data);
  wt.close();
}, 'Bytes received before any read are given to later reads');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const data = ascending_bytes(64);
  const bidi_stream = await echo_bidirectional_stream(wt, data);
  await wait(100);

  // Read one byte at a time. Each view is smaller than what the server sent, so
  // the bytes that do not fit in it must be kept for the following reads.
  const reader = bidi_stream.readable.getReader({mode: 'byob'});
  for (let i = 0; i < data.byteLength; ++i) {
    const {value: view, done} = await reader.read(new Uint8Array(1));
    assert_false(done, `read ${i} should not be done`);
    assert_array_equals(view, data.subarray(i, i + 1), `read ${i}`);
  }

  // All the bytes have been read and the server ended its stream, so the next
  // read closes the readable end.
  const {value: view, done} = await reader.read(new Uint8Array(1));
  assert_true(done, 'the last read should be done');
  assert_equals(view.byteLength, 0, 'the last read should not fill the view');
  await reader.closed;
  reader.releaseLock();
  wt.close();
}, 'A BYOB read smaller than the received bytes keeps the rest for later reads');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const bidi_stream = await wt.createBidirectionalStream();

  // Read before anything is received. The read has to wait until either a byte is
  // received or the server ends its stream.
  const reader = bidi_stream.readable.getReader();
  const read = reader.read();

  const data = ascending_bytes(8);
  const writer = bidi_stream.writable.getWriter();
  await writer.write(data);

  const {value: chunk, done} = await read;
  assert_false(done, 'the pending read should not be done');
  assert_greater_than(chunk.byteLength, 0, 'the pending read should receive at least one byte');

  // Read the rest in case the bytes did not all arrive in a single chunk.
  let received = Array.from(chunk);
  await writer.close();
  while (received.length < data.byteLength) {
    const {value: chunk, done} = await reader.read();
    assert_false(done, 'the stream should not end before all its bytes are read');
    received = received.concat(Array.from(chunk));
  }
  assert_array_equals(received, Array.from(data));

  assert_true((await reader.read()).done, 'the read after FIN should be done');
  reader.releaseLock();
  wt.close();
}, 'A read waits for bytes that have not been received yet');
