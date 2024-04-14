// META: timeout=long
// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const sampleIntervalInMs = 40;
  const readings = ['nominal', 'fair', 'serious', 'critical'];
  // Normative values for rate obfuscation parameters.
  // https://w3c.github.io/compute-pressure/#rate-obfuscation-normative-parameters.
  const minPenaltyTimeInMs = 5000;
  const maxChangesThreshold = 100;
  const minChangesThreshold = 50;
  let gotPenalty = false;
  await new Promise(async resolve => {
    const observerChanges = [];
    const observer = new PressureObserver(changes => {
      if (observerChanges.length >= (minChangesThreshold - 1)) {
        const lastSample = observerChanges.at(-1);
        if ((changes[0].time - lastSample[0].time) >= minPenaltyTimeInMs) {
          // The update delivery might still be working even if
          // maxChangesThreshold have been reached and before disconnect() is
          // processed.
          // Therefore we are adding a flag to dismiss any updates after the
          // penalty is detected, which is the condition for the test to pass.
          gotPenalty = true;
          observer.disconnect();
          resolve();
        }
      }
      observerChanges.push(changes);
    });

    observer.observe('cpu', {sampleInterval: sampleIntervalInMs});
    mockPressureService.startPlatformCollector(sampleIntervalInMs);
    let i = 0;
    // mockPressureService.updatesDelivered() does not necessarily match
    // pressureChanges.length, as system load and browser optimizations can
    // cause the actual timer used by mockPressureService to deliver readings
    // to be a bit slower or faster than requested.
    while (observerChanges.length <= maxChangesThreshold || !gotPenalty) {
      mockPressureService.setPressureUpdate(
          'cpu', readings[i++ % readings.length]);
      // Allow tasks to run (avoid a micro-task loop).
      await new Promise((resolve) => t.step_timeout(resolve, 0));
      await t.step_wait(
          () => mockPressureService.updatesDelivered() >= i,
          `At least ${i} readings have been delivered`);
    }

    assert_true(gotPenalty, 'Penalty not triggered');

  });
}, 'Rate obfuscation mitigation should have been triggered, when changes is higher than minimum changes before penalty');
