// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

'use strict';

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  });
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  const promise = observer1.observe('cpu');
  observer1.disconnect();
  await promise_rejects_dom(t, 'AbortError', promise);

  const observer2_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });
  await update_virtual_pressure_source('cpu', 'critical');
  const observer2_changes = await observer2_promise;

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].state, 'critical');
}, 'Stopped PressureObservers do not receive changes');

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  });
  t.add_cleanup(() => observer1.disconnect());

  let observer2;
  const observer2_promise = new Promise(resolve => {
    observer2 = new PressureObserver(resolve);
    t.add_cleanup(() => observer2.disconnect());
  });

  await update_virtual_pressure_source('cpu', 'critical');

  const observer1_observe_promise = observer1.observe('cpu');
  observer2.observe('cpu');
  observer1.disconnect();
  await promise_rejects_dom(t, 'AbortError', observer1_observe_promise);
  const observer2_changes = await observer2_promise;

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].state, 'critical');
}, 'Removing observer before observe() resolves does not affect other observers');

mark_as_done();
