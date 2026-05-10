// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.12
description: >
  Prevent extensions on target.
info: |
  26.1.12 Reflect.preventExtensions ( target )

  ...
  2. Return target.[[PreventExtensions]]().

  9.1.4 [[PreventExtensions]] ( )

  1. Set the value of the [[Extensible]] internal slot of O to false.
  ...
features: [Reflect]
---*/

var o = {};
Reflect.preventExtensions(o);
assert.sameValue(Object.isExtensible(o), false, 'object is not extensible');

assert.throws(TypeError, function() {
  Object.defineProperty(o, 'y', {});
});
assert.throws(TypeError, function() {
  Object.setPrototypeOf(o, Array.prototype);
});

Reflect.preventExtensions(o);
assert.sameValue(
  Object.isExtensible(o), false,
  'object is still not extensible on exhausted calls'
);
