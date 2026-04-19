// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: >
    Behavior when `constructor` property is neither an Object nor undefined
info: |
    [...]
    8. Let A be ? ArraySpeciesCreate(O, count).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    9. If IsConstructor(C) is false, throw a TypeError exception.
---*/

var a = [];

a.constructor = null;
assert.throws(TypeError, function() {
  a.slice();
}, 'null value');

a = [];
a.constructor = 1;
assert.throws(TypeError, function() {
  a.slice();
}, 'number value');

a = [];
a.constructor = 'string';
assert.throws(TypeError, function() {
  a.slice();
}, 'string value');

a = [];
a.constructor = true;
assert.throws(TypeError, function() {
  a.slice();
}, 'boolean value');
