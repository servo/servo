// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

'use strict';

pressure_test(async (t) => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  let pressureChanges = [];
  const observer = new PressureObserver((changes) => {
    pressureChanges = pressureChanges.concat(changes);
  });
  t.add_cleanup(() => {
    observer.disconnect();
  });
  await observer.observe('cpu', {sampleInterval: 100});
  const input = ['critical', 'critical', 'nominal'];
  while (input.length != 0) {
    await update_virtual_pressure_source('cpu', input.shift());
    const currentChangesLength = pressureChanges.length;
    await Promise.race([
      new Promise((resolve) => {
        t.step_timeout(() => resolve('TIMEOUT'), 1000);
      }),
      t.step_wait(
          () => pressureChanges.length === currentChangesLength + 1,
          'Wait for new reading'),
    ]);
  }
  assert_equals(pressureChanges.length, 2);
  assert_equals(pressureChanges[0].state, 'critical');
  assert_equals(pressureChanges[1].state, 'nominal');
}, 'Changes that fail the "has change in data" test are discarded.');

mark_as_done();
