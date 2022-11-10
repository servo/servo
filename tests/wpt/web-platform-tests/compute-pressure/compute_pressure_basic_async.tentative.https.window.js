// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

pressure_test(async (t, mockPressureService) => {
  const changes = await new Promise(resolve => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    observer.observe('cpu');
    mockPressureService.setPressureUpdate('critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 1.0);
  });
  assert_true(changes.length === 1);
  assert_equals(changes[0].state, 'critical');
  assert_equals(changes[0].source, 'cpu');
  assert_equals(typeof changes[0].time, 'number');
}, 'Basic functionality test');

pressure_test((t, mockPressureService) => {
  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });

  observer.observe('cpu');
  observer.unobserve('cpu');
  mockPressureService.setPressureUpdate('critical');
  mockPressureService.startPlatformCollector(/*sampleRate=*/ 1.0);

  return new Promise(resolve => t.step_timeout(resolve, 1000));
}, 'Removing observer before observe() resolves works');
