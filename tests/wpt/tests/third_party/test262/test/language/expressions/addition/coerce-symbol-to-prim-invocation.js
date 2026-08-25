// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addition-operator-plus-runtime-semantics-evaluation
es6id: 12.7.3.1
description: Invocation of `Symbol.toPrimitive` method during coercion
info: |
    [...]
    5. Let lprim be ? ToPrimitive(lval).
    6. Let rprim be ? ToPrimitive(rval).
    [...]

    ES6 Section 7.2.12 Abstract Equality Comparison

    [...]
    10. If Type(x) is either String, Number, or Symbol and Type(y) is Object,
        then return the result of the comparison x == ToPrimitive(y).

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

var left = {};
var right = {};
var log = '';
var leftThisVal, rightThisVal, leftArgs, rightArgs;

left[Symbol.toPrimitive] = function() {
  log += 'left';
  leftThisVal = this;
  leftArgs = arguments;
};

right[Symbol.toPrimitive] = function() {
  log += 'right';
  rightThisVal = this;
  rightArgs = arguments;
};


left + right;

assert.sameValue(log, 'leftright', 'methods invoked in correct sequence');

assert.sameValue(leftThisVal, left, 'left-hand side `this` value');
assert.sameValue(leftArgs.length, 1, 'left-hand side argument length');
assert.sameValue(leftArgs[0], 'default', 'left-hand side argument value');

assert.sameValue(rightThisVal, right, 'right-hand side `this` value');
assert.sameValue(rightArgs.length, 1, 'right-hand side argument length');
assert.sameValue(rightArgs[0], 'default', 'right-hand side argument value');
