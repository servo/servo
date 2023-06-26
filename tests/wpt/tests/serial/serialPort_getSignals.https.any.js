// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await promise_rejects_dom(t, 'InvalidStateError', port.getSignals());
}, 'getSignals() rejects if the port is not open');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});

  let expectedSignals = {
    dataCarrierDetect: false,
    clearToSend: false,
    ringIndicator: false,
    dataSetReady: false
  };
  fakePort.simulateInputSignals(expectedSignals);
  let signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals);

  expectedSignals.dataCarrierDetect = true;
  fakePort.simulateInputSignals(expectedSignals);
  signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals, 'DCD set');

  expectedSignals.clearToSend = true;
  fakePort.simulateInputSignals(expectedSignals);
  signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals, 'CTS set');

  expectedSignals.ringIndicator = true;
  fakePort.simulateInputSignals(expectedSignals);
  signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals, 'RI set');

  expectedSignals.dataSetReady = true;
  fakePort.simulateInputSignals(expectedSignals);
  signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals, 'DSR set');
}, 'getSignals() returns the current state of input control signals');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600});

  fakePort.simulateInputSignalFailure(true);
  await promise_rejects_dom(t, 'NetworkError', port.getSignals());

  fakePort.simulateInputSignalFailure(false);
  const expectedSignals = {
    dataCarrierDetect: false,
    clearToSend: false,
    ringIndicator: false,
    dataSetReady: false
  };
  const signals = await port.getSignals();
  assert_object_equals(signals, expectedSignals);

  await port.close();
}, 'getSignals() rejects on failure');
