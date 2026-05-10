// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Detached buffer is only checked after ToIndex(requestIndex)
features: [Float16Array]
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(RangeError, function() {
  sample.setFloat16(Infinity, 0);
}, "Infinity");

assert.throws(RangeError, function() {
  sample.setFloat16(-1, 0);
});
