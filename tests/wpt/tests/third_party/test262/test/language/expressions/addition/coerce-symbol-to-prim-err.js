// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addition-operator-plus-runtime-semantics-evaluation
es6id: 12.7.3.1
description: >
    Behavior when error thrown by invocation of `Symbol.toPrimitive` method
    during coercion
info: |
    [...]
    5. Let lprim be ? ToPrimitive(lval).
    6. Let rprim be ? ToPrimitive(rval).
    [...]

    ES6 Section 7.1.1 ToPrimitive ( input [, PreferredType] )

    [...]
    4. Let exoticToPrim be GetMethod(input, @@toPrimitive).
    5. ReturnIfAbrupt(exoticToPrim).
    6. If exoticToPrim is not undefined, then
       a. Let result be Call(exoticToPrim, input, «hint»).
       b. ReturnIfAbrupt(result).
features: [Symbol.toPrimitive]
---*/

var thrower = {};
var counter = {};
var log;

Object.defineProperty(thrower, Symbol.toPrimitive, {
  get: function() {
    log += 'accessThrower';
    return function() { throw new Test262Error(); };
  }
});
Object.defineProperty(counter, Symbol.toPrimitive, {
  get: function() {
    log += 'accessCounter';
    return function() { log += 'callCounter'; };
  }
});

log = '';

assert.throws(Test262Error, function() {
  thrower + counter;
}, 'error thrown by left-hand side');
assert.sameValue(log, 'accessThrower');

log = '';

assert.throws(Test262Error, function() {
  counter + thrower;
}, 'error thrown by right-hand side');
assert.sameValue(log, 'accessCountercallCounteraccessThrower');
