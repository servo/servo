'use strict';

promise_test(async t => {
  const update = await new Promise(resolve => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu');
    observer.observe('cpu');
    observer.observe('cpu');
  });

  assert_equals(typeof update[0].state, 'string');
  assert_in_array(
      update[0].state, ['nominal', 'fair', 'serious', 'critical'],
      'cpu pressure state');
}, 'PressureObserver.observe() is idempotent');
