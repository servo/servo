'use strict';

promise_test(async t => {
  // The quantization thresholds and the quantized values that they lead to can
  // be represented exactly in floating-point, so === comparison works.

  const update = await new Promise((resolve, reject) => {
    const observer = new ComputePressureObserver(
        resolve,
        {cpuUtilizationThresholds: [0.25], cpuSpeedThresholds: [0.75]});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  assert_in_array(
      update.cpuUtilization, [0.125, 0.625], 'cpuUtilization quantization');
  assert_in_array(update.cpuSpeed, [0.375, 0.875], 'cpuSpeed quantization');
}, 'ComputePressureObserver quantizes utilization and speed separately');
