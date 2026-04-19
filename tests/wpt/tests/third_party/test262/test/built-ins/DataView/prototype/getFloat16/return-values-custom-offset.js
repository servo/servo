// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Return values from Buffer using a custom offset
features: [Float16Array, DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(4, 75); // 01001011
sample.setUint8(5, 75); // 01001011
sample.setUint8(6, 76); // 01001100
sample.setUint8(7, 77); // 01001101

sample = new DataView(buffer, 4);

assert.sameValue(sample.getFloat16(0, false), 14.5859375, "0, false"); // 01001011 01001011
assert.sameValue(sample.getFloat16(1, false), 14.59375, "1, false"); // 01001011 01001100
assert.sameValue(sample.getFloat16(2, false), 17.203125, "2, false"); // 01001100 01001101
assert.sameValue(sample.getFloat16(0, true), 14.5859375, "0, true"); // 01001011 01001011
assert.sameValue(sample.getFloat16(1, true), 17.171875, "1, true"); // 01001100 01001011
assert.sameValue(sample.getFloat16(2, true), 21.1875, "2, true"); // 01001101 01001100
