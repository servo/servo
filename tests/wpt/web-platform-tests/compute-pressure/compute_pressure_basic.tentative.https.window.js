'use strict';

promise_test(async t => {
  await new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });
}, 'An active PressureObserver calls its callback at least once');

promise_test(async t => {
  await new Promise(resolve => {
    const myDedicatedWorker = new Worker(
        '/compute-pressure/resources/observer-in-dedicated-worker.js');
    myDedicatedWorker.onmessage = event => {
      assert_equals(typeof event.data.time, 'number');
      assert_equals('cpu', event.data.source);
      resolve();
    };
  });
}, 'Test receives updates from dedicated worker');

promise_test(async t => {
  await new Promise(resolve => {
    const mySharedWorker = new SharedWorker(
        '/compute-pressure/resources/observer-in-shared-worker.js');
    mySharedWorker.port.start();
    mySharedWorker.port.postMessage('observe');
    mySharedWorker.port.onmessage = event => {
      assert_equals(typeof event.data.time, 'number');
      assert_equals('cpu', event.data.source);
      resolve();
    };
  });
}, 'Test receives updates from shared worker');
