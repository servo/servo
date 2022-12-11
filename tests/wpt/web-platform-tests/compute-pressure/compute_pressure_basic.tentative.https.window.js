'use strict';

promise_test(async t => {
  await new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });
}, 'An active PressureObserver calls its callback at least once');
