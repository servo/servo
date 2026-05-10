// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: >
    Behavior when `constructor` property is neither an Object nor undefined
    - if IsConstructor(C) is false, throw a TypeError exception.
features: [Array.prototype.flat]
---*/

assert.sameValue(typeof Array.prototype.flat, 'function');

var a = [];
a.constructor = null;
assert.throws(TypeError, function() {
  a.flat();
}, 'null value');

a = [];
a.constructor = 1;
assert.throws(TypeError, function() {
  a.flat();
}, 'number value');

a = [];
a.constructor = 'string';
assert.throws(TypeError, function() {
  a.flat();
}, 'string value');

a = [];
a.constructor = true;
assert.throws(TypeError, function() {
  a.flat();
}, 'boolean value');
