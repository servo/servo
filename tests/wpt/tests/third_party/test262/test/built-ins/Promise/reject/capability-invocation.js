// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Invocation of "reject" capability
esid: sec-promise.reject
info: |
    1. Let C be the this value.
    [...]
    3. Let promiseCapability be ? NewPromiseCapability(C).
    4. Perform ? Call(promiseCapability.[[Reject]], undefined, « r »).
    [...]

    25.4.1.5 NewPromiseCapability
    [...]
    6. Let promise be Construct(C, «executor»).
    7. ReturnIfAbrupt(promise).
---*/

var expectedThis = (function() {
  return this;
})();
var resolveCount = 0;
var thisValue, args;
var P = function(executor) {
  return new Promise(function() {
    executor(
      function() {
        resolveCount += 1;
      },
      function() {
        thisValue = this;
        args = arguments;
      }
    );
  });
};

Promise.reject.call(P, 24601);

assert.sameValue(resolveCount, 0);

assert.sameValue(thisValue, expectedThis);
assert.sameValue(typeof args, 'object');
assert.sameValue(args.length, 1);
assert.sameValue(args[0], 24601);
