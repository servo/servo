// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const changes = await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
  assert_true(changes.length === 1);
  assert_equals(changes[0].state, 'critical');
  assert_equals(changes[0].source, 'cpu');
  assert_equals(typeof changes[0].time, 'number');
}, 'Basic factors functionality test');
