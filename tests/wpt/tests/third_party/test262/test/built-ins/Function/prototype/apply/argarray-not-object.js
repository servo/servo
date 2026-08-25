// Copyright 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.apply
description: >
  Throws a TypeError exception if argArray is not an object
info: |
  Function.prototype.apply ( thisArg, argArray )

  [...]
  4. Let argList be ? CreateListFromArrayLike(argArray).

  CreateListFromArrayLike ( obj [ , elementTypes ] )

  [...]
  2. If Type(obj) is not Object, throw a TypeError exception.
---*/

function fn() {}

assert.throws(TypeError, function() {
  fn.apply(null, true);
});

assert.throws(TypeError, function() {
  fn.apply(null, NaN);
});

assert.throws(TypeError, function() {
  fn.apply(null, '1,2,3');
});

assert.throws(TypeError, function() {
  fn.apply(null, Symbol());
});
