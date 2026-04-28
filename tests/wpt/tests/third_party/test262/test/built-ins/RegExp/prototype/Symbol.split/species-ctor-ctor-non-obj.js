// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: TypeError when `constructor` property is defined but not an object
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
features: [Symbol.split]
---*/

var obj = { flags: '' };

// Avoid false positives from unrelated TypeErrors
RegExp.prototype[Symbol.split].call(obj);

obj.constructor = false;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj);
});

obj.constructor = 'string';
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj);
});

obj.constructor = Symbol.split;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj);
});

obj.constructor = 86;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj);
});

obj.constructor = null;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj);
});
