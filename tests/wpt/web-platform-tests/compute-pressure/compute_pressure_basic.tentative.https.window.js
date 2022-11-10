'use strict';

promise_test(async t => {
  await new Promise((resolve) => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu');
  });
}, 'An active PressureObserver calls its callback at least once');
