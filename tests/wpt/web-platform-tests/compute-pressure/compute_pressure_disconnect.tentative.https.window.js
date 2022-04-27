'use strict';

promise_test(async t => {
  const observer1_updates = [];
  const observer1 = new ComputePressureObserver(update => {
    observer1_updates.push(update);
  }, {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
  t.add_cleanup(() => observer1.disconnect());
  // Ensure that observer1's schema gets registered before observer2 starts.
  await observer1.observe('cpu');
  observer1.disconnect();

  const observer2_updates = [];
  await new Promise((resolve, reject) => {
    const observer2 = new ComputePressureObserver(update => {
      observer2_updates.push(update);
      resolve();
    }, {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
    t.add_cleanup(() => observer2.disconnect());
    observer2.observe('cpu').catch(reject);
  });

  assert_equals(
      observer1_updates.length, 0,
      'disconnected observers should not receive callbacks');

  assert_equals(observer2_updates.length, 1);
  assert_in_array(
      observer2_updates[0].cpuUtilization, [0.25, 0.75],
      'cpuUtilization quantization');
  assert_in_array(
      observer2_updates[0].cpuSpeed, [0.25, 0.75], 'cpuSpeed quantization');

  // Go through one more update cycle so any (incorrect) update for observer1
  // makes it through the IPC queues.

  const observer3_updates = [];
  await new Promise((resolve, reject) => {
    const observer3 = new ComputePressureObserver(update => {
      observer3_updates.push(update);
      resolve();
    }, {cpuUtilizationThresholds: [0.75], cpuSpeedThresholds: [0.25]});
    t.add_cleanup(() => observer3.disconnect());
    observer3.observe('cpu').catch(reject);
  });

  assert_equals(
      observer1_updates.length, 0,
      'disconnected observers should not receive callbacks');

  assert_equals(observer3_updates.length, 1);
  assert_in_array(
      observer3_updates[0].cpuUtilization, [0.375, 0.875],
      'cpuUtilization quantization');
  assert_in_array(
      observer3_updates[0].cpuSpeed, [0.125, 0.625], 'cpuSpeed quantization');
}, 'Stopped ComputePressureObservers do not receive updates');
