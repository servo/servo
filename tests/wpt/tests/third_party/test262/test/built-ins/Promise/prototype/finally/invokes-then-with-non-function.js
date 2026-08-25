// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: Promise.prototype.finally invokes `then` method
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
---*/

var target = new Promise(function() {});
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

result = Promise.prototype.finally.call(target, 1, 2, 3);

assert.sameValue(callCount, 1, 'Invokes `then` method exactly once');
assert.sameValue(
  thisValue,
  target,
  'Invokes `then` method with the instance as the `this` value'
);
assert.sameValue(argCount, 2, 'Invokes `then` method with exactly two single arguments');
assert.sameValue(
  firstArg,
  1,
  'Invokes `then` method with the provided non-callable first argument'
);
assert.sameValue(
  secondArg,
  1,
  'Invokes `then` method with the provided non-callable first argument'
);
assert.sameValue(result, returnValue, 'Returns the result of the invocation of `then`');
