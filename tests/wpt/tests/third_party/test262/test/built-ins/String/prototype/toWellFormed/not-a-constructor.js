// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
  String.prototype.toWellFormed does not implement [[Construct]], is not new-able
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
features: [String.prototype.toWellFormed, Reflect.construct]
---*/

assert.sameValue(
  isConstructor(String.prototype.toWellFormed),
  false,
  'isConstructor(String.prototype.toWellFormed) must return false'
);

assert.throws(TypeError, function () {
  new String.prototype.toWellFormed();
});
