// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

test(t => {
  const observer = new PressureObserver(
      t.unreached_func('This callback should not have been called.'));

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record before observe');
}, 'Calling takeRecords() before observe()');

pressure_test(async (t, mockPressureService) => {
  let observer;
  const changes = await new Promise(resolve => {
    observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());

    observer.observe('cpu');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
  assert_equals(changes[0].state, 'critical');

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record available');
}, 'takeRecords() returns empty record after callback invoke');
