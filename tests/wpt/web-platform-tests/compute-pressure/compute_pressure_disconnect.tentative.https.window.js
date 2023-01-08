'use strict';

test(t => {
  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });
  t.add_cleanup(() => observer.disconnect());
  observer.disconnect();
}, 'Call disconnect() directly should not crash');

promise_test(async t => {
  const observer1_changes = [];
  const observer1 = new PressureObserver(change => {
    observer1_changes.push(change);
  }, {sampleRate: 1.0});
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  await observer1.observe('cpu');
  observer1.disconnect();

  const observer2_changes = [];
  await new Promise((resolve, reject) => {
    const observer2 = new PressureObserver(change => {
      observer2_changes.push(change);
      resolve();
    }, {sampleRate: 1.0});
    t.add_cleanup(() => observer2.disconnect());
    observer2.observe('cpu').catch(reject);
  });

  assert_equals(
      observer1_changes.length, 0,
      'disconnected observers should not receive callbacks');

  assert_equals(observer2_changes.length, 1);
  assert_equals(observer2_changes[0].length, 1);
  assert_in_array(
      observer2_changes[0][0].state, ['nominal', 'fair', 'serious', 'critical'],
      'cpu pressure state');
}, 'Stopped PressureObserver do not receive changes');
