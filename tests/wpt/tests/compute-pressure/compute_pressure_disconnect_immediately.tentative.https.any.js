// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  });
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  const promise = observer1.observe('cpu');
  observer1.disconnect();
  await promise_rejects_dom(t, 'AbortError', promise);

  const observer2_changes = [];
  await new Promise((resolve, reject) => {
    const observer2 = new PressureObserver(changes => {
      observer2_changes.push(changes);
      resolve();
    });
    t.add_cleanup(() => observer2.disconnect());
    observer2.observe('cpu').catch(reject);
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].length, 1);
  assert_equals(observer2_changes[0][0].state, 'critical');
}, 'Stopped PressureObserver do not receive changes');

pressure_test(async (t, mockPressureService) => {
  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  });
  t.add_cleanup(() => observer1.disconnect());

  const observer2_changes = [];
  await new Promise(async resolve => {
    const observer2 = new PressureObserver(changes => {
      observer2_changes.push(changes);
      resolve();
    });
    t.add_cleanup(() => observer2.disconnect());
    const promise = observer1.observe('cpu');
    observer2.observe('cpu');
    observer1.disconnect();
    await promise_rejects_dom(t, 'AbortError', promise);
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0][0].state, 'critical');
}, 'Removing observer before observe() resolves does not affect other observers');
