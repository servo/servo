// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    TypeError thrown when `Symbol.species` property value is not a constructor
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
    6. ReturnIfAbrupt(C).

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
    10. Throw a TypeError exception.
features: [Symbol.split, Symbol.species]
---*/

var re = /./;
re.constructor = function() {};

// Avoid false positives from unrelated TypeErrors
re[Symbol.split]();

re.constructor[Symbol.species] = {};
assert.throws(TypeError, function() {
  re[Symbol.split]();
});

re.constructor[Symbol.species] = 0;
assert.throws(TypeError, function() {
  re[Symbol.split]();
});

re.constructor[Symbol.species] = '';
assert.throws(TypeError, function() {
  re[Symbol.split]();
});

re.constructor[Symbol.species] = Symbol.split;
assert.throws(TypeError, function() {
  re[Symbol.split]();
});

re.constructor[Symbol.species] = Date.now;
assert.throws(TypeError, function() {
  re[Symbol.split]();
});
