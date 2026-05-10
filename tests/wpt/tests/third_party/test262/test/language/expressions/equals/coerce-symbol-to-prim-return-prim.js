// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.10.3
description: >
    Behavior when coercion via `Symbol.toPrimitive` yields a primitive value
info: |
    [...]
    7. Return the result of performing Abstract Equality Comparison rval ==
       lval.

    ES6 Section 7.2.12 Abstract Equality Comparison

    [...]
    10. If Type(x) is either String, Number, or Symbol and Type(y) is Object,
        then return the result of the comparison x == ToPrimitive(y).

    ES6 Section 7.1.1 ToPrimitive ( input [, PreferredType] )

    [...]
    4. Let exoticToPrim be GetMethod(input, @@toPrimitive).
    5. ReturnIfAbrupt(exoticToPrim).
    6. If exoticToPrim is not undefined, then
       a. Let result be Call(exoticToPrim, input, «hint»).
       b. ReturnIfAbrupt(result).
       c. If Type(result) is not Object, return result.
features: [Symbol.toPrimitive]
---*/

var y = {};
var retVal;

y[Symbol.toPrimitive] = function() {
  return retVal;
};

retVal = 86;
assert.sameValue(0 == y, false, 'number primitive (not equal)');
assert.sameValue(86 == y, true, 'number primitive (equal)');

retVal = 'str';
assert.sameValue(0 == y, false, 'string primitive (not equal)');
assert.sameValue('str' == y, true, 'sting primitive (equal)');

retVal = Symbol.toPrimitive;
assert.sameValue(0 == y, false, 'symbol (not equal)');
assert.sameValue(Symbol.toPrimitive == y, true, 'symbol (equal)');
