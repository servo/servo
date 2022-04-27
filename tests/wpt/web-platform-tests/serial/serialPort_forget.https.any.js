// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.forget();
  return promise_rejects_dom(
      t, 'NetworkError', port.open({baudRate: 9600}));
}, 'open() rejects after forget()');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});
  await port.forget();
  return promise_rejects_dom(t, 'InvalidStateError', port.close());
}, 'close() rejects after forget()');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});
  await port.forget();
  return promise_rejects_dom(t, 'InvalidStateError', port.setSignals());
}, 'setSignals() rejects after forget()');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});
  await port.forget();
  return promise_rejects_dom(t, 'InvalidStateError', port.getSignals());
}, 'getSignals() rejects after forget()');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  const portsBeforeForget = await navigator.serial.getPorts();
  assert_equals(portsBeforeForget.length, 1);
  assert_equals(portsBeforeForget[0], port);

  await port.forget();

  const portsAfterForget = await navigator.serial.getPorts();
  assert_equals(portsAfterForget.length, 0);
}, 'forget() removes the device from getPorts()');