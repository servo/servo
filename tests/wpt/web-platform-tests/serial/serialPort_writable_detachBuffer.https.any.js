// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

function detachBuffer(buffer) {
  const channel = new MessageChannel();
  channel.port1.postMessage('', [buffer]);
}

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600, bufferSize: 64});

  const writer = port.writable.getWriter();
  const data = new Uint8Array(64);
  detachBuffer(data.buffer);

  // Writing a detached buffer is equivalent to writing an empty buffer so this
  // should trivially succeed.
  await writer.write(data);
  writer.releaseLock();

  await port.close();
}, 'Writing a detached buffer is safe');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size smaller than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  // Start writing a buffer much larger than bufferSize above so that it can't
  // all be transfered in a single operation.
  const writer = port.writable.getWriter();
  const data = new Uint8Array(1024);
  const promise = writer.write(data);
  writer.releaseLock();

  // Read half of the written data and then detach the buffer.
  await fakePort.readable();
  await fakePort.readWithLength(data.byteLength / 2);
  detachBuffer(data.buffer);

  // When the buffer is detached its length becomes zero and so the write should
  // succeed but it is undefined how much data was written before that happened.
  await promise;

  await port.close();
}, 'Detaching a buffer while writing is safe');
