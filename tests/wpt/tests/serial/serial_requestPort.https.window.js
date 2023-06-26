// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test((t, fake) => {
  return promise_rejects_dom(
      t, 'SecurityError', navigator.serial.requestPort());
}, 'requestPort() rejects without a user gesture');

serial_test(async (t, fake) => {
  await trustedClick();
  return promise_rejects_dom(
      t, 'NotFoundError', navigator.serial.requestPort());
}, 'requestPort() rejects if no port has been selected');

serial_test(async (t, fake) => {
  let token = fake.addPort();
  fake.setSelectedPort(token);

  await trustedClick();
  let port = await navigator.serial.requestPort();
  assert_true(port instanceof SerialPort);
}, 'requestPort() returns the selected port');

serial_test(async (t, fake) => {
  let token = fake.addPort();
  fake.setSelectedPort(token);

  await trustedClick();
  let firstPort = await navigator.serial.requestPort();
  assert_true(firstPort instanceof SerialPort);
  let secondPort = await navigator.serial.requestPort();
  assert_true(secondPort instanceof SerialPort);
  assert_true(firstPort === secondPort);
}, 'requestPort() returns the same port object every time');

serial_test(async (t, fake) => {
  let token = fake.addPort();
  fake.setSelectedPort(token);

  await trustedClick();
  let port = await navigator.serial.requestPort({filters: []});
  assert_true(port instanceof SerialPort);
}, 'An empty list of filters is valid');

serial_test(async (t, fake) => {
  let token = fake.addPort();
  fake.setSelectedPort(token);

  await trustedClick();
  return promise_rejects_js(t, TypeError, navigator.serial.requestPort({
    filters: [{}],
  }));
}, 'An empty filter is not valid');

serial_test(async (t, fake) => {
  let token = fake.addPort();
  fake.setSelectedPort(token);

  await trustedClick();
  return promise_rejects_js(t, TypeError, navigator.serial.requestPort({
    filters: [{usbProductId: 0x0001}],
  }));
}, 'requestPort() requires a USB vendor ID if a product ID specified');
