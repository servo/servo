// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

// Write datagrams until the producer receives the AbortSignal.
async function write_datagrams(writer, signal) {
  const encoder = new TextEncoder();
  let counter = 0;
  const sentTokens = [];
  const aborted = new Promise((resolve) => {
    signal.addEventListener('abort', resolve);
  });
  while (true) {
    await Promise.race([writer.ready, aborted]);
    if (signal.aborted) {
      break;
    }
    var token = counter.toString();
    sentTokens.push(token);
    writer.write(encoder.encode(token));
    counter++;
  }
  return sentTokens;
}

// Write N datagrams without waiting, then wait for them
async function write_N_datagrams(writer, n) {
  const encoder = new TextEncoder();
  const sentTokens = [];
  const promises = [];
  while (sentTokens.length < n) {
    const token = sentTokens.length.toString();
    sentTokens.push(token);
    promises.push(writer.write(encoder.encode(token)));
  }
  await Promise.all(promises);
  return sentTokens;
}

// Read datagrams until the consumer has received enough i.e. N datagrams. Call
// abort() after reading.
async function read_datagrams(reader, controller, N) {
  const decoder = new TextDecoder();
  const receivedTokens = [];
  while (receivedTokens.length < N) {
    const { value: token, done } = await reader.read();
    assert_false(done);
    receivedTokens.push(decoder.decode(token));
  }
  controller.abort();
  return receivedTokens;
}

// Write numbers until the producer receives the AbortSignal.
async function write_numbers(writer, signal) {
  let counter = 0;
  const sentNumbers = [];
  const aborted =
    new Promise((resolve) => signal.addEventListener('abort', resolve));
  // Counter should be less than 256 because reader stores numbers in Uint8Array.
  while (counter < 256) {
    await Promise.race([writer.ready, aborted])
    if (signal.aborted) {
      break;
    }
    sentNumbers.push(counter);
    chunk = new Uint8Array(1);
    chunk[0] = counter;
    writer.write(chunk);
    counter++;
  }
  return sentNumbers;
}

// Write large datagrams of size 10 until the producer receives the AbortSignal.
async function write_large_datagrams(writer, signal) {
  const aborted = new Promise((resolve) => {
    signal.addEventListener('abort', resolve);
  });
  while (true) {
    await Promise.race([writer.ready, aborted]);
    if (signal.aborted) {
      break;
    }
    writer.write(new Uint8Array(10));
  }
}

// Read datagrams with BYOB reader until the consumer has received enough i.e. N
// datagrams. Call abort() after reading.
async function read_numbers_byob(reader, controller, N) {
  let buffer = new ArrayBuffer(N);
  buffer = await readInto(reader, buffer);
  controller.abort();
  return Array.from(new Uint8Array(buffer));
}

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  const controller = new AbortController();
  const signal = controller.signal;

  // Write and read datagrams.
  const N = 5;
  const [sentTokens, receivedTokens] = await Promise.all([
      write_datagrams(writer, signal),
      read_datagrams(reader, controller, N)
  ]);

  // Check receivedTokens is a subset of sentTokens.
  const subset = receivedTokens.every(token => sentTokens.includes(token));
  assert_true(subset);
}, 'Datagrams are echoed successfully');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader({ mode: 'byob' });

  const controller = new AbortController();
  const signal = controller.signal;

  // Write and read datagrams.
  // Numbers are less than 256, consider N to be a small number.
  const N = 5;
  const [sentNumbers, receiveNumbers] = await Promise.all([
    write_numbers(writer, signal),
    read_numbers_byob(reader, controller, N)
  ]);

  // No duplicated numbers received.
  assert_equals((new Set(receiveNumbers)).size, N);

  // Check receiveNumbers is a subset of sentNumbers.
  const subset = receiveNumbers.every(token => sentNumbers.includes(token));
  assert_true(subset);
}, 'Successfully reading datagrams with BYOB reader.');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader({ mode: 'byob' });

  const controller = new AbortController();
  const signal = controller.signal;

  // Write datagrams of size 10, but only 1 byte buffer is provided for BYOB
  // reader. To avoid splitting a datagram, stream will be errored.
  const buffer = new ArrayBuffer(1);
  const [error, _] = await Promise.all([
    reader.read(new Uint8Array(buffer)).catch(e => {
      controller.abort();
      return e;
    }),
    write_large_datagrams(writer, signal)
  ]);
  assert_equals(error.name, 'RangeError');
}, 'Reading datagrams with insufficient buffer should be rejected.');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo_datagram_length.py'));
  await wt.ready;

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  // Write and read max-size datagram.
  const maxDatagramSize = wt.datagrams.maxDatagramSize;
  await writer.write(new Uint8Array(maxDatagramSize));

  // the server should echo the datagram length encoded in JSON
  const { value: token, done } = await reader.read();
  assert_false(done);

  const decoder = new TextDecoder();
  const datagramStr = decoder.decode(token);
  const jsonObject = JSON.parse(datagramStr);
  assert_equals(jsonObject['length'], maxDatagramSize);
}, 'Transfer max-size datagram');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  // Write and read max-size datagram.
  await writer.write(new Uint8Array(wt.datagrams.maxDatagramSize+1));
  // This should resolve with no datagram sent, which is hard to test for.
  // Wait for incoming datagrams to arrive, and if they do, fail.
  const result = await Promise.race([reader.read(), wait(500)]);
  assert_equals(result, undefined);
}, 'Fail to transfer max-size+1 datagram');

