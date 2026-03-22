'use strict';

importScripts('/resources/testharness.js');

setup(function() {
  assert_true(typeof IdleDetector !== 'undefined', 'IdleDetector must be defined');
});

let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(async () => {
  await new IdleDetector().start()
},
    `Inherited header permissions policy allows ${workerType} workers.`)

done();
