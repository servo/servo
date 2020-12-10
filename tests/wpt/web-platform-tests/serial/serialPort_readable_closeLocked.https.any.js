// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  assert_true(port.readable instanceof ReadableStream);

  const reader = port.readable.getReader();
  await promise_rejects_js(t, TypeError, port.close());

  reader.releaseLock();
  await port.close();
  assert_equals(port.readable, null);
}, 'Port cannot be closed while readable is locked');
