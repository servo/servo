// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws a TypeError if `this` is a SharedArrayBuffer
features: [SharedArrayBuffer]
---*/

assert.throws(TypeError, function() {
  var sab = new SharedArrayBuffer(0);
  ArrayBuffer.prototype.slice.call(sab);
}, "`this` value cannot be a SharedArrayBuffer");
