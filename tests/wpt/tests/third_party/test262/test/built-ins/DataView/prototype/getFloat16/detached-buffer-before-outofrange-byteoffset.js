// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Detached buffer is checked before out of range byteOffset's value
features: [Float16Array]
includes: [detachArrayBuffer.js]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(TypeError, function() {
  sample.getFloat16(13);
}, "13");
