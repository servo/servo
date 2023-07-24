// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await port.open({baudRate: 9600});
  return promise_rejects_dom(
      t, 'InvalidStateError', port.open({baudRate: 9600}));
}, 'A SerialPort cannot be opened if it is already open.');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const firstRequest = port.open({baudRate: 9600});
  await promise_rejects_dom(
      t, 'InvalidStateError', port.open({baudRate: 9600}));
  await firstRequest;
}, 'Simultaneous calls to open() are disallowed.');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await promise_rejects_js(t, TypeError, port.open({}));

  await Promise.all([-1, 0].map(
      baudRate => {
          return promise_rejects_js(t, TypeError, port.open({baudRate}))}));
}, 'Baud rate is required and must be greater than zero.');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await Promise.all([-1, 0, 6, 9].map(dataBits => {
    return promise_rejects_js(
        t, TypeError, port.open({baudRate: 9600, dataBits}));
  }));

  await[undefined, 7, 8].reduce(async (previousTest, dataBits) => {
    await previousTest;
    await port.open({baudRate: 9600, dataBits});
    await port.close();
  }, Promise.resolve());
}, 'Data bits must be 7 or 8');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await Promise.all([0, null, 'cats'].map(parity => {
    return promise_rejects_js(
        t, TypeError, port.open({baudRate: 9600, parity}),
        `Should reject parity option "${parity}"`);
  }));

  await[undefined, 'none', 'even', 'odd'].reduce(
      async (previousTest, parity) => {
        await previousTest;
        await port.open({baudRate: 9600, parity});
        await port.close();
      },
      Promise.resolve());
}, 'Parity must be "none", "even" or "odd"');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await Promise.all([-1, 0, 3, 4].map(stopBits => {
    return promise_rejects_js(
        t, TypeError, port.open({baudRate: 9600, stopBits}));
  }));

  await[undefined, 1, 2].reduce(async (previousTest, stopBits) => {
    await previousTest;
    await port.open({baudRate: 9600, stopBits});
    await port.close();
  }, Promise.resolve());
}, 'Stop bits must be 1 or 2');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  await promise_rejects_js(
      t, TypeError, port.open({baudRate: 9600, bufferSize: -1}));
  await promise_rejects_js(
      t, TypeError, port.open({baudRate: 9600, bufferSize: 0}));
}, 'Buffer size must be greater than zero.');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const bufferSize = 1 * 1024 * 1024 * 1024 /* 1 GiB */;
  return promise_rejects_js(
      t, TypeError, port.open({baudRate: 9600, bufferSize}));
}, 'Unreasonably large buffer sizes are rejected.');
