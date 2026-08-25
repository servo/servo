// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Rejects when [[GetOwnProperty]] on the promises object throws
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    ...

  Promise.allKeyed ( promises )

  ...
  6. Let result be Completion(PerformPromiseAllKeyed(...)).
  7. IfAbruptRejectPromise(result, promiseCapability).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Proxy]
---*/

var error = new Test262Error();
var input = new Proxy({ key: 1 }, {
  getOwnPropertyDescriptor: function() {
    throw error;
  }
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function() {
    throw new Test262Error('The promise should be rejected.');
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
