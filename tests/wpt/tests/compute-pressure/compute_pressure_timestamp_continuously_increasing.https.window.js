// META: timeout=long
// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js
// META: script=./resources/sync-pressure-observer.js

'use strict';

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const readings = ['nominal', 'fair', 'serious', 'critical'];

  const syncObserver = new SyncPressureObserver(t);
  await syncObserver.observer().observe('cpu', {sampleInterval: 250});

  for (let i = 0; i < readings.length; ++i) {
    await update_virtual_pressure_source('cpu', readings[i]);
    await syncObserver.waitForUpdate();
  }

  const pressureChanges = syncObserver.changes();
  assert_equals(pressureChanges.length, readings.length);

  assert_greater_than(pressureChanges[1][0].time, pressureChanges[0][0].time);
  assert_greater_than(pressureChanges[2][0].time, pressureChanges[1][0].time);
  assert_greater_than(pressureChanges[3][0].time, pressureChanges[2][0].time);
}, 'Timestamp difference between two changes should be continuously increasing');

mark_as_done();
