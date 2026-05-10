// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Return values from Buffer
features: [Float16Array, DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 66); // 01000010
sample.setUint8(1, 40); // 00101000
sample.setUint8(2, 64); // 01000000
sample.setUint8(3, 224); // 11100000

assert.sameValue(sample.getFloat16(0, false), 3.078125, "0, false"); // 01000010 00101000
assert.sameValue(sample.getFloat16(1, false), 0.033203125, "1, false"); // 00101000 01000000
assert.sameValue(sample.getFloat16(2, false), 2.4375, "2, false"); // 01000000 11100000

assert.sameValue(sample.getFloat16(0, true), 0.03326416015625, "0, true"); // 00101000 01000010
assert.sameValue(sample.getFloat16(1, true), 2.078125, "1, true"); // 01000000 00101000
assert.sameValue(sample.getFloat16(2, true), -544, "2, true"); // 11100000 01000000

sample.setUint8(0, 75); // 01001011
sample.setUint8(1, 75); // 01001011
sample.setUint8(2, 76); // 01001100
sample.setUint8(3, 76); // 01001101

assert.sameValue(sample.getFloat16(0, false), 14.5859375, "0, false"); // 01001011 01001011
assert.sameValue(sample.getFloat16(1, false), 14.59375, "1, false"); // 01001011 01001100
assert.sameValue(sample.getFloat16(2, false), 17.1875, "2, false"); // 01001100 01001101
assert.sameValue(sample.getFloat16(0, true), 14.5859375, "0, true"); // 01001011 01001011
assert.sameValue(sample.getFloat16(1, true), 17.171875, "1, true"); // 01001100 01001011
assert.sameValue(sample.getFloat16(2, true), 17.1875, "2, true"); // 01001100 01001101
