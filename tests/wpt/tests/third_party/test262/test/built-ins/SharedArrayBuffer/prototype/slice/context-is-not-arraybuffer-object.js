// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
features: [SharedArrayBuffer]
---*/

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call({});
}, "`this` value is Object");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.slice.call([]);
}, "`this` value is Array");
