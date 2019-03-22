'use strict';

importScripts('/resources/testharness.js');

let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(async () => {
  await new IdleDetector().start()
},
    `Inherited header feature policy allows ${workerType} workers.`)

done();
