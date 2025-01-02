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

  const syncObserver = new SyncPressureObserver(t);
  await syncObserver.observer().observe('cpu', {sampleInterval: 100});

  await update_virtual_pressure_source('cpu', 'critical');
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[0][0].state, 'critical');

  await update_virtual_pressure_source('cpu', 'critical');
  await new Promise(resolve => {t.step_timeout(resolve, 3000)});
  assert_equals(syncObserver.changes().length, 1);

  await update_virtual_pressure_source('cpu', 'nominal');
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[1][0].state, 'nominal');

  assert_equals(syncObserver.changes().length, 2);
}, 'Changes that fail the "has change in data" test are discarded.');

mark_as_done();
