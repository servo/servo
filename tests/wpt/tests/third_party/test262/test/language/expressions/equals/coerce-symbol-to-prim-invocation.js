// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.10.3
description: Invocation of `Symbol.toPrimitive` method during coercion
info: |
    [...]
    7. Return the result of performing Abstract Equality Comparison rval ==
       lval.

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

var y = {};
var callCount = 0;
var thisVal, args;

y[Symbol.toPrimitive] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
};

0 == y;

assert.sameValue(callCount, 1, 'method invoked exactly once');
assert.sameValue(thisVal, y, '`this` value is the object being compared');
assert.sameValue(args.length, 1, 'method invoked with exactly one argument');
assert.sameValue(
  args[0],
  'default',
  'method invoked with the string "default" as the first argument'
);
