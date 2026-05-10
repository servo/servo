// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
  %TypedArray%.prototype.with does not implement [[Construct]], is not new-able
info: |
  ECMAScript Function Objects

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.

  sec-evaluatenew

  ...
  7. If IsConstructor(constructor) is false, throw a TypeError exception.
  ...
includes: [isConstructor.js, testTypedArray.js]
features: [TypedArray, change-array-by-copy, Reflect.construct]
---*/

assert.sameValue(
  isConstructor(TypedArray.prototype.with),
  false,
  'isConstructor(TypedArray.prototype.with) must return false'
);

assert.throws(TypeError, () => {
  new TypedArray.prototype.with(0, 1);
}, '`new %TypedArray%.prototype.with()` throws TypeError');

