// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  assert_equals(port.writable, null);

  await port.open({baudRate: 9600});
  const writable = port.writable;
  assert_true(writable instanceof WritableStream);

  await port.close();
  assert_equals(port.writable, null);

  const writer = writable.getWriter();
  const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  await promise_rejects_dom(t, 'InvalidStateError', writer.write(data));
}, 'open() and close() set and unset SerialPort.writable');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  assert_true(port.writable instanceof WritableStream);

  const writer = port.writable.getWriter();
  await promise_rejects_js(t, TypeError, port.close());

  writer.releaseLock();
  await port.close();
  assert_equals(port.writable, null);
}, 'Port cannot be closed while writable is locked');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  assert_true(port.writable instanceof WritableStream);

  const writer = port.writable.getWriter();
  const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  let writePromise = writer.write(data);
  writer.releaseLock();

  await fakePort.readable();
  let {value, done} = await fakePort.read();
  await writePromise;
  compareArrays(value, data);

  await port.close();
  assert_equals(port.writable, null);
}, 'Can write a small amount of data');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  const data = new Uint8Array(1024);  // Much larger than bufferSize above.
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  writer.write(data);
  writer.releaseLock();

  await fakePort.readable();
  const value = await fakePort.readWithLength(data.byteLength);
  compareArrays(data, value);

  await port.close();
}, 'Can write a large amount of data');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});

  const writable = port.writable;
  assert_true(writable instanceof WritableStream);
  let writer = writable.getWriter();

  await fakePort.readable();
  fakePort.simulateSystemErrorOnWrite();
  const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  await promise_rejects_dom(t, 'UnknownError', writer.write(data));

  assert_true(port.writable instanceof WritableStream);
  assert_not_equals(port.writable, writable);

  writer = port.writable.getWriter();
  let writePromise = writer.write(data);
  writer.releaseLock();
  await fakePort.readable();
  let {value, done} = await fakePort.read();
  await writePromise;
  compareArrays(value, data);

  await port.close();
  assert_equals(port.writable, null);
}, 'System error closes writable and replaces it with a new stream');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});

  assert_true(port.writable instanceof WritableStream);
  const writer = port.writable.getWriter();

  await fakePort.readable();
  fakePort.simulateDisconnectOnWrite();
  const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  await promise_rejects_dom(t, 'NetworkError', writer.write(data));
  assert_equals(port.writable, null);

  await port.close();
}, 'Disconnect error closes writable and sets it to null');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600, bufferSize: 64});
  const originalWritable = port.writable;
  assert_true(originalWritable instanceof WritableStream);

  let writer = originalWritable.getWriter();
  let data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  // The buffer size is large enough to allow this write to complete without
  // the data being read from the fake port.
  await writer.write(data);
  await writer.abort();

  assert_true(port.writable instanceof WritableStream);
  assert_true(port.writable !== originalWritable);
  writer = port.writable.getWriter();
  data = new Uint8Array([9, 10, 11, 12, 13, 14, 15, 16]);
  const writePromise = writer.write(data);
  writer.releaseLock();

  await fakePort.readable();
  const {value, done} = await fakePort.read();
  await writePromise;
  compareArrays(value, data);

  await port.close();
  assert_equals(port.writable, null);
}, 'abort() discards the write buffer');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  // Wait for microtasks to execute in order to ensure that the WritableStream
  // has been set up completely.
  await Promise.resolve();

  const data = new Uint8Array(1024);  // Much larger than bufferSize above.
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  const writePromise =
      promise_rejects_exactly(t, 'Aborting.', writer.write(data));

  await writer.abort('Aborting.');
  await writePromise;
  await port.close();
  assert_equals(port.writable, null);
}, 'abort() does not wait for the write buffer to be cleared');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  const data = new Uint8Array(1024);  // Much larger than bufferSize above.
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  const closed = (async () => {
    await promise_rejects_exactly(t, 'Aborting.', writer.write(data));
    writer.releaseLock();
    await port.close();
    assert_equals(port.writable, null);
  })();

  await writer.abort('Aborting.');
  await closed;
}, 'Can close while aborting');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  const data = new Uint8Array(1024);  // Much larger than bufferSize above.
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  writer.write(data);

  let readComplete = false;
  let writePromise = writer.close().then(() => {
    assert_true(readComplete);
  });

  await fakePort.readable();
  let readPromise = fakePort.readWithLength(data.byteLength).then(result => {
    readComplete = true;
    return result;
  });
  const value = await readPromise;
  compareArrays(data, value);
  await writePromise;

  await port.close();
}, 'close() waits for the write buffer to be cleared');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  // Wait for microtasks to execute in order to ensure that the WritableStream
  // has been set up completely.
  await Promise.resolve();

  const data = new Uint8Array(1024);  // Much larger than bufferSize above.
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  const writePromise =
      promise_rejects_exactly(t, 'Aborting.', writer.write(data));
  const closePromise = promise_rejects_exactly(t, 'Aborting.', writer.close());

  await writer.abort('Aborting.');
  await writePromise;
  await closePromise;
  await port.close();
  assert_equals(port.writable, null);
}, 'Can abort while closing');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  assert_true(port.writable instanceof WritableStream);

  const encoder = new TextEncoderStream();
  const streamClosed = encoder.readable.pipeTo(port.writable);
  const writer = encoder.writable.getWriter();
  const writePromise = writer.write('Hello world!');

  await fakePort.readable();
  const {value, done} = await fakePort.read();
  await writePromise;
  assert_equals('Hello world!', new TextDecoder().decode(value));
  await writer.close();
  await streamClosed;

  await port.close();
  assert_equals(port.writable, null);
}, 'Can pipe a stream to writable');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  assert_true(port.writable instanceof WritableStream);

  const transform = new TransformStream();
  const readable = transform.readable.pipeThrough(new TextEncoderStream())
                       .pipeThrough(new TransformStream())
                       .pipeThrough(new TransformStream())
                       .pipeThrough(new TransformStream());
  const streamClosed = readable.pipeTo(port.writable);
  const writer = transform.writable.getWriter();
  const writePromise = writer.write('Hello world!');

  await fakePort.readable();
  const {value, done} = await fakePort.read();
  await writePromise;
  assert_equals('Hello world!', new TextDecoder().decode(value));
  await writer.close();
  await streamClosed;

  await port.close();
  assert_equals(port.writable, null);
}, 'Stream closure is observable through a long chain of transformers');
