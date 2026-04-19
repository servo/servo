// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Resolution is a collection of all the settled keyed values (all fulfilled)
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  If variant is all-settled, then
    Let onRejected be a new Abstract Closure ...
  ...
  The onFulfilled closure for all-settled:
    Let obj be OrdinaryObjectCreate(%Object.prototype%).
    Perform ! CreateDataPropertyOrThrow(obj, "status", "fulfilled").
    Perform ! CreateDataPropertyOrThrow(obj, "value", x).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var obj = {};
var input = {
  first: Promise.resolve(1),
  second: Promise.resolve('test262'),
  third: Promise.resolve(obj)
};

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function(settled) {
    assert.sameValue(Object.getPrototypeOf(settled), null);
    assert.compareArray(Object.keys(settled), ['first', 'second', 'third']);

    assert.sameValue(settled.first.status, 'fulfilled');
    assert.sameValue(settled.first.value, 1);

    assert.sameValue(settled.second.status, 'fulfilled');
    assert.sameValue(settled.second.value, 'test262');

    assert.sameValue(settled.third.status, 'fulfilled');
    assert.sameValue(settled.third.value, obj);
  });
});
