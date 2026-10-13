// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: ArrayBuffer.prototype.transferToImmutable is not a constructor function
info: |
  ECMAScript Standard Built-in Objects
  ...
  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in the
  description of a particular function.
features: [Reflect.construct, immutable-arraybuffer]
includes: [isConstructor.js]
---*/

assert(!isConstructor(ArrayBuffer.prototype.transferToImmutable),
  "ArrayBuffer.prototype.transferToImmutable is not a constructor");

var arrayBuffer = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  new arrayBuffer.transferToImmutable();
});
