// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Rejects when [[OwnPropertyKeys]] on the promises object throws
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...

  Promise.allSettledKeyed ( promises )

  ...
  6. Let result be Completion(PerformPromiseAllKeyed(...)).
  7. IfAbruptRejectPromise(result, promiseCapability).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Proxy]
---*/

var error = new Test262Error();
var input = new Proxy({}, {
  ownKeys: function() {
    throw error;
  }
});

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function() {
    throw new Test262Error('The promise should be rejected.');
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
