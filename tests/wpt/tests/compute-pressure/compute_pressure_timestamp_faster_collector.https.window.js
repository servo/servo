// META: timeout=long
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

  const readings = ['nominal', 'fair', 'serious', 'critical'];

  const sampleInterval = 250;
  let pressureChanges = [];
  const observer = new PressureObserver((changes) => {
    pressureChanges = pressureChanges.concat(changes);
  });
  t.add_cleanup(() => observer.disconnect());
  observer.observe('cpu', {sampleInterval});

  for (let i = 0; i < 4;) {
    await update_virtual_pressure_source(
        'cpu', readings[i++ % readings.length]);
    await t.step_wait(
        () => pressureChanges.length === i,
        `At least ${i} readings have been delivered`);
  }

  assert_equals(pressureChanges.length, 4);
  assert_greater_than_equal(
      pressureChanges[1].time - pressureChanges[0].time, sampleInterval);
  assert_greater_than_equal(
      pressureChanges[2].time - pressureChanges[1].time, sampleInterval);
  assert_greater_than_equal(
      pressureChanges[3].time - pressureChanges[2].time, sampleInterval);
}, 'Faster collector: Timestamp difference between two changes should be higher or equal to the observer sample rate');

mark_as_done();
