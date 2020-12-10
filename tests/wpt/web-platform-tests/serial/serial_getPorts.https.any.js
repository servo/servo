// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  fake.addPort();
  fake.addPort();

  let ports = await navigator.serial.getPorts();
  assert_equals(ports.length, 2);
  assert_true(ports[0] instanceof SerialPort);
  assert_true(ports[1] instanceof SerialPort);
}, 'getPorts() returns the set of configured fake ports');

serial_test(async (t, fake) => {
  fake.addPort();

  let portsFirst = await navigator.serial.getPorts();
  assert_equals(portsFirst.length, 1, 'first call returns one port');
  assert_true(portsFirst[0] instanceof SerialPort);
  let portsSecond = await navigator.serial.getPorts();
  assert_equals(portsSecond.length, 1, 'second call returns one port');
  assert_true(portsSecond[0] instanceof SerialPort);
  assert_true(portsFirst[0] === portsSecond[0]);
}, 'getPorts() returns the same port objects every time');
