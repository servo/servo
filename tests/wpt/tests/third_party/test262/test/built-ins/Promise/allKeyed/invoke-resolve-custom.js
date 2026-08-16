// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Uses the receiver constructor's `resolve` method with the receiver as this value
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  5. For each element key of allKeys, do
    ...
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, ctor, « propertyValue »).
features: [await-dictionary]
---*/

var resultStartCount = 0;
var resultEndCount = 0;
var resolveCallCount = 0;
var resolveValues = [];

Promise.resolve = Test262Error.thrower;

function Constructor(executor) {
  function resolve(result) {
    resultStartCount += 1;
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.sameValue(result.first, 1, "result.first");
    assert.sameValue(result.second, 2, "result.second");
    resultEndCount += 1;
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(value) {
  assert.sameValue(this, Constructor, "resolve receiver");
  resolveCallCount += 1;
  resolveValues.push(value);
  return {
    then: function(onFulfilled) {
      onFulfilled(value);
    }
  };
};

Promise.allKeyed.call(Constructor, {
  first: 1,
  second: 2
});

assert.sameValue(resolveCallCount, 2, "resolve call count");
assert.sameValue(resolveValues[0], 1, "first resolve value");
assert.sameValue(resolveValues[1], 2, "second resolve value");
assert.sameValue(resultStartCount, 1, "result callback entered once");
assert.sameValue(resultEndCount, 1, "result callback completed once");
