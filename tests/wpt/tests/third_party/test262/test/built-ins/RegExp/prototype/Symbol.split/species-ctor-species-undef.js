// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    RegExp used when the `Symbol.species` property of the `this` value's
    constructor is `undefined` or `null`
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
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
features: [Symbol.split, Symbol.species]
---*/

var noSpecies = function() {};
var re = /[db]/;
var result;
re.constructor = noSpecies;

noSpecies[Symbol.species] = undefined;
result = re[Symbol.split]('abcde');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'c');
assert.sameValue(result[2], 'e');

noSpecies[Symbol.species] = null;
result = re[Symbol.split]('abcde');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'c');
assert.sameValue(result[2], 'e');
