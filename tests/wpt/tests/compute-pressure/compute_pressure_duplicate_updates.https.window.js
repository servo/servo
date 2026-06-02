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
  await syncObserver.observer().observe('cpu');

  await update_virtual_pressure_source('cpu', 'critical', 0.2);
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[0][0].state, 'critical');
  assert_equals(syncObserver.changes()[0][0].ownContributionEstimate, 0.2);

  await update_virtual_pressure_source('cpu', 'critical', 0.2);
  await new Promise(resolve => {t.step_timeout(resolve, 3000)});
  assert_equals(syncObserver.changes().length, 1);

  await update_virtual_pressure_source('cpu', 'nominal', 0.2);
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[1][0].state, 'nominal');
  assert_equals(syncObserver.changes()[1][0].ownContributionEstimate, 0.2);

  assert_equals(syncObserver.changes().length, 2);
}, 'Changes that fail the "should dispatch" test are discarded.');

pressure_test(async (t) => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const syncObserver = new SyncPressureObserver(t);
  await syncObserver.observer().observe('cpu', {sampleInterval: 500});

  await update_virtual_pressure_source('cpu', 'critical');
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[0][0].state, 'critical');

  await update_virtual_pressure_source('cpu', 'critical');
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[1][0].state, 'critical');

  await update_virtual_pressure_source('cpu', 'nominal');
  await syncObserver.waitForUpdate();
  assert_equals(syncObserver.changes()[2][0].state, 'nominal');

  assert_equals(syncObserver.changes().length, 3);
}, 'Updates should be received even when no state change, if sampleInterval is set.');

mark_as_done();
