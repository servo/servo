// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allSettledKeyed result key order matches property key order, not settlement order
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  6. For each element key of allKeys, do
    ...
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
      ii. Append key to keys.
      ...
    ...
  ...
  8. If remainingElementsCount.[[Value]] = 0, then
    ...
    b. Let result be CreateKeyedPromiseCombinatorResultObject(keys, values).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var resolveFirst;
var resolveSecond;
var resolveThird;

var input = {
  first: new Promise(function(resolve) {
    resolveFirst = resolve;
  }),
  second: new Promise(function(resolve) {
    resolveSecond = resolve;
  }),
  third: new Promise(function(resolve) {
    resolveThird = resolve;
  })
};

var combined = Promise.allSettledKeyed(input);

resolveSecond('second');
resolveThird('third');
resolveFirst('first');

asyncTest(function() {
  return combined.then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);
    assert.compareArray(Object.keys(result), ['first', 'second', 'third']);
    assert.sameValue(result.first.status, 'fulfilled');
    assert.sameValue(result.first.value, 'first');
    assert.sameValue(result.second.status, 'fulfilled');
    assert.sameValue(result.second.value, 'second');
    assert.sameValue(result.third.status, 'fulfilled');
    assert.sameValue(result.third.value, 'third');
  });
});
