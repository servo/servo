// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors does not see inherited properties.
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

var F = function() {};
F.prototype.a = {};
F.prototype.b = {};

var f = new F();
var bValue = {};
f.b = bValue; // shadow the prototype
Object.defineProperty(f, 'c', {
  enumerable: false,
  configurable: true,
  writable: false,
  value: {}
}); // solely an own property

var result = Object.getOwnPropertyDescriptors(f);

assert.sameValue(!!result.b, true, 'b has a descriptor');
assert.sameValue(!!result.c, true, 'c has a descriptor');

assert.sameValue(result.b.enumerable, true, 'b is enumerable');
assert.sameValue(result.b.configurable, true, 'b is configurable');
assert.sameValue(result.b.writable, true, 'b is writable');
assert.sameValue(result.b.value, bValue, 'b’s value is `bValue`');

assert.sameValue(result.c.enumerable, false, 'c is enumerable');
assert.sameValue(result.c.configurable, true, 'c is configurable');
assert.sameValue(result.c.writable, false, 'c is writable');
assert.sameValue(result.c.value, f.c, 'c’s value is `f.c`');

assert.sameValue(
  Object.keys(result).length,
  2,
  'result has same number of own property names as f'
);
