// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addition-operator-plus-runtime-semantics-evaluation
es6id: 12.7.3.1
description: >
  Behavior when error is thrown while accessing `Symbol.toPrimitive` property
info: |
    [...]
    5. Let lprim be ? ToPrimitive(lval).
    6. Let rprim be ? ToPrimitive(rval).
    [...]

    ES6 Section 7.1.1 ToPrimitive ( input [, PreferredType] )

    1. If PreferredType was not passed, let hint be "default".
    [...]
    4. Let exoticToPrim be GetMethod(input, @@toPrimitive).
    5. ReturnIfAbrupt(exoticToPrim).
    6. If exoticToPrim is not undefined, then
       a. Let result be Call(exoticToPrim, input, «hint»).
       [...]
features: [Symbol.toPrimitive]
---*/

var thrower = {};
var counter = {};
var callCount = 0;

Object.defineProperty(thrower, Symbol.toPrimitive, {
  get: function() {
    throw new Test262Error();
  }
});
Object.defineProperty(counter, Symbol.toPrimitive, {
  get: function() {
    callCount += 1;
  }
});

assert.throws(Test262Error, function() {
  thrower + counter;
}, 'error from property access of left-hand side');

assert.sameValue(callCount, 0);

assert.throws(Test262Error, function() {
  counter + thrower;
}, 'error from property access of right-hand side');

assert.sameValue(callCount, 1);
