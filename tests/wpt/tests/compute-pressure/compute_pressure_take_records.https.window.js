// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

'use strict';

pressure_test(async (t) => {
  const observer = new PressureObserver(
      t.unreached_func('This callback should not have been called.'));

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record before observe');
}, 'Calling takeRecords() before observe()');

pressure_test(async (t) => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  let observer;
  const changes = await new Promise((resolve, reject) => {
    observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
    update_virtual_pressure_source('cpu', 'critical').catch(reject);
  });
  assert_equals(changes[0].state, 'critical');

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record available');
}, 'takeRecords() returns empty record after callback invoke');

mark_as_done();
