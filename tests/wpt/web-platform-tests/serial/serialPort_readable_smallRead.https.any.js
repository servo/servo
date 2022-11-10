// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size larger than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const reader = port.readable.getReader();
  assert_true(reader instanceof ReadableStreamDefaultReader);

  await fakePort.writable();
  const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  fakePort.write(data);

  let {value, done} = await reader.read();
  assert_false(done);
  compareArrays(data, value);
  reader.releaseLock();

  await port.close();
}, 'Can read a small amount of data');
