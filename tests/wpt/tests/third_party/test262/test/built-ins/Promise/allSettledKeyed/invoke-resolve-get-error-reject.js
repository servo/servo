// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Error retrieving the constructor's `resolve` method rejects the promise
info: |
  Promise.allSettledKeyed ( promises )

  ...
  3. Let promiseResolve be Completion(GetPromiseResolve(C)).
  4. IfAbruptRejectPromise(promiseResolve, promiseCapability).

  GetPromiseResolve ( promiseConstructor )

  1. Let promiseResolve be ? Get(promiseConstructor, "resolve").
  ...
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
Object.defineProperty(Promise, 'resolve', {
  get: function() {
    throw error;
  }
});

asyncTest(function() {
  return Promise.allSettledKeyed({ key: 1 }).then(function() {
    throw new Test262Error('The promise should be rejected.');
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
