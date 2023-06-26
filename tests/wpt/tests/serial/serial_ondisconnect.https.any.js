// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const eventWatcher =
      new EventWatcher(t, navigator.serial, ['connect', 'disconnect']);

  // Wait for getPorts() to resolve in order to ensure that the Mojo client
  // interface has been configured.
  let ports = await navigator.serial.getPorts();
  assert_equals(ports.length, 0);

  // Add ports one at a time so that we can map tokens to ports.
  const token1 = fake.addPort();
  const port1 = (await eventWatcher.wait_for(['connect'])).target;

  const token2 = fake.addPort();
  const port2 = (await eventWatcher.wait_for(['connect'])).target;

  fake.removePort(token2);
  const event1 = await eventWatcher.wait_for(['disconnect']);
  assert_true(event1 instanceof Event);
  assert_equals(event1.target, port2);

  ports = await navigator.serial.getPorts();
  assert_equals(ports.length, 1);
  assert_equals(ports[0], port1);

  fake.removePort(token1);
  const event2 = await eventWatcher.wait_for(['disconnect']);
  assert_true(event2 instanceof Event);
  assert_equals(event2.target, port1);

  ports = await navigator.serial.getPorts();
  assert_equals(ports.length, 0);
}, 'A "disconnect" event is fired when ports are added.');
