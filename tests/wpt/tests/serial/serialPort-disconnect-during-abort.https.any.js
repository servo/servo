// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});

  assert_true(port.writable instanceof WritableStream);
  const writer = port.writable.getWriter();

  await fakePort.readable();

  // Simulate disconnection error.
  fakePort.simulateDisconnectOnWrite();

  // Much larger than default bufferSize so that the write doesn't complete
  // synchronously.
  const data = new Uint8Array(1024);
  const writePromise = writer.write(data);

  // writer.abort is rejected with NetworkError due to simulated disconnection.
  await promise_rejects_dom(t, 'NetworkError', writer.abort('Aborting'));

  // writePromise is rejected with "Aborting" due to writer.abort.
  await promise_rejects_exactly(t, 'Aborting', writePromise);

  await port.close();
}, 'Disconnect error during abort works correctly');
