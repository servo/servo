// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
  SharedArrayBuffer.prototype.slice does not implement [[Construct]], is not new-able
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
features: [Reflect.construct, SharedArrayBuffer, arrow-function]
---*/

assert.sameValue(
  isConstructor(SharedArrayBuffer.prototype.slice),
  false,
  'isConstructor(SharedArrayBuffer.prototype.slice) must return false'
);

assert.throws(TypeError, () => {
  let sab = new SharedArrayBuffer(1); new sab.slice();
});

