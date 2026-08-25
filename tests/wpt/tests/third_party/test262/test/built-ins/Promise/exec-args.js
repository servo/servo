// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.3.1
description: Promise executor is invoked synchronously
info: |
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).

    25.4.1.3.2 Promise Resolve Functions

    The length property of a promise resolve function is 1.

    25.4.1.3.1 Promise Reject Functions

    The length property of a promise reject function is 1.
---*/

var callCount = 0;
var resolve, reject, argCount;

new Promise(function(a, b) {
  resolve = a;
  reject = b;
  argCount = arguments.length;
});

assert.sameValue(typeof resolve, 'function', 'type of first argument');
assert.sameValue(resolve.length, 1, 'length of first argument');
assert.sameValue(typeof reject, 'function', 'type of second argument');
assert.sameValue(reject.length, 1, 'length of second argument');
assert.sameValue(argCount, 2);
