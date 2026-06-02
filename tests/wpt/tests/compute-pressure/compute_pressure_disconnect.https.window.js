// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

'use strict';

pressure_test(async t => {
  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });
  t.add_cleanup(() => observer.disconnect());
  observer.disconnect();
}, 'Calling disconnect() immediately should not crash');

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const observer1_changes = [];
  const observer1 = new PressureObserver(change => {
    observer1_changes.push(change);
  });
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  await observer1.observe('cpu');
  observer1.disconnect();

  const observer2_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });
  await update_virtual_pressure_source('cpu', 'critical');
  const observer2_changes = await observer2_promise;

  assert_equals(
      observer1_changes.length, 0,
      'disconnected observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].state, 'critical');
}, 'Stopped PressureObserver do not receive changes');

mark_as_done();
