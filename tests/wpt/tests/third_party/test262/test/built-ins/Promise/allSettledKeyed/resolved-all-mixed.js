// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Resolution is a collection of all the settled keyed values (fulfilled and rejected)
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    ...
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
      11. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
  ...
  8. If remainingElementsCount.[[Value]] = 0, then
    ...
    b. Let result be CreateKeyedPromiseCombinatorResultObject(keys, values).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var obj1 = {};
var obj2 = {};
var input = {
  first: Promise.reject(1),
  second: Promise.resolve(2),
  third: Promise.resolve('tc39'),
  fourth: Promise.reject('test262'),
  fifth: Promise.reject(obj1),
  sixth: Promise.resolve(obj2)
};

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function(settled) {
    assert.sameValue(Object.getPrototypeOf(settled), null);
    assert.compareArray(Object.keys(settled), [
      'first',
      'second',
      'third',
      'fourth',
      'fifth',
      'sixth'
    ]);

    assert.sameValue(settled.first.status, 'rejected');
    assert.sameValue(settled.first.reason, 1);

    assert.sameValue(settled.second.status, 'fulfilled');
    assert.sameValue(settled.second.value, 2);

    assert.sameValue(settled.third.status, 'fulfilled');
    assert.sameValue(settled.third.value, 'tc39');

    assert.sameValue(settled.fourth.status, 'rejected');
    assert.sameValue(settled.fourth.reason, 'test262');

    assert.sameValue(settled.fifth.status, 'rejected');
    assert.sameValue(settled.fifth.reason, obj1);

    assert.sameValue(settled.sixth.status, 'fulfilled');
    assert.sameValue(settled.sixth.value, obj2);
  });
});
