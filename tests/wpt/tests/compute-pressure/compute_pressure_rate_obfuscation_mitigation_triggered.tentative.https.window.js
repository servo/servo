// META: timeout=long
// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const sampleRateInHz = 25;
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
      if (observerChanges.length >= (minChangesThreshold - 1) && !gotPenalty) {
        // Add an assert to the maximum threshold possible.
        t.step(() => {
          assert_less_than_equal(observerChanges.length, maxChangesThreshold,
                                 "Sample count reaching maxChangesThreshold.");
        });

        const lastSample = observerChanges.at(-1);
        if ((changes[0].time - lastSample[0].time) >= minPenaltyTimeInMs) {
          // The update delivery might still be working even if
          // maxChangesThreshold have been reached and before disconnect() is
          // processed. This will corrupt the result for the above t.step().
          // Therefore we are adding a flag to dismiss any updates after the
          // penalty is detected, which is the condition for the test to pass.
          gotPenalty = true;
          observer.disconnect();
          resolve();
        }
      }
      observerChanges.push(changes);
    }, {sampleRate: sampleRateInHz});

    observer.observe('cpu');
    mockPressureService.startPlatformCollector(sampleRateInHz);
    let i = 0;
    // mockPressureService.updatesDelivered() does not necessarily match
    // pressureChanges.length, as system load and browser optimizations can
    // cause the actual timer used by mockPressureService to deliver readings
    // to be a bit slower or faster than requested.
    while (true) {
      mockPressureService.setPressureUpdate(
          'cpu', readings[i++ % readings.length]);
      await t.step_wait(
          () => mockPressureService.updatesDelivered() >= i,
          `At least ${i} readings have been delivered`);
    }
  });
}, 'Rate obfuscation mitigation should have been triggered, when changes is higher than minimum changes before penalty');
