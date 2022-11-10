// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

test(t => {
  const observer = new PressureObserver(
      t.unreached_func('This callback should not have been called.'),
      {sampleRate: 1.0});

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record before observe');
}, 'Calling takeRecords() before observe()');

promise_test(async t => {
  let observer;
  const changes = await new Promise(resolve => {
    observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());

    observer.observe('cpu');
  });
  assert_in_array(
      changes[0].state, ['nominal', 'fair', 'serious', 'critical'],
      'cpu presure state');

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record available');
}, 'takeRecords() returns empty record after callback invoke');
