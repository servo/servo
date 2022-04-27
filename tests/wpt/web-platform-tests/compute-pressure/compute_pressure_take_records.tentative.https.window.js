'use strict';

test(t => {
  const observer = new ComputePressureObserver(
      t.unreached_func('This callback should not have been called.'),
      {cpuUtilizationThresholds: [0.25], cpuSpeedThresholds: [0.75]});

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record before observe');
}, 'Calling takeRecords() before observe()');

promise_test(async t => {
  let observer;
  const record = await new Promise((resolve, reject) => {
    observer = new ComputePressureObserver(
        resolve,
        {cpuUtilizationThresholds: [0.25], cpuSpeedThresholds: [0.75]});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  assert_in_array(
      record.cpuUtilization, [0.125, 0.625], 'cpuUtilization quantization');
  assert_in_array(record.cpuSpeed, [0.375, 0.875], 'cpuSpeed quantization');

  const records = observer.takeRecords();
  assert_equals(records.length, 0, 'No record available');
}, 'takeRecords() returns empty record after callback invoke');
