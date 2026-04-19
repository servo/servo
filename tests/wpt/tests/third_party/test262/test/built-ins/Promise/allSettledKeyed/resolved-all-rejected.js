// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Resolution is a collection of all the settled keyed values (all rejected)
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  If variant is all-settled, then
    Let onRejected be a new Abstract Closure ...
  ...
  The onRejected closure:
    Let obj be OrdinaryObjectCreate(%Object.prototype%).
    Perform ! CreateDataPropertyOrThrow(obj, "status", "rejected").
    Perform ! CreateDataPropertyOrThrow(obj, "reason", x).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var obj = {};
var input = {
  first: Promise.reject(1),
  second: Promise.reject('test262'),
  third: Promise.reject(obj)
};

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function(settled) {
    assert.sameValue(Object.getPrototypeOf(settled), null);
    assert.compareArray(Object.keys(settled), ['first', 'second', 'third']);

    assert.sameValue(settled.first.status, 'rejected');
    assert.sameValue(settled.first.reason, 1);

    assert.sameValue(settled.second.status, 'rejected');
    assert.sameValue(settled.second.reason, 'test262');

    assert.sameValue(settled.third.status, 'rejected');
    assert.sameValue(settled.third.reason, obj);
  });
});
