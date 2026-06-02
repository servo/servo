'use strict';

importScripts('/resources/testharness.js');

const header = 'Permissions-Policy header hid=()';
let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(() => navigator.hid.getDevices().then(
        () => assert_unreached('expected promise to reject with SecurityError'),
        error => assert_equals(error.name, 'SecurityError')),
    `Inherited ${header} disallows ${workerType} workers.`);

done();
