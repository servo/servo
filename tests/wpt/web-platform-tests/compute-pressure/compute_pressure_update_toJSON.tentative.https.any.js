// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

pressure_test(async (t, mockPressureService) => {
  const changes = await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    observer.observe('cpu');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
  assert_true(changes.length === 1);
  const json = changes[0].toJSON();
  assert_equals(json.state, 'critical');
  assert_equals(json.source, 'cpu');
  assert_equals(typeof json.time, 'number');
}, 'Basic functionality test');
