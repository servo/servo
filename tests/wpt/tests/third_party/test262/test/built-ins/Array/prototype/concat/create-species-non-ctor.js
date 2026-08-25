// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: >
    Behavior when the @@species attribute is a non-constructor object
info: |
    1. Let O be ? ToObject(this value).
    2. Let A be ? ArraySpeciesCreate(O, 0).

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    7. If Type(C) is Object, then
       a. Let C be ? Get(C, @@species).
       b. If C is null, let C be undefined.
    [...]
    9. If IsConstructor(C) is false, throw a TypeError exception.
includes: [isConstructor.js]
features: [Symbol.species, Reflect.construct]
---*/

assert.sameValue(
  isConstructor(parseInt),
  false,
  'precondition: isConstructor(parseInt) must return false'
);

var a = [];

a.constructor = {};
a.constructor[Symbol.species] = parseInt;

assert.throws(TypeError, function() {
  a.concat();
}, 'a.concat() throws a TypeError exception');
