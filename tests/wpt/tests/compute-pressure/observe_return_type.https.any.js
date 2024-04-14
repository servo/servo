// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

// Regression test for https://issues.chromium.org/issues/333957909
// Make sure that observe() always returns a Promise.
pressure_test(async (t, mockPressureService) => {
  const observer = new PressureObserver(() => {});
  t.add_cleanup(() => observer.disconnect());

  for (let i = 0; i < 2; i++) {
    const promise = observer.observe('cpu');
    assert_class_string(promise, 'Promise');
    await promise;
  }
}, 'PressureObserver.observe() is idempotent');
