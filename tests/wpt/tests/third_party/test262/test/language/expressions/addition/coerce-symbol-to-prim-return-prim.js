// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addition-operator-plus-runtime-semantics-evaluation
es6id: 12.7.3.1
description: >
    Behavior when coercion via `Symbol.toPrimitive` yields a primitive value
info: |
    [...]
    5. Let lprim be ? ToPrimitive(lval).
    6. Let rprim be ? ToPrimitive(rval).
    7. If Type(lprim) is String or Type(rprim) is String, then
       a. Let lstr be ? ToString(lprim).
       b. Let rstr be ? ToString(rprim).
       c. Return the String that is the result of concatenating lstr and rstr.
    8. Let lnum be ? ToNumber(lprim).
    9. Let rnum be ? ToNumber(rprim).
    10. Return the result of applying the addition operation to lnum and rnum.
        See the Note below 12.8.5. 

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
assert.sameValue(1 + y, 87);
assert.sameValue(y + 2, 88);
assert.sameValue('s' + y, 's86');
assert.sameValue(y + 's', '86s');

retVal = 'str';
assert.sameValue(0 + y, '0str');
assert.sameValue(y + 0, 'str0');
assert.sameValue('s' + y, 'sstr');
assert.sameValue(y + 's', 'strs');

retVal = Symbol.toPrimitive;
assert.throws(TypeError, function() {
  0 + y;
}, 'ToNumber(Symbol): right-hand side');
assert.throws(TypeError, function() {
  y + 0;
}, 'ToNumber(Symbol): left-hand side');
assert.throws(TypeError, function() {
  '' + y;
}, 'ToString(Symbol): right-hand side');
assert.throws(TypeError, function() {
  y + '';
}, 'ToString(Symbol): left-hand size');
