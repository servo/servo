// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expectedTargets = [navigator.serial];

  const actualTargets = [];
  function eventHandler(evt) {
    actualTargets.push(evt.currentTarget);

    if (evt.currentTarget == navigator.serial) {
      evt.stopPropagation();
    }
  }

  targets.forEach((target) => {
    target.addEventListener('foo', eventHandler, {capture: true});
    // stopPropagation() during capturing prevents bubbling.
    target.addEventListener('foo', eventHandler);

    t.add_cleanup(() => {
      target.removeEventListener('foo', eventHandler, {capture: true});
      target.removeEventListener('foo', eventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));

  assert_array_equals(actualTargets, expectedTargets, 'actualTargets');
}, 'stopPropagation() during capturing');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expectedTargets = [navigator.serial];

  const actualTargets = [];
  function eventHandler(evt) {
    actualTargets.push(evt.currentTarget);

    if (evt.currentTarget == navigator.serial) {
      evt.cancelBubble = true;
    }
  }

  targets.forEach((target) => {
    target.addEventListener('foo', eventHandler, {capture: true});
    // Setting cancelBubble during capturing prevents bubbling.
    target.addEventListener('foo', eventHandler);

    t.add_cleanup(() => {
      target.removeEventListener('foo', eventHandler, {capture: true});
      target.removeEventListener('foo', eventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));

  assert_array_equals(actualTargets, expectedTargets, 'actualTargets');
}, 'Set cancelBubble during capturing');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expectedTargets = [port];

  const actualTargets = [];
  function eventHandler(evt) {
    actualTargets.push(evt.currentTarget);

    if (evt.currentTarget == port) {
      evt.stopPropagation();
    }
  }

  targets.forEach((target) => {
    target.addEventListener('foo', eventHandler);

    t.add_cleanup(() => {
      target.removeEventListener('foo', eventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));

  assert_array_equals(actualTargets, expectedTargets, 'actualTargets');
}, 'stopPropagation() during bubbling');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expectedTargets = [port];

  const actualTargets = [];
  function eventHandler(evt) {
    actualTargets.push(evt.currentTarget);

    if (evt.currentTarget == port) {
      evt.cancelBubble = true;
    }
  }

  targets.forEach((target) => {
    target.addEventListener('foo', eventHandler);

    t.add_cleanup(() => {
      target.removeEventListener('foo', eventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));

  assert_array_equals(actualTargets, expectedTargets, 'actualTargets');
}, 'Set cancelBubble during bubbling');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expectedTargets = [
    navigator.serial,
    port,
    navigator.serial,
    port,
  ];
  const expectedTypes = [
    'foo',
    'bar',
    'bar',
    'foo',
  ];

  const actualTargets = [];
  const actualTypes = [];
  function eventHandler(evt) {
    actualTargets.push(evt.currentTarget);
    actualTypes.push(evt.type);

    if (evt.currentTarget == navigator.serial && evt.type == 'foo') {
      port.dispatchEvent(new CustomEvent('bar', {bubbles: true}));
    }
  }

  targets.forEach((target) => {
    target.addEventListener('foo', eventHandler, {capture: true});
    target.addEventListener('bar', eventHandler);

    t.add_cleanup(() => {
      target.removeEventListener('foo', eventHandler, {capture: true});
      target.removeEventListener('bar', eventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));

  assert_array_equals(actualTargets, expectedTargets, 'actualTargets');
  assert_array_equals(actualTypes, expectedTypes, 'actualTypes');
}, 'An event dispatched in an event handler is propagated before continuing');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);

  const targets = [navigator.serial, port];
  const expected = [
    'capturing Serial',
    'capturing SerialPort',
    'bubbling SerialPort',
    'bubbling Serial',
  ];

  const actual = [];
  targets.forEach((target) => {
    const bubblingEventHandler = () => {
      actual.push(`bubbling ${target.constructor.name}`);
    };
    target.addEventListener('foo', bubblingEventHandler);
    const capturingEventHandler = () => {
      actual.push(`capturing ${target.constructor.name}`);
    };
    target.addEventListener('foo', capturingEventHandler, {capture: true});

    t.add_cleanup(() => {
      target.removeEventListener('foo', bubblingEventHandler, {capture: true});
      target.removeEventListener('foo', capturingEventHandler);
    });
  });

  port.dispatchEvent(new CustomEvent('foo', {bubbles: true}));
  assert_array_equals(actual, expected);
}, 'Capturing and bubbling events delivered to listeners in the expected order');
