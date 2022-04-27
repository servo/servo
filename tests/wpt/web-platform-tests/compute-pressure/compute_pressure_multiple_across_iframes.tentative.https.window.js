'use strict';

promise_test(async t => {
  const update1_promise = new Promise((resolve, reject) => {
    const observer = new ComputePressureObserver(
        resolve, {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  // iframe numbers are aligned with observer numbers. The first observer is in
  // the main frame, so there is no iframe1.
  const iframe2 = document.createElement('iframe');
  document.body.appendChild(iframe2);

  const update2_promise = new Promise((resolve, reject) => {
    const observer = new iframe2.contentWindow.ComputePressureObserver(
        resolve, {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const iframe3 = document.createElement('iframe');
  document.body.appendChild(iframe3);

  const update3_promise = new Promise((resolve, reject) => {
    const observer = new iframe3.contentWindow.ComputePressureObserver(
        resolve, {cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5]});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const [update1, update2, update3] =
      await Promise.all([update1_promise, update2_promise, update3_promise]);

  for (const update of [update1, update2, update3]) {
    assert_in_array(update.cpuUtilization, [0.25, 0.75],
                    'cpuUtilization quantization');
    assert_in_array(update.cpuSpeed, [0.25, 0.75], 'cpuSpeed quantization');
  }
}, 'Three ComputePressureObserver instances in different iframes, but with ' +
   'the same quantization schema, receive updates');
