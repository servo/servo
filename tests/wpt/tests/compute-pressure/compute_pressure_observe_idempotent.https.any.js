// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test(async (t, mockPressureService) => {
  const update = await new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
    observer.observe('cpu').catch(reject);
    observer.observe('cpu').catch(reject);
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleInterval=*/ 200);
  });

  assert_equals(update[0].state, 'critical');
}, 'PressureObserver.observe() is idempotent');
