'use strict';

promise_test(async t => {
  const observer1_updates = [];
  const observer1 = new ComputePressureObserver(
      update => { observer1_updates.push(update); },
      {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
  t.add_cleanup(() => observer1.stop());
  // Ensure that observer1's quantization scheme gets registered as the origin's
  // scheme before observer2 starts.
  await observer1.observe();

  // iframe numbers are aligned with observer numbers.  The first observer is in
  // the main frame, so there is no iframe1.
  const iframe2 = document.createElement('iframe');
  document.body.appendChild(iframe2);

  const observer2_updates = [];
  await new Promise((resolve, reject) => {
    const observer2 = new iframe2.contentWindow.ComputePressureObserver(
        update => {
          observer2_updates.push(update);
          resolve();
        },
        {cpuUtilizationThresholds: [0.25], cpuSpeedThresholds: [0.75]});
    t.add_cleanup(() => observer2.stop());
    observer2.observe().catch(reject);
  });

  // observer2 uses a different quantization scheme than observer1. After
  // observer2.observe() completes, observer1 should no longer be active.
  //
  // The check below assumes that observer2.observe() completes before the
  // browser dispatches any update for observer1.  This assumption is highly
  // likely to be true, because there shold be a 1-second delay between
  // observer1.observe() and the first update that observer1 would receive.
  assert_equals(
      observer1_updates.length, 0,
      'observer2.observe() should have stopped observer1; the two observers ' +
      'have different quantization schemes');

  assert_equals(observer2_updates.length, 1);
  assert_in_array(observer2_updates[0].cpuUtilization, [0.125, 0.625],
                  'cpuUtilization quantization');
  assert_in_array(observer2_updates[0].cpuSpeed, [0.375, 0.875],
                  'cpuSpeed quantization');

  // Go through one more update cycle so any (incorrect) update for observer1
  // makes it through the IPC queues.
  observer1_updates.length = 0;
  observer2_updates.length = 0;

  const iframe3 = document.createElement('iframe');
  document.body.appendChild(iframe3);

  const observer3_updates = [];
  await new Promise((resolve, reject) => {
    const observer3 = new iframe3.contentWindow.ComputePressureObserver(
        update => {
          observer3_updates.push(update);
          resolve();
        },
        {cpuUtilizationThresholds: [0.75], cpuSpeedThresholds: [0.25]});
    t.add_cleanup(() => observer3.stop());
    observer3.observe().catch(reject);
  });

  assert_equals(
      observer1_updates.length, 0,
      'observer2.observe() should have stopped observer1; the two observers ' +
      'have different quantization schemes');

  // observer3 uses a different quantization scheme than observer2. So,
  // observer3.observe() should stop observer2.
  assert_equals(
      observer2_updates.length, 0,
      'observer3.observe() should have stopped observer2; the two observers ' +
      'have different quantization schemes');

  assert_equals(observer3_updates.length, 1);
  assert_in_array(observer3_updates[0].cpuUtilization, [0.375, 0.875],
                  'cpuUtilization quantization');
  assert_in_array(observer3_updates[0].cpuSpeed, [0.125, 0.625],
                  'cpuSpeed quantization');

}, 'ComputePressureObserver with a new quantization schema stops all ' +
   'other active observers');
