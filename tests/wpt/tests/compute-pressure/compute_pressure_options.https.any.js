// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu', {sampleInterval: 0});
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleInterval=*/ 200);
  });
}, 'PressureObserver observe method doesnt throw error for sampleInterval value 0');

promise_test(async t => {
  const observer =
      new PressureObserver(t.unreached_func('oops should not end up here'));
  t.add_cleanup(() => observer.disconnect());
  await promise_rejects_js(
      t, TypeError, observer.observe('cpu', {sampleInterval: -2}));
}, 'PressureObserver observe method requires a positive sampleInterval');

promise_test(async t => {
  const observer =
      new PressureObserver(t.unreached_func('oops should not end up here'));
  t.add_cleanup(() => observer.disconnect());
  await promise_rejects_js(
      t, TypeError, observer.observe('cpu', {sampleInterval: 2 ** 32}));
}, 'PressureObserver observe method requires a sampleInterval in unsigned long range');
