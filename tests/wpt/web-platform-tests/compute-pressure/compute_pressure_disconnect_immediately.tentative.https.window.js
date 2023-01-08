'use strict';

promise_test(async t => {
  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  }, {sampleRate: 1.0});
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  const promise = observer1.observe('cpu');
  observer1.disconnect();
  await promise_rejects_dom(t, 'NotSupportedError', promise);

  const observer2_changes = [];
  await new Promise((resolve, reject) => {
    const observer2 = new PressureObserver(changes => {
      observer2_changes.push(changes);
      resolve();
    }, {sampleRate: 1.0});
    t.add_cleanup(() => observer2.disconnect());
    observer2.observe('cpu').catch(reject);
  });

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].length, 1);
  assert_in_array(
      observer2_changes[0][0].state, ['nominal', 'fair', 'serious', 'critical'],
      'cpu pressure state');
}, 'Stopped PressureObserver do not receive changes');

promise_test(async t => {
  const observer1_changes = [];
  const observer1 = new PressureObserver(changes => {
    observer1_changes.push(changes);
  }, {sampleRate: 1});
  t.add_cleanup(() => observer1.disconnect());

  const observer2_changes = [];
  await new Promise(async resolve => {
    const observer2 = new PressureObserver(changes => {
      observer2_changes.push(changes);
      resolve();
    }, {sampleRate: 1});
    t.add_cleanup(() => observer2.disconnect());
    const promise = observer1.observe('cpu');
    observer2.observe('cpu');
    observer1.disconnect();
    await promise_rejects_dom(t, 'NotSupportedError', promise);
  });

  assert_equals(
      observer1_changes.length, 0,
      'stopped observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_in_array(
      observer2_changes[0][0].state, ['nominal', 'fair', 'serious', 'critical'],
      'cpu pressure state');
}, 'Removing observer before observe() resolves does not affect other observers');
