// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Invocation of custom species constructor
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
    [...]
    13. Let splitter be Construct(C, «rx, newFlags»).
    [...]

    ES6 Section 7.3.20 SpeciesConstructor ( O, defaultConstructor )

    1. Assert: Type(O) is Object.
    2. Let C be Get(O, "constructor").
    3. ReturnIfAbrupt(C).
    4. If C is undefined, return defaultConstructor.
    5. If Type(C) is not Object, throw a TypeError exception.
    6. Let S be Get(C, @@species).
    7. ReturnIfAbrupt(S).
    8. If S is either undefined or null, return defaultConstructor.
    9. If IsConstructor(S) is true, return S.
features: [Symbol.split, Symbol.species]
---*/

var thisVal, args, result;
var re = /x/iy;
re.constructor = function() {};
re.constructor[Symbol.species] = function() {
  thisVal = this;
  args = arguments;
  return /[db]/y;
};

result = RegExp.prototype[Symbol.split].call(re, 'abcde');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'c');
assert.sameValue(result[2], 'e');

assert(thisVal instanceof re.constructor[Symbol.species]);
assert.sameValue(args.length, 2);
assert.sameValue(args[0], re);
assert.sameValue(args[1], 'iy');
