'use strict';

importScripts('/resources/testharness.js');

let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(() => navigator.idle.query(),
    `Inherited header feature policy allows ${workerType} workers.`)

done();
