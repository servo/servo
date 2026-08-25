// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Gets the constructor's `resolve` method once and calls it for each enumerable key
info: |
  Promise.allSettledKeyed ( promises )

  ...
  3. Let promiseResolve be Completion(GetPromiseResolve(ctor)).
  ...

  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  5. For each element key of allKeys, do
    ...
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, ctor, « propertyValue »).
features: [await-dictionary]
---*/

var originalResolve = Promise.resolve;
var getCount = 0;
var callCount = 0;

Object.defineProperty(Promise, "resolve", {
  configurable: true,
  get: function() {
    getCount += 1;
    return function() {
      callCount += 1;
      return originalResolve.apply(Promise, arguments);
    };
  }
});

var input = {
  first: 1,
  second: 2
};
Object.defineProperty(input, "hidden", {
  value: 3,
  enumerable: false
});

Promise.allSettledKeyed(input);

assert.sameValue(getCount, 1, "resolve getter called once");
assert.sameValue(callCount, 2, "resolve called once for each enumerable key");
