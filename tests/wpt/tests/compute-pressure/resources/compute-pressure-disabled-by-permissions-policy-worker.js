'use strict';

importScripts('/resources/testharness.js');

const header = 'Permissions-Policy header compute-pressure=()';
let workerType;

if (typeof postMessage === 'function') {
  workerType = 'dedicated';
}

promise_test(async t => {
  const observer =
      new PressureObserver(t.unreached_func('oops should not end up here'));
  await promise_rejects_dom(t, 'NotAllowedError', observer.observe('cpu'));
}, `$Inherited ${header} disallows ${workerType} workers.`);

done();
