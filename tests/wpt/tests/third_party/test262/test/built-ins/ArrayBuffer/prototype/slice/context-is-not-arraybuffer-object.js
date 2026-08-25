// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have an [[ArrayBufferData]] internal slot, throw a TypeError exception.
  ...
---*/

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.slice.call({});
}, "`this` value is Object");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.slice.call([]);
}, "`this` value is Array");
