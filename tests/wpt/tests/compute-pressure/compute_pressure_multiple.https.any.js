// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const changes1_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const changes2_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const changes3_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  mockPressureService.setPressureUpdate('cpu', 'critical');
  mockPressureService.startPlatformCollector(/*sampleInterval=*/ 200);

  const [changes1, changes2, changes3] =
      await Promise.all([changes1_promise, changes2_promise, changes3_promise]);

  for (const changes of [changes1, changes2, changes3]) {
    assert_equals(changes[0].state, 'critical');
  }
}, 'Three PressureObserver instances receive changes');
