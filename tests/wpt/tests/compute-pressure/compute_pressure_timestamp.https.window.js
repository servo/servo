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

  let pressureChanges = [];
  const observer = new PressureObserver((changes) => {
    pressureChanges = pressureChanges.concat(changes);
  });
  t.add_cleanup(() => {
    observer.disconnect();
  });

  // When disconnect() is called, PressureRecord in [[LastRecordMap]] for cpu
  // should be cleared. The effect we observe in this test is the "has change
  // in data" algorithm passing with the same state twice.
  const states = ['critical', 'critical'];
  for (let i = 0; i < states.length; ++i) {
    await observer.observe('cpu', {sampleInterval: 500});
    await update_virtual_pressure_source('cpu', states[i]);
    await t.step_wait(() => pressureChanges.length == i + 1, 'foo');
    observer.disconnect();
  }

  assert_equals(pressureChanges.length, 2);
  assert_equals(pressureChanges[0].state, 'critical');
  assert_equals(pressureChanges[1].state, 'critical');
}, 'disconnect() should update [[LastRecordMap]]');

mark_as_done();
