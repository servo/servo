// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Detached buffer is only checked after ToIndex(requestIndex)
features: [Float16Array]
includes: [detachArrayBuffer.js]
---*/

var buffer = new ArrayBuffer(6);
var sample = new DataView(buffer, 0);

$DETACHBUFFER(buffer);

assert.throws(RangeError, function() {
  sample.getFloat16(-1);
});

assert.throws(RangeError, function() {
  sample.getFloat16(Infinity);
}, "Infinity");
