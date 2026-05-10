// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allKeyed resolves an empty object to an empty null-prototype object
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  7. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
  8. If remainingElementsCount.[[Value]] = 0, then
    a. Let result be CreateKeyedPromiseCombinatorResultObject(keys, values).
    b. Perform ? Call(resultCapability.[[Resolve]], undefined, « result »).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

asyncTest(function() {
  return Promise.allKeyed({}).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);
    assert.sameValue(result.hasOwnProperty, undefined);
    assert.compareArray(Reflect.ownKeys(result), []);
  });
});
