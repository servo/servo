// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  ArrayBuffer.prototype.slice is not a constructor function.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  17 ECMAScript Standard Built-in Objects:
    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified
    in the description of a particular function.
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert(!isConstructor(ArrayBuffer.prototype.slice), "ArrayBuffer.prototype.slice is not a constructor");

var arrayBuffer = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  new arrayBuffer.slice();
});
