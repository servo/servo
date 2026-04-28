// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Abrupt completion returned by "reject" capability
esid: sec-promise.reject
info: |
    1. Let C be the this value.
    [...]
    3. Let promiseCapability be ? NewPromiseCapability(C).
    4. Perform ? Call(promiseCapability.[[Reject]], undefined, « r »).

    25.4.1.5 NewPromiseCapability
    [...]
    6. Let promise be Construct(C, «executor»).
    7. ReturnIfAbrupt(promise).
---*/

var P = function(executor) {
  return new Promise(function() {
    executor(
      function() {},
      function() {
        throw new Test262Error();
      }
    );
  });
};

assert.throws(Test262Error, function() {
  Promise.reject.call(P);
});
