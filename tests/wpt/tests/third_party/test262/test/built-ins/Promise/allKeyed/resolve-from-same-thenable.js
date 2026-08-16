// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Cannot tamper remainingElementsCount when the same resolve element function is
  called multiple times.
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  The onFulfilled closure:
    ...
    1. Let activeFunc be the active function object.
    2. If activeFunc.[[AlreadyCalled]].[[Value]] is true, return undefined.
    3. Set activeFunc.[[AlreadyCalled]].[[Value]] to true.
features: [await-dictionary]
---*/

var resultStartCount = 0;
var resultEndCount = 0;

function Constructor(executor) {
  function resolve(result) {
    resultStartCount += 1;
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.sameValue(result.key, "first", "first fulfillment wins");
    resultEndCount += 1;
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(value) {
  return value;
};

var input = {
  key: {
    then: function(onFulfilled) {
      onFulfilled("first");
      assert.sameValue(onFulfilled("second"), undefined);
      onFulfilled("third");
    }
  }
};

Promise.allKeyed.call(Constructor, input);

assert.sameValue(resultStartCount, 1, "result callback entered once");
assert.sameValue(resultEndCount, 1, "result callback completed once");
