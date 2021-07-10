// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  assert_equals(port.readable, null);

  await port.open({baudRate: 9600});
  const readable = port.readable;
  assert_true(readable instanceof ReadableStream);

  await port.close();
  assert_equals(port.readable, null);

  const reader = readable.getReader();
  const {value, done} = await reader.read();
  assert_true(done);
  assert_equals(value, undefined);
}, 'SerialPort.readable is set by open() and closes on port close');
