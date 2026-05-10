// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allkeyed
description: >
  If the constructor's `resolve` method is not callable, reject with a TypeError.
info: |
  Promise.allKeyed ( promises )

  ...
  3. Let promiseResolve be Completion(GetPromiseResolve(C)).
  4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
  ...

  GetPromiseResolve ( promiseConstructor )

  ...
  3. If IsCallable(promiseResolve) is false, throw a TypeError exception.
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

Promise.resolve = null;

asyncTest(function() {
  return assert.throwsAsync(TypeError, function() {
    return Promise.allKeyed({ key: 1 });
  });
});
