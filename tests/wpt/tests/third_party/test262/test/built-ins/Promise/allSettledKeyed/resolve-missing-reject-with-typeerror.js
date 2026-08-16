// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  If the constructor does not have a `resolve` method, reject with a TypeError.
info: |
  Promise.allSettledKeyed ( promises )

  ...
  3. Let promiseResolve be Completion(GetPromiseResolve(ctor)).
  4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
  ...

  GetPromiseResolve ( promiseConstructor )

  ...
  2. Let promiseResolve be ? Get(promiseConstructor, "resolve").
  3. If IsCallable(promiseResolve) is false, throw a TypeError exception.
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

delete Promise.resolve;

asyncTest(function() {
  return assert.throwsAsync(TypeError, function() {
    return Promise.allSettledKeyed({ key: 1 });
  });
});