promise_test(async t => {
  // Make a WebTransport connection, but session is not necessarily established.
  const wt = new WebTransport(webtransport_url('echo.py'));

  const writer = wt.datagrams.writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  const controller = new AbortController();
  const signal = controller.signal;

  // Write and read datagrams.
  const N = 5;
  wt.datagrams.outgoingHighWaterMark = N;
  const [sentTokens, receivedTokens] = await Promise.all([
      write_N_datagrams(writer, N),
      read_datagrams(reader, controller, N)
  ]);

  // Check receivedTokens is a subset of sentTokens.
  const subset = receivedTokens.every(token => sentTokens.includes(token));
  assert_true(subset);

  // Make sure WebTransport session is established.
  await wt.ready;
}, 'Sending and receiving datagrams is ready to use before session is established');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const N = 5;
  wt.datagrams.outgoingHighWaterMark = N;

  const writer = wt.datagrams.writable.getWriter();
  const encoder = new TextEncoder();

  // Write N-1 datagrams.
  let counter;
  for (counter = 0; counter < N-1; counter++) {
    var datagram = counter.toString();
    let resolved = false;
    writer.write(encoder.encode(datagram));

    // Check writer.ready resolves immediately.
    writer.ready.then(() => resolved = true);
    // TODO(nidhijaju): The number of `await Promise.resolve()` calls is
    // implementation dependent, so we should not have this as the final
    // solution.
    for (let i = 0; i < 10; i++) {
      await Promise.resolve();
    }
    assert_true(resolved);
  }

  // Write one more datagram.
  resolved = false;
  const last_datagram = counter.toString();
  writer.write(encoder.encode(last_datagram));

  // Check writer.ready does not resolve immediately.
  writer.ready.then(() => resolved = true);
  for (let i = 0; i < 10; i++) {
    await Promise.resolve();
  }
  assert_false(resolved);

  // Make sure writer.ready is resolved eventually.
  await writer.ready;
}, 'Datagram\'s outgoingHighWaterMark correctly regulates written datagrams');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const N = 5;
  wt.datagrams.incomingHighWaterMark = N;

  const writer = wt.datagrams.writable.getWriter();
  const encoder = new TextEncoder();

  // Write 10*N datagrams.
  let counter;
  for (counter = 0; counter < 10*N; counter++) {
    var datagram = counter.toString();
    writer.write(encoder.encode(datagram));
    await writer.ready;
  }

  // Wait for incoming datagrams to arrive.
  wait(500);

  const reader = wt.datagrams.readable.getReader();

  // Read all of the immediately available datagrams.
  let receivedDatagrams = 0;
  while (true) {
    let resolved = false;
    reader.read().then(() => resolved = true);
    // TODO(nidhijaju): Find a better solution instead of just having numerous
    // `await Promise.resolve()` calls.
    for (let i = 0; i < 10; i++) {
      await Promise.resolve();
    }
    if (!resolved) {
      break;
    }
    receivedDatagrams++;
  }

  // Check that the receivedDatagrams is less than or equal to the
  // incomingHighWaterMark.
  assert_less_than_equal(receivedDatagrams, N);
}, 'Datagrams read is less than or equal to the incomingHighWaterMark');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  assert_equals(wt.datagrams.incomingMaxAge, Infinity);
  assert_equals(wt.datagrams.outgoingMaxAge, Infinity);

  wt.datagrams.incomingMaxAge = 5;
  assert_equals(wt.datagrams.incomingMaxAge, 5);
  wt.datagrams.outgoingMaxAge = 5;
  assert_equals(wt.datagrams.outgoingMaxAge, 5);

  assert_throws_js(RangeError, () => { wt.datagrams.incomingMaxAge = -1; });
  assert_throws_js(RangeError, () => { wt.datagrams.outgoingMaxAge = -1; });
  assert_throws_js(RangeError, () => { wt.datagrams.incomingMaxAge = NaN; });
  assert_throws_js(RangeError, () => { wt.datagrams.outgoingMaxAge = NaN; });

  wt.datagrams.incomingMaxAge = 0;
  assert_equals(wt.datagrams.incomingMaxAge, Infinity);
  wt.datagrams.outgoingMaxAge = 0;
  assert_equals(wt.datagrams.outgoingMaxAge, Infinity);
}, 'Datagram MaxAge getters/setters work correctly');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Initial values are implementation-defined
  assert_greater_than_equal(wt.datagrams.incomingHighWaterMark, 1);
  assert_greater_than_equal(wt.datagrams.outgoingHighWaterMark, 1);

  wt.datagrams.incomingHighWaterMark = 5;
  assert_equals(wt.datagrams.incomingHighWaterMark, 5);
  wt.datagrams.outgoingHighWaterMark = 5;
  assert_equals(wt.datagrams.outgoingHighWaterMark, 5);

  assert_throws_js(RangeError, () => { wt.datagrams.incomingHighWaterMark = -1; });
  assert_throws_js(RangeError, () => { wt.datagrams.outgoingHighWaterMark = -1; });
  assert_throws_js(RangeError, () => { wt.datagrams.incomingHighWaterMark = NaN; });
  assert_throws_js(RangeError, () => { wt.datagrams.outgoingHighWaterMark = NaN; });

  wt.datagrams.incomingHighWaterMark = 0.5;
  assert_equals(wt.datagrams.incomingHighWaterMark, 1);
  wt.datagrams.outgoingHighWaterMark = 0.5;
  assert_equals(wt.datagrams.outgoingHighWaterMark, 1);
  wt.datagrams.incomingHighWaterMark = 0;
  assert_equals(wt.datagrams.incomingHighWaterMark, 1);
  wt.datagrams.outgoingHighWaterMark = 0;
  assert_equals(wt.datagrams.outgoingHighWaterMark, 1);
}, 'Datagram HighWaterMark getters/setters work correctly');
