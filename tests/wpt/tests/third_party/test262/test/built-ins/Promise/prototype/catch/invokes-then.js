// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.1
description: Promise.prototype.catch invokes `then` method
info: |
    1. Let promise be the this value.
    2. Return Invoke(promise, "then", «undefined, onRejected»).
---*/

var target = {};
var returnValue = {};
var callCount = 0;
var thisValue = null;
var argCount = null;
var firstArg = null;
var secondArg = null;
var result = null;

target.then = function(a, b) {
  callCount += 1;

  thisValue = this;
  argCount = arguments.length;
  firstArg = a;
  secondArg = b;

  return returnValue;
};

result = Promise.prototype.catch.call(target, 1, 2, 3);

assert.sameValue(callCount, 1, 'Invokes `then` method exactly once');
assert.sameValue(
  thisValue,
  target,
  'Invokes `then` method with the instance as the `this` value'
);
assert.sameValue(
  argCount, 2, 'Invokes `then` method with exactly two single arguments'
);
assert.sameValue(
  firstArg,
  undefined,
  'Invokes `then` method with `undefined` as the first argument'
);
assert.sameValue(
  secondArg, 1, 'Invokes `then` method with the provided argument'
);
assert.sameValue(
  result, returnValue, 'Returns the result of the invocation of `then`'
);
