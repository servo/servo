// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

pressure_test(async (t, mockPressureService) => {
  const readings = ['nominal', 'fair', 'serious', 'critical'];

  const sampleRate = 4.0;
  const pressureChanges = await new Promise(async resolve => {
    const observerChanges = [];
    const observer = new PressureObserver(changes => {
      observerChanges.push(changes);
    }, {sampleRate});
    observer.observe('cpu');

    mockPressureService.startPlatformCollector(sampleRate * 2);
    let i = 0;
    // mockPressureService.updatesDelivered() does not necessarily match
    // pressureChanges.length, as system load and browser optimizations can
    // cause the actual timer used by mockPressureService to deliver readings
    // to be a bit slower or faster than requested.
    while (observerChanges.length < 4) {
      mockPressureService.setPressureUpdate(readings[i++ % readings.length]);
      await t.step_wait(
          () => mockPressureService.updatesDelivered() >= i,
          `At least ${i} readings have been delivered`);
    }
    observer.disconnect();
    resolve(observerChanges);
  });

  assert_equals(pressureChanges.length, 4);
  assert_greater_than_equal(
      pressureChanges[1][0].time - pressureChanges[0][0].time,
      (1 / sampleRate * 1000));
  assert_greater_than_equal(
      pressureChanges[2][0].time - pressureChanges[1][0].time,
      (1 / sampleRate * 1000));
  assert_greater_than_equal(
      pressureChanges[3][0].time - pressureChanges[2][0].time,
      (1 / sampleRate * 1000));
}, 'Faster collector: Timestamp difference between two changes should be higher or equal to the observer sample rate');
