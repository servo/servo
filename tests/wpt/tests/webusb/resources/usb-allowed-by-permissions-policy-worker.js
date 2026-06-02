'use strict';

importScripts('/resources/testharness.js');

let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(() => navigator.usb.getDevices(),
    `Inherited header permissions policy allows ${workerType} workers.`);

done();
