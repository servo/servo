// META: timeout=long
// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const sampleRateInHz = 10;
  const readings = ['nominal', 'fair', 'serious', 'critical'];
  // Normative values for rate obfuscation parameters.
  // https://w3c.github.io/compute-pressure/#rate-obfuscation-normative-parameters.
  const minPenaltyTimeInMs = 5000;
  const maxChangesThreshold = 100;
  const changes = await new Promise(async resolve => {
    const observerChanges = [];
    const observer = new PressureObserver(changes => {
      observerChanges.push(changes);
    }, {sampleRate: sampleRateInHz});

    observer.observe('cpu');
    mockPressureService.startPlatformCollector(sampleRateInHz);
    let i = 0;
    // mockPressureService.updatesDelivered() does not necessarily match
    // pressureChanges.length, as system load and browser optimizations can
    // cause the actual timer used by mockPressureService to deliver readings
    // to be a bit slower or faster than requested.
    while (observerChanges.length <= maxChangesThreshold) {
      mockPressureService.setPressureUpdate(
          'cpu', readings[i++ % readings.length]);
      await t.step_wait(
          () => mockPressureService.updatesDelivered() >= i,
          `At least ${i} readings have been delivered`);
    }
    observer.disconnect();
    resolve(observerChanges);
  });

  assert_equals(changes.length, (maxChangesThreshold + 1));

  let gotPenalty = false;
  for (let i = 0; i < changes.length; i++) {
    // Because penalty should be triggered once, one timestamp difference must
    // at least bigger or equal to the minimum penalty time specified.
    if ((changes[i + 1][0].time - changes[i][0].time) >= minPenaltyTimeInMs) {
      gotPenalty = true;
      break;
    }
  }
  assert_true(gotPenalty);
}, 'Rate obfuscation mitigation should have been triggered, when changes is higher than minimum changes before penalty');
