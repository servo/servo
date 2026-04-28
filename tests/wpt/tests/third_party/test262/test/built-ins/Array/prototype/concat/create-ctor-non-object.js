// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: >
    Behavior when `constructor` property is neither an Object nor undefined
info: |
    1. Let O be ? ToObject(this value).
    2. Let A be ? ArraySpeciesCreate(O, 0).

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    9. If IsConstructor(C) is false, throw a TypeError exception.
---*/

var a = [];

a.constructor = null;
assert.throws(TypeError, function() {
  a.concat();
}, 'a.concat() throws a TypeError exception');

a = [];
a.constructor = 1;
assert.throws(TypeError, function() {
  a.concat();
}, 'a.concat() throws a TypeError exception');

a = [];
a.constructor = 'string';
assert.throws(TypeError, function() {
  a.concat();
}, 'a.concat() throws a TypeError exception');

a = [];
a.constructor = true;
assert.throws(TypeError, function() {
  a.concat();
}, 'a.concat() throws a TypeError exception');
