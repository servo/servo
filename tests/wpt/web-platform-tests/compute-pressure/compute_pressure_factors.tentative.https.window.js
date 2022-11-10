// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

pressure_test(async (t, mockPressureService) => {
  const changes = await new Promise(resolve => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    observer.observe('cpu');
    mockPressureService.setPressureUpdate('critical', ['thermal']);
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 1.0);
  });
  assert_true(changes.length === 1);
  assert_equals(changes[0].state, 'critical');
  assert_equals(changes[0].source, 'cpu');
  assert_equals(typeof changes[0].time, 'number');
  assert_equals(changes[0].factors[0], 'thermal');
}, 'Basic factors functionality test');
