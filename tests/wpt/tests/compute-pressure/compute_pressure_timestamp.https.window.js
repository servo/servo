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

pressure_test(async (t) => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const changes = await new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
    update_virtual_pressure_source('cpu', 'critical').catch(reject);
  });

  assert_less_than(changes[0].time, performance.now());
}, 'Timestamp from update should be tied to the global object\'s time origin');

pressure_test(async (t) => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const readings = ['critical', 'critical'];
  const syncObserver = new SyncPressureObserver(t);

  // When disconnect() is called, PressureRecord in [[LastRecordMap]] for cpu
  // should be cleared. The effect we observe in this test is the "has change
  // in data" algorithm passing with the same state twice.
  for (let i = 0; i < readings.length; ++i) {
    await syncObserver.observer().observe('cpu', {sampleInterval: 500});
    await update_virtual_pressure_source('cpu', readings[i]);
    await syncObserver.waitForUpdate();
    syncObserver.observer().disconnect();
  }

  const pressureChanges = syncObserver.changes();
  assert_equals(pressureChanges.length, readings.length);
  assert_equals(pressureChanges[0][0].state, 'critical');
  assert_equals(pressureChanges[1][0].state, 'critical');
}, 'disconnect() should update [[LastRecordMap]]');

mark_as_done();
