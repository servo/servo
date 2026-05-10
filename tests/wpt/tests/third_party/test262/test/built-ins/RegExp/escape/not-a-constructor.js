// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: >
  RegExp.escape does not implement [[Construct]], is not new-able
info: |
  ECMAScript Function Objects

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.

  sec-evaluatenew

  ...
  7. If IsConstructor(constructor) is false, throw a TypeError exception.
  ...
includes: [isConstructor.js]
features: [RegExp.escape, Reflect.construct]
---*/

assert.sameValue(isConstructor(RegExp.escape), false, 'isConstructor(RegExp.escape) must return false');

assert.throws(TypeError, function () {
  new RegExp.escape();
});
